//! Scalers for a finished frame of `0x00RRGGBB` pixels, rows top to bottom, to a whole
//! multiple of its size. Used by `ddraw` for the builds whose picture is drawn at 640 × 480
//! (the ATI, S3 and software builds), where scaling the frame is all a wrapper can do.
//!
//! - `nearest`: each pixel becomes a `k × k` block. Sharp, and exact at any whole factor.
//! - `scale2x`: Scale2x (also called EPX or AdvMAME2x), applied once per doubling: edges
//!   between flat colours come out diagonal instead of stepped, and nothing is blurred. Other
//!   factors finish with `nearest`.
//! - `crt`: `nearest`, then each pixel's last row darkened (a scanline) and a faint
//!   red/green/blue column mask, like a 1997 monitor seen up close.

/// Each pixel as a `k × k` block.
pub fn nearest(src: &[u32], w: usize, h: usize, k: usize) -> Vec<u32> {
    let mut out = vec![0; w * k * h * k];
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let line: Vec<u32> = row.iter().flat_map(|p| std::iter::repeat_n(*p, k)).collect();
        for dy in 0..k {
            let o = (y * k + dy) * w * k;
            out[o..o + w * k].copy_from_slice(&line);
        }
    }
    out
}

/// One Scale2x pass: `w × h` to `2w × 2h`. For pixel E with neighbours B (above), D (left),
/// F (right) and H (below), each quarter takes a neighbour's colour where two neighbours that
/// meet at that corner agree and the opposite ones differ.
pub fn scale2x_once(src: &[u32], w: usize, h: usize) -> Vec<u32> {
    let at = |x: isize, y: isize| src[y.clamp(0, h as isize - 1) as usize * w + x.clamp(0, w as isize - 1) as usize];
    let mut out = vec![0; 4 * w * h];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let (b, d, e, f, hh) = (at(x, y - 1), at(x - 1, y), at(x, y), at(x + 1, y), at(x, y + 1));
            let (mut e0, mut e1, mut e2, mut e3) = (e, e, e, e);
            if b != hh && d != f {
                if d == b {
                    e0 = d;
                }
                if b == f {
                    e1 = f;
                }
                if d == hh {
                    e2 = d;
                }
                if hh == f {
                    e3 = f;
                }
            }
            let (ox, oy) = (2 * x as usize, 2 * y as usize);
            out[oy * 2 * w + ox] = e0;
            out[oy * 2 * w + ox + 1] = e1;
            out[(oy + 1) * 2 * w + ox] = e2;
            out[(oy + 1) * 2 * w + ox + 1] = e3;
        }
    }
    out
}

/// Scale2x while the factor allows a doubling, then `nearest` for what is left (3× is one
/// `nearest` pass; 4× is two Scale2x passes; 6× is one pass, then 3×).
pub fn scale2x(src: &[u32], w: usize, h: usize, k: usize) -> Vec<u32> {
    let (mut img, mut cw, mut ch, mut left) = (src.to_vec(), w, h, k);
    while left % 2 == 0 && left >= 2 {
        img = scale2x_once(&img, cw, ch);
        cw *= 2;
        ch *= 2;
        left /= 2;
    }
    if left > 1 { nearest(&img, cw, ch, left) } else { img }
}

fn mul(p: u32, [r, g, b]: [u32; 3]) -> u32 {
    let c = |shift: u32, m: u32| (((p >> shift) & 0xff) * m / 256).min(255) << shift;
    c(16, r) | c(8, g) | c(0, b)
}

/// `nearest`, with a scanline on each pixel's last row (at 2× and above) and a column mask
/// (at 3× and above). The weights are a look, not a measurement of any monitor.
pub fn crt(src: &[u32], w: usize, h: usize, k: usize) -> Vec<u32> {
    let mut out = nearest(src, w, h, k);
    let ow = w * k;
    for (i, p) in out.iter_mut().enumerate() {
        let (x, y) = (i % ow, i / ow);
        let mut m = [256u32; 3];
        if k >= 2 && y % k == k - 1 {
            m = [150; 3];
        }
        if k >= 3 {
            // An aperture grille: each column favours one of red, green, blue.
            let c = x % 3;
            for (j, v) in m.iter_mut().enumerate() {
                *v = *v * if j == c { 290 } else { 220 } / 256;
            }
        }
        *p = mul(*p, m);
    }
    out
}

/// The filters by name, for a setting: `nearest`, `scale2x`, `crt`.
pub fn by_name(name: &str) -> Option<fn(&[u32], usize, usize, usize) -> Vec<u32>> {
    Some(match name {
        "nearest" => nearest,
        "scale2x" | "epx" => scale2x,
        "crt" => crt,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_makes_blocks() {
        let out = nearest(&[1, 2, 3, 4], 2, 2, 2);
        assert_eq!(out, vec![1, 1, 2, 2, 1, 1, 2, 2, 3, 3, 4, 4, 3, 3, 4, 4]);
    }

    #[test]
    fn scale2x_keeps_flat_areas_and_rounds_a_diagonal() {
        assert!(scale2x(&[7; 9], 3, 3, 2).iter().all(|p| *p == 7));
        // A diagonal edge: the white pixel at the centre's corner takes black where its two
        // black neighbours meet.
        let (k, o) = (0, 0xffffff);
        let src = [k, k, o, k, o, o, o, o, o];
        let out = scale2x_once(&src, 3, 3);
        assert_eq!(out[2 * 6 + 2], k, "top-left quarter of the centre follows the black corner");
        assert_eq!(out[3 * 6 + 3], o);
    }

    #[test]
    fn scale2x_finishes_odd_factors_with_nearest() {
        assert_eq!(scale2x(&[1, 2], 2, 1, 3).len(), 6 * 3);
        assert_eq!(scale2x(&[1, 2], 2, 1, 4).len(), 8 * 4);
    }

    #[test]
    fn crt_darkens_the_last_row_of_each_pixel() {
        let out = crt(&[0x808080], 1, 1, 2);
        assert_eq!(out[0], 0x808080);
        assert!(out[2] & 0xff < 0x80);
    }
}
