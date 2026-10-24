//! From band samples to 8-bit planes (`FUN_10025f10` and the routines it calls on the Pentium
//! path): one band is clipped (`FUN_10025b00`, `FUN_10025d90`), four bands are recomposed by the
//! inverse Haar step (`FUN_10024eb0`). Intel's band words hold samples x and x + 8 (x and x + 4
//! for 4 × 4 blocks), which only matters to the four-band arithmetic, done here on such words.

use super::blocks::Band;

/// Intel's clip table at 0x10077070, indexed by sample / 4: 0 below 0xF80, then 0 to 255.
fn clip(s: u16) -> u8 {
    // Deviation: Intel's table is valid from index 0xB80 to 0x117F; outside it reads other data.
    ((s >> 2) as i32 - 0xF80).clamp(0, 255) as u8
}

/// One band to `out` (`w` × `h`, rows `w` apart).
pub(super) fn single(b: &Band, out: &mut [u8], w: usize, h: usize) {
    let pitch = b.pitch as usize / 2;
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = clip(b.samples[y * pitch + x]);
        }
    }
}

/// Intel's recomposition clip (0x100779F0), indexed by a sum / 16 in 10 bits: 0 to 255 as they
/// are, 256 to 511 → 255, 512 to 1023 (a negative sum) → 0.
fn clip10(v: u32) -> u8 {
    match v & 0x3FF {
        v @ 0..=255 => v as u8,
        256..=511 => 255,
        _ => 0,
    }
}

/// The two pixels a 32-bit sum gives, low half and high half (`v >> 4` per half; through the
/// clip when bits 8 or 9 of either are set).
fn pixels(v: u32) -> (u8, u8) {
    let s = v >> 4;
    if s & 0x0300_0300 != 0 { (clip10(s), clip10(v >> 20)) } else { (s as u8, (s >> 16) as u8) }
}

/// Four bands (LL, then the three others in the header's order) to `out` (`w` × `h`).
pub(super) fn four(bands: &[Band], out: &mut [u8], w: usize, h: usize) {
    let pitch = bands[0].pitch as usize / 2;
    let word = |b: &Band, y: usize, x: usize| -> u32 {
        let k = y * pitch + x;
        (b.samples.get(k + 8).copied().unwrap_or(0) as u32) << 16 | b.samples[k] as u32
    };
    for y in 0..h.div_ceil(2) {
        for g in (0..w.div_ceil(2)).step_by(16) {
            for j in 0..8 {
                let x = g + j;
                let (a, b, c, d) =
                    (word(&bands[0], y, x), word(&bands[1], y, x), word(&bands[2], y, x), word(&bands[3], y, x));
                let i3 = a.wrapping_sub(0x77F7_77F8).wrapping_sub(b);
                let i7 = c.wrapping_sub(d);
                let u4 = i3.wrapping_sub(i7);
                let i6 = i7.wrapping_add(0x7FFF_8000).wrapping_add(d.wrapping_mul(2));
                let u5 = u4.wrapping_add(i7.wrapping_mul(2));
                let u12 = i3.wrapping_add(0x7FFF_8000).wrapping_add(b.wrapping_mul(2)).wrapping_sub(i6);
                let u10 = u12.wrapping_add(i6.wrapping_mul(2));
                for (k, (lo, hi)) in [(0usize, pixels(u10)), (1, pixels(u12)), (2, pixels(u5)), (3, pixels(u4))] {
                    let (oy, ox) = (2 * y + k / 2, 2 * x + k % 2);
                    if oy < h {
                        if ox < w {
                            out[oy * w + ox] = lo;
                        }
                        if ox + 16 < w {
                            out[oy * w + ox + 16] = hi;
                        }
                    }
                }
            }
        }
    }
}
