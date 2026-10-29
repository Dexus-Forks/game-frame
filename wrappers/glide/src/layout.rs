//! Where a texture's mip levels lie in TMU memory. Plain Rust, tested on the host.
//!
//! The game passes texel data packed tightly, each level after the one before. On the TMU each
//! level takes at least 2 × 2 texels: Glide 2.43's table of level sizes gives 4 bytes to an
//! 8-bit 1 × 1 level and 16 to an 8-bit 8 × 1 one (*measured*: the table that
//! `grTexCalcMemRequired` sums, at 0x1004b920 in the Win95 `glide2x.dll`). So memory sizes
//! (`grTexCalcMemRequired`) and level offsets use the padded sizes, and the data copied in is
//! the tight one.
//!
//! Multibase (`grTexMultibase`) splits a texture into four ranges at their own addresses:
//! level 256, level 128, level 64, and levels 32 down to 1 (`GR_TEXBASE_*`). Within a range
//! the levels follow each other from the range's first level.

use voodoo::formats::{Format, size};

/// TMU bytes of one level: at least 2 × 2 texels.
pub fn level_bytes(lod: u32, aspect: u32, format: Format) -> Option<usize> {
    let (w, h) = size(lod, aspect)?;
    Some(format.texel().bytes(w.max(2), h.max(2)))
}

/// Bytes of one level in the game's data: its texels, packed.
pub fn host_level_bytes(lod: u32, aspect: u32, format: Format) -> Option<usize> {
    let (w, h) = size(lod, aspect)?;
    Some(format.texel().bytes(w, h))
}

/// TMU offset of level `lod` from level `first` (`first` ≤ `lod`).
pub fn level_offset(first: u32, lod: u32, aspect: u32, format: Format) -> Option<usize> {
    (first..lod).map(|l| level_bytes(l, aspect, format)).sum()
}

/// Offset of level `lod` in the game's data for a chain that starts at `large`.
pub fn host_offset(large: u32, lod: u32, aspect: u32, format: Format) -> Option<usize> {
    (large..lod).map(|l| host_level_bytes(l, aspect, format)).sum()
}

/// `grTexCalcMemRequired`: the levels `large` to `small`, rounded up to 8 bytes.
pub fn mem_required(small: u32, large: u32, aspect: u32, format: Format) -> u32 {
    let n: usize = (large..=small).filter_map(|l| level_bytes(l, aspect, format)).sum();
    (n as u32 + 7) & !7
}

/// The multibase range of a level: 0, 1, 2 for levels 256, 128, 64; 3 for 32 to 1.
pub fn range(lod: u32) -> usize {
    lod.min(3) as usize
}

/// TMU address of level `lod`. Without multibase the chain starts at `start` with level
/// `large`; with it, each range starts at its own address with its own first level.
pub fn level_address(start: u32, bases: Option<[u32; 4]>, large: u32, lod: u32, aspect: u32, format: Format) -> Option<usize> {
    match bases {
        None => Some(start as usize + level_offset(large, lod, aspect, format)?),
        Some(b) => {
            let r = range(lod);
            Some(b[r] as usize + level_offset((r as u32).max(large), lod, aspect, format)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_levels_take_two_by_two() {
        // GR_LOD_1, GR_ASPECT_1x1, 8-bit: 4 bytes. GR_LOD_8 at 8x1: 16. At 16-bit, twice.
        assert_eq!(level_bytes(8, 3, Format::P8), Some(4));
        assert_eq!(level_bytes(5, 0, Format::P8), Some(16));
        assert_eq!(level_bytes(5, 0, Format::Rgb565), Some(32));
        assert_eq!(level_bytes(0, 3, Format::Rgb565), Some(256 * 256 * 2));
        assert_eq!(host_level_bytes(8, 3, Format::P8), Some(1));
    }

    #[test]
    fn mem_required_sums_padded_levels() {
        // 256x256 8-bit down to 1x1: 65536 + 16384 + 4096 + 1024 + 256 + 64 + 16 + 4 + 4.
        assert_eq!(mem_required(8, 0, 3, Format::P8), 87384);
        // 16-bit doubles it; a lone 1x1 level still takes 8 bytes.
        assert_eq!(mem_required(8, 0, 3, Format::Rgb565), 174768);
        assert_eq!(mem_required(8, 8, 3, Format::Rgb565), 8);
        // 8x1 256x32 chain, 8-bit: 8192 + 2048 + 512 + 128 + 32 + 16 + 8 + 4 + 4.
        assert_eq!(mem_required(8, 0, 0, Format::P8), 10944);
    }

    #[test]
    fn addresses() {
        let f = Format::Rgb565;
        // One base: level 64 of a chain from 256 is after 256 and 128.
        assert_eq!(level_address(0x1000, None, 0, 2, 3, f), Some(0x1000 + 131072 + 32768));
        // Multibase: each of the first three levels at its own address; 16 follows 32.
        let b = Some([0x10000, 0x40000, 0x80000, 0xc0000]);
        assert_eq!(level_address(0, b, 0, 0, 3, f), Some(0x10000));
        assert_eq!(level_address(0, b, 0, 2, 3, f), Some(0x80000));
        assert_eq!(level_address(0, b, 0, 3, 3, f), Some(0xc0000));
        assert_eq!(level_address(0, b, 0, 4, 3, f), Some(0xc0000 + 32 * 32 * 2));
        assert_eq!(range(8), 3);
        assert_eq!(host_offset(0, 2, 3, f), Some(131072 + 32768));
    }
}
