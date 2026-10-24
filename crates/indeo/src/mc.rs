//! Motion compensation, as Intel's Pentium path does it while it reads the macroblocks
//! (`FUN_10019540` at 0x1001E0E6, and the fragments its tables at 0x10077220 and 0x10077520
//! name). Every band sample is a multiple of 4, so Intel's word-wide shifts and masks keep the
//! two halves apart, and each rule below is that word arithmetic on one sample.

use super::blocks::{Band, Entry};

/// A reference sample; Intel reads whatever lies there, here outside the band reads as the
/// nearest row's sample past its end, or 0x4000.
fn at(r: &Band, x: i32, y: i32) -> u32 {
    if x < 0 || y < 0 {
        // Deviation: Intel reads the memory before the band (the band above it in the frame).
        return 0x4000;
    }
    let i = y as usize * (r.pitch as usize / 2) + x as usize;
    r.samples.get(i).copied().unwrap_or(0x4000) as u32
}

/// The prediction of one sample from `r` at (x, y) in samples, with the half-sample bits.
fn predict(r: &Band, x: i32, y: i32, hx: bool, hy: bool) -> u32 {
    let a = at(r, x, y);
    match (hx, hy) {
        (false, false) => a,
        // 0x1001E84B: ror both by 1, add, mask.
        (true, false) => ((a >> 1) + (at(r, x + 1, y) >> 1)) & 0x7FFC,
        // 0x1001ED13: add, shift by 1, mask.
        (false, true) => ((a + at(r, x, y + 1)) >> 1) & 0x7FFC,
        // 0x1001F0FF: the two vertical sums by 4, added, masked.
        (true, true) => (((a + at(r, x, y + 1)) >> 2) + ((at(r, x + 1, y) + at(r, x + 1, y + 1)) >> 2)) & 0x7FFC,
    }
}

/// The second vector's prediction averaged with the first, `p` (0x1001ABA8, 0x1001AE25,
/// 0x1001B2D2, 0x1001B7BE).
fn average(r: &Band, p: u32, x: i32, y: i32, hx: bool, hy: bool) -> u32 {
    let a = at(r, x, y);
    match (hx, hy) {
        (false, false) => ((p + a) & 0xFFF8) >> 1,
        (true, false) => ((a >> 2) + (at(r, x + 1, y) >> 2) + (p >> 1)) & 0x7FFC,
        (false, true) => ((p >> 1) + ((a + at(r, x, y + 1)) >> 2)) & 0x7FFC,
        (true, true) => {
            ((((a + at(r, x, y + 1)) >> 2) + ((at(r, x + 1, y) + at(r, x + 1, y + 1)) >> 2) + p) & 0xFFF8) >> 1
        }
    }
}

/// A vector (a biased byte pair, vertical over horizontal) as a sample offset and half bits.
/// `negate` for backward vectors, which Intel takes the other way round.
fn split(v16: u32, half: bool, negate: bool) -> (i32, i32, bool, bool) {
    let (mut h, mut v) = ((v16 & 0xFF) as u8, (v16 >> 8 & 0xFF) as u8);
    if negate {
        (h, v) = (h.wrapping_neg(), v.wrapping_neg());
    }
    let (h, v) = (h as i32 - 0x80, v as i32 - 0x80);
    if half {
        // Horizontal: half samples are the low bit of the byte offset; vertical: an arithmetic
        // shift, whose carry is the half bit.
        (h >> 1, v >> 1, h & 1 != 0, v & 1 != 0)
    } else {
        (h, v, false, false)
    }
}

/// Predict a block into `cur` at its place from the forward and backward references.
pub(super) fn block(cur: &mut Band, fwd: &Band, bwd: &Band, e: &Entry, n: usize, half: bool) {
    let dir = e.flags & 0xC;
    if dir == 0 {
        return;
    }
    let pitch = cur.pitch as usize / 2;
    let x0 = (e.dest as usize % cur.pitch as usize) / 2;
    let y0 = e.dest as usize / cur.pitch as usize;
    let (first, negate) = if dir & 4 != 0 { (fwd, false) } else { (bwd, true) };
    let (dx, dy, hx, hy) = split(e.mv & 0xFFFF, half, negate);
    let second = (dir == 0xC).then(|| split(e.mv >> 16, half, true));
    for row in 0..n {
        for col in 0..n {
            let (x, y) = ((x0 + col) as i32, (y0 + row) as i32);
            let mut p = predict(first, x + dx, y + dy, hx, hy);
            if let Some((dx2, dy2, hx2, hy2)) = second {
                p = average(bwd, p, x + dx2, y + dy2, hx2, hy2);
            }
            if let Some(s) = cur.samples.get_mut((y0 + row) * pitch + x0 + col) {
                *s = p as u16;
            }
        }
    }
}
