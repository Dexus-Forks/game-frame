//! The Glide `GrVertex`, and what the drawing calls do to vertices before they are recorded:
//! the signed area that culling tests, fans for polygons, and clipping to a rectangle for the
//! `gu…WithClip` calls. Plain Rust, tested on the host.

/// A Glide `GrVertex` (Glide 2.x with two TMUs): 15 floats, 60 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub ooz: f32,
    pub a: f32,
    pub oow: f32,
    pub tmu: [[f32; 3]; 2],
}

const FLOATS: usize = 15;

impl Vertex {
    fn floats(&self) -> [f32; FLOATS] {
        // Safety: Vertex is repr(C) and holds exactly 15 f32.
        unsafe { std::mem::transmute::<Vertex, [f32; FLOATS]>(*self) }
    }

    fn from_floats(f: [f32; FLOATS]) -> Vertex {
        unsafe { std::mem::transmute::<[f32; FLOATS], Vertex>(f) }
    }

    /// The vertex `t` of the way from `self` to `b`. Every Glide parameter (colour, `ooz`,
    /// `oow`, `sow`, `tow`) is linear in screen space, so this is exact.
    pub fn lerp(&self, b: &Vertex, t: f32) -> Vertex {
        let (p, q) = (self.floats(), b.floats());
        Vertex::from_floats(std::array::from_fn(|i| p[i] + (q[i] - p[i]) * t))
    }
}

/// Twice the signed area of the triangle on screen. With y down, a triangle whose vertices run
/// clockwise on screen has a positive area. `GR_CULL_NEGATIVE` drops the negative ones,
/// `GR_CULL_POSITIVE` the positive ones.
pub fn area(a: &Vertex, b: &Vertex, c: &Vertex) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// `GrCullMode_t`.
pub const CULL_DISABLE: u32 = 0;
pub const CULL_NEGATIVE: u32 = 1;
pub const CULL_POSITIVE: u32 = 2;

/// Whether `mode` drops a triangle of this area. A triangle of zero area is never drawn by the
/// Voodoo, so it is dropped in every mode.
pub fn culled(mode: u32, area: f32) -> bool {
    match mode {
        CULL_NEGATIVE => area <= 0.0,
        CULL_POSITIVE => area >= 0.0,
        _ => area == 0.0,
    }
}

/// The triangles of a convex polygon of `n` vertices as a fan from its first vertex, as
/// `grDrawPolygon` draws it.
pub fn fan(n: usize) -> impl Iterator<Item = [usize; 3]> {
    (1..n.saturating_sub(1)).map(|i| [0, i, i + 1])
}

/// Clip a convex polygon to the rectangle `[minx, miny, maxx, maxy]` (Sutherland–Hodgman, one
/// edge at a time). Returns the vertices of what is inside, none if nothing is.
pub fn clip(poly: &[Vertex], [minx, miny, maxx, maxy]: [f32; 4]) -> Vec<Vertex> {
    let mut out = poly.to_vec();
    // Each edge: the coordinate (x or y), the limit, and whether inside is below the limit.
    for (use_x, limit, below) in [(true, minx, false), (true, maxx, true), (false, miny, false), (false, maxy, true)] {
        if out.is_empty() {
            break;
        }
        let d = |v: &Vertex| {
            let c = if use_x { v.x } else { v.y };
            if below { limit - c } else { c - limit }
        };
        let input = std::mem::take(&mut out);
        for i in 0..input.len() {
            let (p, q) = (&input[i], &input[(i + 1) % input.len()]);
            let (dp, dq) = (d(p), d(q));
            if dp >= 0.0 {
                out.push(*p);
            }
            if (dp >= 0.0) != (dq >= 0.0) {
                out.push(p.lerp(q, dp / (dp - dq)));
            }
        }
    }
    if out.len() < 3 { Vec::new() } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, r: f32) -> Vertex {
        Vertex { x, y, r, oow: 1.0, ..Default::default() }
    }

    #[test]
    fn clockwise_on_screen_is_positive() {
        // Top left, top right, bottom left: clockwise with y down.
        let (a, b, c) = (v(0.0, 0.0, 0.0), v(10.0, 0.0, 0.0), v(0.0, 10.0, 0.0));
        assert_eq!(area(&a, &b, &c), 100.0);
        assert_eq!(area(&a, &c, &b), -100.0);
        assert!(culled(CULL_NEGATIVE, -100.0) && !culled(CULL_NEGATIVE, 100.0));
        assert!(culled(CULL_POSITIVE, 100.0) && !culled(CULL_POSITIVE, -100.0));
        assert!(!culled(CULL_DISABLE, -1.0) && culled(CULL_DISABLE, 0.0));
    }

    #[test]
    fn fans() {
        assert_eq!(fan(5).collect::<Vec<_>>(), vec![[0, 1, 2], [0, 2, 3], [0, 3, 4]]);
        assert_eq!(fan(2).count(), 0);
        assert_eq!(fan(0).count(), 0);
    }

    #[test]
    fn clip_inside_is_unchanged() {
        let tri = [v(1.0, 1.0, 0.0), v(9.0, 1.0, 0.0), v(1.0, 9.0, 0.0)];
        assert_eq!(clip(&tri, [0.0, 0.0, 10.0, 10.0]), tri.to_vec());
    }

    #[test]
    fn clip_outside_is_empty() {
        let tri = [v(20.0, 1.0, 0.0), v(29.0, 1.0, 0.0), v(21.0, 9.0, 0.0)];
        assert!(clip(&tri, [0.0, 0.0, 10.0, 10.0]).is_empty());
    }

    #[test]
    fn clip_interpolates_every_parameter() {
        // A triangle that crosses x = 10: the cut vertices sit on the edge with the colour
        // interpolated along it.
        let tri = [v(0.0, 0.0, 0.0), v(20.0, 0.0, 200.0), v(0.0, 20.0, 0.0)];
        let out = clip(&tri, [0.0, 0.0, 10.0, 100.0]);
        assert_eq!(out.len(), 4);
        assert!(out.iter().all(|p| p.x <= 10.0));
        let cut: Vec<_> = out.iter().filter(|p| p.x == 10.0).collect();
        assert_eq!(cut.len(), 2);
        assert!(cut.iter().any(|p| p.y == 0.0 && p.r == 100.0));
        assert!(cut.iter().any(|p| p.y == 10.0 && p.r == 100.0));
        // The clipped area is the triangle less the corner beyond x = 10.
        let total: f32 = fan(out.len()).map(|[i, j, k]| area(&out[i], &out[j], &out[k])).sum();
        assert_eq!(total.abs(), 400.0 - 100.0);
    }
}
