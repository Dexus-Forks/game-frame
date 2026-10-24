//! Scale2x (EPX): each pixel P becomes four. With its neighbours A (above), B (right),
//! C (left) and D (below), the top-left pixel is A when C == A, C != D and A != B, and P
//! otherwise; the other three follow by symmetry. A pixel on the edge uses itself for a
//! missing neighbour.

/// Twice the size, rounding diagonal edges and keeping flat areas.
pub fn scale2x<P: Copy + PartialEq>(src: &[P], w: usize, h: usize) -> Vec<P> {
    assert!(src.len() >= w * h, "scale2x: a frame shorter than w * h");
    if w * h == 0 {
        return Vec::new();
    }
    let at = |x: usize, y: usize| src[y * w + x];
    let mut out = vec![src[0]; 4 * w * h];
    let ow = 2 * w;
    for y in 0..h {
        for x in 0..w {
            let p = at(x, y);
            let a = if y > 0 { at(x, y - 1) } else { p };
            let d = if y + 1 < h { at(x, y + 1) } else { p };
            let c = if x > 0 { at(x - 1, y) } else { p };
            let b = if x + 1 < w { at(x + 1, y) } else { p };
            let (mut e0, mut e1, mut e2, mut e3) = (p, p, p, p);
            if a != d && c != b {
                if c == a { e0 = a; }
                if a == b { e1 = b; }
                if d == c { e2 = c; }
                if b == d { e3 = d; }
            }
            let o = 2 * y * ow + 2 * x;
            out[o] = e0;
            out[o + 1] = e1;
            out[o + ow] = e2;
            out[o + ow + 1] = e3;
        }
    }
    out
}

/// Four times the size: two passes of Scale2x.
pub fn scale4x<P: Copy + PartialEq>(src: &[P], w: usize, h: usize) -> Vec<P> {
    scale2x(&scale2x(src, w, h), 2 * w, 2 * h)
}
