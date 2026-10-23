//! Reducing textures the way 1997 loaders did to fit texture memory.

/// Halve a texture by point sampling: keep the texel at every even column of every even row.
/// `bytes_per_texel` is 1, 2 or 4; odd sizes drop the last row or column.
pub fn halve_point(data: &[u8], width: usize, height: usize, bytes_per_texel: usize) -> Vec<u8> {
    let (w2, h2) = (width / 2, height / 2);
    let mut out = Vec::with_capacity(w2 * h2 * bytes_per_texel);
    for y in 0..h2 {
        for x in 0..w2 {
            let o = ((2 * y) * width + 2 * x) * bytes_per_texel;
            out.extend_from_slice(&data[o..o + bytes_per_texel]);
        }
    }
    out
}
