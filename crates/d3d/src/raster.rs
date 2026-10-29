//! The triangle walker with Direct3D's rasterisation rules.
//!
//! Public documentation (the Direct3D "Rasterization Rules" and "Directly Mapping Texels to
//! Pixels" pages): a pixel's centre is at integer screen coordinates (pixel `(x, y)` is
//! sampled at `x`, `y`, not `x + 0.5`), and a triangle owns the pixels whose centres are
//! inside it or on a top or left edge (the top-left filling convention), so two triangles
//! that share an edge draw each pixel once.

/// A point of a triangle with its attributes: depth `z` (interpolated in screen space),
/// `rhw`, and up to twelve values `a`. With `perspective`, `a[..perspective_count]` are
/// interpolated as `a × rhw` and divided by the interpolated `rhw` (perspective-correct);
/// the rest are interpolated in screen space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rhw: f32,
    pub a: [f32; 12],
}

/// One pixel the walker hands out: its position, depth and attributes.
pub struct Fragment {
    pub x: usize,
    pub y: usize,
    pub z: f32,
    pub a: [f32; 12],
}

fn edge(a: &Point, b: &Point, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

/// With the triangle wound so its area is positive (clockwise on a y-down screen), is the
/// edge from `a` to `b` a top edge (horizontal, above the triangle) or a left edge?
fn top_left(a: &Point, b: &Point) -> bool {
    (a.y == b.y && b.x > a.x) || b.y < a.y
}

/// The signed area of a screen triangle, times two: positive when its vertices run clockwise
/// on the screen (y down).
pub fn area(v: &[Point; 3]) -> f32 {
    edge(&v[0], &v[1], v[2].x, v[2].y)
}

/// Walks one triangle inside a `w` × `h` target and calls `pixel` for each pixel it owns.
pub fn triangle(v: [Point; 3], w: usize, h: usize, perspective: Option<usize>, pixel: &mut impl FnMut(Fragment)) {
    let [mut a, mut b, c] = v;
    let mut ar = edge(&a, &b, c.x, c.y);
    if ar == 0.0 || !ar.is_finite() {
        return;
    }
    if ar < 0.0 {
        std::mem::swap(&mut a, &mut b);
        ar = -ar;
    }
    let pc = perspective.unwrap_or(0).min(12);
    let pre = |p: &Point| {
        let mut q = p.a;
        if perspective.is_some() {
            for x in q.iter_mut().take(pc) {
                *x *= p.rhw;
            }
        }
        q
    };
    let (qa, qb, qc) = (pre(&a), pre(&b), pre(&c));
    // Pixel centres at integers: the box runs from ceil(min) to floor(max).
    let x0 = a.x.min(b.x).min(c.x).ceil().max(0.0) as i64;
    let x1 = (a.x.max(b.x).max(c.x).floor() as i64).min(w as i64 - 1);
    let y0 = a.y.min(b.y).min(c.y).ceil().max(0.0) as i64;
    let y1 = (a.y.max(b.y).max(c.y).floor() as i64).min(h as i64 - 1);
    let owns = |e: f32, tl: bool| e > 0.0 || (e == 0.0 && tl);
    let (ta, tb, tc) = (top_left(&b, &c), top_left(&c, &a), top_left(&a, &b));
    for y in y0..=y1 {
        let py = y as f32;
        for x in x0..=x1 {
            let px = x as f32;
            let (wa, wb, wc) = (edge(&b, &c, px, py), edge(&c, &a, px, py), edge(&a, &b, px, py));
            if !(owns(wa, ta) && owns(wb, tb) && owns(wc, tc)) {
                continue;
            }
            let (la, lb, lc) = (wa / ar, wb / ar, wc / ar);
            let z = la * a.z + lb * b.z + lc * c.z;
            let mut at = [0f32; 12];
            for (i, o) in at.iter_mut().enumerate() {
                *o = la * qa[i] + lb * qb[i] + lc * qc[i];
            }
            if perspective.is_some() {
                let rhw = la * a.rhw + lb * b.rhw + lc * c.rhw;
                if rhw != 0.0 {
                    for o in at.iter_mut().take(pc) {
                        *o /= rhw;
                    }
                }
            }
            pixel(Fragment { x: x as usize, y: y as usize, z, a: at });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f32, y: f32) -> Point {
        Point { x, y, rhw: 1.0, ..Point::default() }
    }

    fn cover(tris: &[[Point; 3]], w: usize, h: usize) -> Vec<u32> {
        let mut n = vec![0u32; w * h];
        for t in tris {
            triangle(*t, w, h, None, &mut |f| n[f.y * w + f.x] += 1);
        }
        n
    }

    #[test]
    fn a_square_owns_its_top_and_left_edges() {
        // Corners at 1 and 3: pixels 1 and 2 on each axis, the right and bottom edges out.
        let sq = [[p(1.0, 1.0), p(3.0, 1.0), p(3.0, 3.0)], [p(1.0, 1.0), p(3.0, 3.0), p(1.0, 3.0)]];
        let n = cover(&sq, 5, 5);
        let drawn: Vec<usize> = (0..25).filter(|&i| n[i] > 0).collect();
        assert_eq!(drawn, vec![6, 7, 11, 12]);
        assert!(n.iter().all(|&c| c <= 1));
    }

    #[test]
    fn pixel_centres_are_at_integers() {
        // A triangle from x 0.4 to 1.6 holds the centre x = 1 only.
        let t = [[p(0.4, 0.0), p(1.6, 0.0), p(1.0, 3.0)]];
        let n = cover(&t, 3, 3);
        assert_eq!(n[1], 1);
        assert_eq!(n[0] + n[2], 0);
    }

    #[test]
    fn perspective_divides_by_rhw() {
        let mut a = p(0.0, 0.0);
        let mut b = p(8.0, 0.0);
        let mut c = p(0.0, 8.0);
        a.a[0] = 0.0;
        b.a[0] = 1.0;
        c.a[0] = 0.0;
        b.rhw = 0.25; // b is four times as far.
        let mut seen = None;
        triangle([a, b, c], 9, 9, Some(1), &mut |f| {
            if (f.x, f.y) == (4, 0) {
                seen = Some(f.a[0]);
            }
        });
        // Halfway on screen is a fifth of the way in u: (0.5 × 0.25) / (0.5 + 0.125).
        assert!((seen.unwrap() - 0.2).abs() < 1e-6);
    }
}
