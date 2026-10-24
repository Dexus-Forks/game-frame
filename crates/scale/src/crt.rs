//! A CRT-style look: nearest, then the last row of every block darkened to `keep` percent.

/// Each RGBA pixel (`0xAARRGGBB`, any channel order) becomes a `k` × `k` block; the last row of
/// each block keeps `keep` percent of each colour channel, and the top byte (alpha) as it is.
/// With `k` = 1 the frame is returned unchanged.
pub fn crt(src: &[u32], w: usize, h: usize, k: usize, keep: u32) -> Vec<u32> {
    let mut out = crate::nearest(src, w, h, k);
    if k < 2 {
        return out;
    }
    let keep = keep.min(100);
    let dim = |p: u32| {
        let ch = |s: u32| (((p >> s) & 0xff) * keep / 100) << s;
        (p & 0xff00_0000) | ch(16) | ch(8) | ch(0)
    };
    let ow = w * k;
    for y in 0..h {
        let row = (y * k + k - 1) * ow;
        for p in &mut out[row..row + ow] {
            *p = dim(*p);
        }
    }
    out
}
