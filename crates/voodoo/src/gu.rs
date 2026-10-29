//! Glide 2.x's texture memory manager, the `gu` calls a game uses instead of placing textures
//! itself: `guTexAllocateMemory` gives a mipmap id with its filter and clamp modes, or none when
//! the TMU's memory is full; `guTexDownloadMipMapLevel` fills a level; `guTexSource` selects the
//! texture and its modes; `guTexMemReset` frees everything; `guTexMemQueryAvail` says what is
//! left.
//!
//! Games learn how many textures fit by allocating until an allocation fails, so the memory is
//! modelled as a budget: [`Glide::set_texture_memory`], 2 MB by default (a Voodoo Graphics
//! board's usual TMU). Deviation: each allocation takes the texel bytes of its levels rounded
//! up to 8 bytes (the texture base address's granularity, SST-1 specification section 5.53);
//! Glide's own placement rules (a texture may not cross a 2 MB boundary, the even and odd
//! split) are not modelled.

use crate::formats::{Format, chain_bytes, size};
use crate::glide::Glide;

/// `GR_NULL_MIPMAP_HANDLE`.
pub const NULL_MIPMAP_HANDLE: u32 = 0xffff_ffff;

/// The textures' keys in [`Glide`]'s texture map: apart from `grTexDownloadMipMap`'s start
/// addresses.
const KEY: u32 = 0x8000_0000;

/// One `guTexAllocateMemory` record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MipMap {
    pub width: usize,
    pub height: usize,
    pub format: Format,
    pub large_lod: u32,
    pub clamp: [u32; 2],
    pub filter: [u32; 2],
}

#[derive(Clone, Debug)]
pub struct TexMemory {
    pub size: usize,
    pub used: usize,
    pub mipmaps: Vec<MipMap>,
}

impl Default for TexMemory {
    fn default() -> TexMemory {
        TexMemory {
            size: 2 << 20,
            used: 0,
            mipmaps: Vec::new(),
        }
    }
}

#[allow(non_snake_case)]
impl Glide {
    /// The TMU's texture memory in bytes; frees what was allocated.
    pub fn set_texture_memory(&mut self, bytes: usize) {
        self.guTexMemReset();
        self.gu.size = bytes;
    }

    /// `guTexMemQueryAvail(tmu)`.
    pub fn guTexMemQueryAvail(&self, _tmu: u32) -> usize {
        self.gu.size - self.gu.used
    }

    /// `guTexAllocateMemory(tmu, evenOdd, width, height, format, mmMode, smallLod, largeLod,
    /// aspect, sClamp, tClamp, minFilter, magFilter, lodBias, trilinear)`: a mipmap id, or
    /// [`NULL_MIPMAP_HANDLE`] when the levels do not fit or the arguments are not a texture.
    #[allow(clippy::too_many_arguments)]
    pub fn guTexAllocateMemory(
        &mut self,
        _tmu: u32,
        _even_odd: u32,
        _width: u32,
        _height: u32,
        format: u32,
        _mm_mode: u32,
        small_lod: u32,
        large_lod: u32,
        aspect: u32,
        s_clamp: u32,
        t_clamp: u32,
        min_filter: u32,
        mag_filter: u32,
        _lod_bias: f32,
        _trilinear: bool,
    ) -> u32 {
        let Some(fmt) = Format::from_u32(format) else {
            return NULL_MIPMAP_HANDLE;
        };
        let (Some(bytes), Some((width, height))) = (chain_bytes(small_lod, large_lod, aspect, fmt), size(large_lod, aspect)) else {
            return NULL_MIPMAP_HANDLE;
        };
        let bytes = (bytes + 7) & !7;
        if bytes > self.gu.size - self.gu.used {
            return NULL_MIPMAP_HANDLE;
        }
        self.gu.used += bytes;
        self.gu.mipmaps.push(MipMap {
            width,
            height,
            format: fmt,
            large_lod,
            clamp: [s_clamp, t_clamp],
            filter: [min_filter, mag_filter],
        });
        (self.gu.mipmaps.len() - 1) as u32
    }

    /// `guTexDownloadMipMapLevel(mmid, lod, &data)`. Only the largest level is kept (mipmaps
    /// are not modelled).
    pub fn guTexDownloadMipMapLevel(&mut self, mmid: u32, lod: u32, data: &[u8]) {
        let Some(&m) = self.gu.mipmaps.get(mmid as usize) else {
            return;
        };
        if lod == m.large_lod {
            self.download(KEY | mmid, m.format, m.width, m.height, data);
        }
    }

    /// `guTexSource(mmid)`: the texture, and its clamp and filter modes into the state.
    /// Deviation: one filter is kept, the magnification one, as `grTexFilterMode` here.
    pub fn guTexSource(&mut self, mmid: u32) {
        let Some(&m) = self.gu.mipmaps.get(mmid as usize) else {
            return;
        };
        self.source = Some(KEY | mmid);
        self.state.clamp = m.clamp;
        self.state.filter = m.filter[1];
    }

    /// `guTexMemReset()`: every mipmap freed.
    pub fn guTexMemReset(&mut self) {
        self.gu.used = 0;
        self.gu.mipmaps.clear();
        self.textures.retain(|k, _| k & KEY == 0);
        self.raw.retain(|k, _| k & KEY == 0);
        if self.source.is_some_and(|k| k & KEY != 0) {
            self.source = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::*;

    fn alloc(g: &mut Glide, lod: u32, format: Format) -> u32 {
        g.guTexAllocateMemory(0, 3, 0, 0, format as u32, 0, lod, lod, 3, TEXTURECLAMP_CLAMP, TEXTURECLAMP_CLAMP, TEXTUREFILTER_BILINEAR, TEXTUREFILTER_BILINEAR, 0.0, false)
    }

    #[test]
    fn allocations_stop_when_memory_is_full() {
        let mut g = Glide::open(4, 4, 1.0);
        // 256 × 256 at 16 bits is 128 KB: 16 fit in 2 MB.
        let ids: Vec<u32> = (0..20).map(|_| alloc(&mut g, 0, Format::Rgb565)).collect();
        assert_eq!(ids.iter().filter(|&&i| i != NULL_MIPMAP_HANDLE).count(), 16);
        assert_eq!(g.guTexMemQueryAvail(0), 0);
        g.guTexMemReset();
        assert_eq!(g.guTexMemQueryAvail(0), 2 << 20);
        assert_eq!(alloc(&mut g, 0, Format::Rgb565), 0);
    }

    #[test]
    fn source_sets_the_modes_and_a_new_palette_recolours() {
        let mut g = Glide::open(1, 1, 1.0);
        let id = alloc(&mut g, 8, Format::P8);
        g.grTexDownloadTable(0, 2, &[0x00ff0000; 256]);
        g.guTexDownloadMipMapLevel(id, 8, &[7]);
        g.guTexSource(id);
        assert_eq!(g.state.filter, TEXTUREFILTER_BILINEAR);
        assert_eq!(g.textures[&(KEY | id)].texels, [255, 0, 0, 255]);
        g.grTexDownloadTable(0, 2, &[0x000000ff; 256]);
        assert_eq!(g.textures[&(KEY | id)].texels, [0, 0, 255, 255]);
    }
}
