//! Triangles and convex polygons (fans).

use crate::pixel::Target;

/// A screen position and up to eight attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub a: [f32; 8],
}

fn edge(a: &Vertex, b: &Vertex, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

/// Is an edge from `a` to `b` (of a triangle wound so the area is positive) a top or a left
/// edge? Those own the pixels whose centres fall exactly on them.
fn top_left(a: &Vertex, b: &Vertex) -> bool {
    (a.y == b.y && b.x < a.x) || b.y < a.y
}

/// Draw one triangle. `shade(attributes, pixel already there)` gives the new pixel, or
/// `None` to leave it.
pub fn triangle(t: &mut Target, v: [Vertex; 3], shade: &mut impl FnMut(&[f32; 8], u32) -> Option<u32>) {
    let [mut a, mut b, c] = v;
    let mut area = edge(&a, &b, c.x, c.y);
    if area == 0.0 || !area.is_finite() {
        return;
    }
    if area < 0.0 {
        std::mem::swap(&mut a, &mut b);
        area = -area;
    }
    let x0 = a.x.min(b.x).min(c.x).floor().max(0.0) as i64;
    let x1 = (a.x.max(b.x).max(c.x).ceil() as i64).min(t.w as i64);
    let y0 = a.y.min(b.y).min(c.y).floor().max(0.0) as i64;
    let y1 = (a.y.max(b.y).max(c.y).ceil() as i64).min(t.h as i64);
    let bias = |p: &Vertex, q: &Vertex| if top_left(p, q) { 0.0 } else { -1e-6 };
    let (ba, bb, bc) = (bias(&b, &c), bias(&c, &a), bias(&a, &b));
    let mut at = [0f32; 8];
    for y in y0..y1 {
        let py = y as f32 + 0.5;
        for x in x0..x1 {
            let px = x as f32 + 0.5;
            let wa = edge(&b, &c, px, py);
            let wb = edge(&c, &a, px, py);
            let wc = edge(&a, &b, px, py);
            if wa + ba < 0.0 || wb + bb < 0.0 || wc + bc < 0.0 || (wa <= 0.0 && ba < 0.0) || (wb <= 0.0 && bb < 0.0) || (wc <= 0.0 && bc < 0.0) {
                continue;
            }
            let (la, lb, lc) = (wa / area, wb / area, wc / area);
            for (i, at) in at.iter_mut().enumerate() {
                *at = la * a.a[i] + lb * b.a[i] + lc * c.a[i];
            }
            let (ux, uy) = (x as usize, y as usize);
            if let Some(p) = shade(&at, t.get(ux, uy)) {
                t.set(ux, uy, p);
            }
        }
    }
}

/// A convex polygon as a fan from its first vertex.
pub fn polygon(t: &mut Target, v: &[Vertex], shade: &mut impl FnMut(&[f32; 8], u32) -> Option<u32>) {
    for i in 1..v.len().saturating_sub(1) {
        triangle(t, [v[0], v[i], v[i + 1]], shade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel::Format;

    fn target(buf: &mut [u8], w: usize, h: usize) -> Target<'_> {
        Target::new(buf, w * 2, w, h, Format::Rgb565)
    }

    fn pixels(t: &Target) -> Vec<u32> {
        (0..t.h).flat_map(|y| (0..t.w).map(move |x| (x, y))).map(|(x, y)| t.get(x, y)).collect()
    }

    fn v(x: f32, y: f32) -> Vertex {
        Vertex { x, y, a: [0.0; 8] }
    }

    #[test]
    fn shared_edges_draw_each_pixel_once() {
        let mut buf = vec![0u8; 16 * 16 * 2];
        let mut t = target(&mut buf, 16, 16);
        let quad = [v(1.0, 1.0), v(13.0, 2.0), v(14.0, 14.0), v(2.0, 12.0)];
        let mut add = |_: &[f32; 8], old: u32| Some(old + 1);
        polygon(&mut t, &quad, &mut add);
        let px = pixels(&t);
        assert!(px.iter().all(|p| *p <= 1), "a pixel drawn twice");
        assert!(px.iter().filter(|p| **p == 1).count() > 100);
    }

    #[test]
    fn a_square_covers_its_pixels() {
        let mut buf = vec![0u8; 8 * 8 * 2];
        let mut t = target(&mut buf, 8, 8);
        let sq = [v(2.0, 2.0), v(6.0, 2.0), v(6.0, 6.0), v(2.0, 6.0)];
        polygon(&mut t, &sq, &mut |_, _| Some(1));
        let n: u32 = pixels(&t).iter().sum();
        assert_eq!(n, 16);
        assert_eq!(t.get(2, 2), 1);
        assert_eq!(t.get(6, 6), 0);
    }

    #[test]
    fn colours_interpolate() {
        let mut buf = vec![0u8; 4 * 2];
        let mut t = target(&mut buf, 4, 1);
        let mut a = v(0.0, 0.0);
        let mut b = v(4.0, 0.0);
        let mut c = v(4.0, 4.0);
        a.a[0] = 0.0;
        b.a[0] = 4.0;
        c.a[0] = 4.0;
        let mut seen = vec![];
        triangle(&mut t, [a, b, c], &mut |at, _| {
            seen.push(at[0]);
            None
        });
        assert!(seen.windows(2).all(|w| w[0] < w[1]));
    }
}
