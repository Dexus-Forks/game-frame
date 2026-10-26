//! One triangle of a view: clipped at the near plane in the camera's space, projected, and
//! walked over the pixel centres it covers (`x + 0.5`, `y + 0.5`; top-left edges in, so two
//! triangles sharing an edge never both draw a pixel). Every attribute is interpolated with
//! the perspective (as `a / z` and `1 / z`), as the PCX2 did for its texture coordinates.
//! Deviation: the colours are interpolated the same way; how the chip's shading iterated
//! them is not taken from the driver.

use super::Proj;
use crate::scene::graph::Texture;
use crate::scene::walk::Vert;
use crate::scene::Colour;

/// What the triangle carries per vertex: u, v, the colour the texel is multiplied by (and its
/// alpha), and the colour added after.
const N: usize = 9;

fn attrs(v: &Vert) -> [f32; N] {
    let (m, a) = (v.modulate, v.add);
    [v.uv[0], v.uv[1], m[0], m[1], m[2], m[3], a[0], a[1], a[2]]
}

/// Cut a polygon at `z = near`, keeping the far side.
fn clip_near(poly: &[([f32; 3], [f32; N])], near: f32) -> Vec<([f32; 3], [f32; N])> {
    let mut out = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let (a, b) = (&poly[i], &poly[(i + 1) % poly.len()]);
        let (ai, bi) = (a.0[2] >= near, b.0[2] >= near);
        if ai {
            out.push(*a);
        }
        if ai != bi {
            let f = (near - a.0[2]) / (b.0[2] - a.0[2]);
            let mut p = [0.0; 3];
            let mut t = [0.0; N];
            for k in 0..3 {
                p[k] = a.0[k] + (b.0[k] - a.0[k]) * f;
            }
            for k in 0..N {
                t[k] = a.1[k] + (b.1[k] - a.1[k]) * f;
            }
            p[2] = near;
            out.push((p, t));
        }
    }
    out
}

/// A texel, wrapped (SGL repeats a map), point sampled; alpha 0–1.
/// Deviation: point sampled at the texel the coordinate falls in; the PCX2's filtering (the
/// driver's texture quality settings, which POD never changes) is not modelled.
fn texel(t: &Texture, u: f32, v: f32) -> [f32; 4] {
    let x = ((u - u.floor()) * t.w as f32) as usize % t.w.max(1) as usize;
    let y = ((v - v.floor()) * t.h as f32) as usize % t.h.max(1) as usize;
    let p = &t.rgba[(y * t.w as usize + x) * 4..];
    [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0, p[3] as f32 / 255.0]
}

/// Draw one triangle. `fog`: the fog colour and the share of a surface's colour kept per
/// unit of depth. A translucent triangle blends by its alpha over what is drawn and leaves the
/// depth; an opaque one keeps a pixel only where it is nearer than what is there.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    frame: &mut super::Frame,
    acc: &mut [Colour],
    p: &Proj,
    t: [Vert; 3],
    tex: Option<&Texture>,
    fog: Option<(Colour, f32)>,
    translucent: bool,
    no_depth: bool,
) {
    if t.iter().all(|v| v.p[2] < p.near) || t.iter().all(|v| v.p[2] > p.far) {
        return;
    }
    let poly: Vec<_> = t.iter().map(|v| (v.p, attrs(v))).collect();
    let poly = clip_near(&poly, p.near);
    if poly.len() < 3 {
        return;
    }
    // Screen x, y, 1/z and the attributes over z.
    let s: Vec<([f32; 3], [f32; N])> = poly
        .iter()
        .map(|(q, a)| {
            let iz = 1.0 / q[2];
            ([p.cx + p.kx * q[0] * iz, p.cy - p.ky * q[1] * iz, iz], a.map(|v| v * iz))
        })
        .collect();
    for i in 1..s.len() - 1 {
        span_tri(frame, acc, p, [&s[0], &s[i], &s[i + 1]], tex, fog, translucent, no_depth);
    }
}

#[allow(clippy::too_many_arguments)]
fn span_tri(
    frame: &mut super::Frame,
    acc: &mut [Colour],
    p: &Proj,
    v: [&([f32; 3], [f32; N]); 3],
    tex: Option<&Texture>,
    fog: Option<(Colour, f32)>,
    translucent: bool,
    no_depth: bool,
) {
    let (a, b, c) = (v[0].0, v[1].0, v[2].0);
    let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if area.abs() < 1e-9 {
        return;
    }
    // Both windings are drawn: SGL meshes are not culled unless the game asks.
    let sign = area.signum();
    let edge = |p0: [f32; 3], p1: [f32; 3], x: f32, y: f32| ((p1[0] - p0[0]) * (y - p0[1]) - (p1[1] - p0[1]) * (x - p0[0])) * sign;
    // Top-left rule: an edge whose function is exactly 0 counts only when it is a top or left edge.
    let top_left = |p0: [f32; 3], p1: [f32; 3]| {
        let (dx, dy) = ((p1[0] - p0[0]) * sign, (p1[1] - p0[1]) * sign);
        dy < 0.0 || (dy == 0.0 && dx > 0.0)
    };
    let edges = [(b, c, top_left(b, c)), (c, a, top_left(c, a)), (a, b, top_left(a, b))];
    let minx = a[0].min(b[0]).min(c[0]).floor().max(p.clip[0] as f32) as usize;
    let maxx = (a[0].max(b[0]).max(c[0]).ceil() as isize).clamp(0, p.clip[2] as isize) as usize;
    let miny = a[1].min(b[1]).min(c[1]).floor().max(p.clip[1] as f32) as usize;
    let maxy = (a[1].max(b[1]).max(c[1]).ceil() as isize).clamp(0, p.clip[3] as isize) as usize;
    let w = p.clip[2] - p.clip[0];
    let inv = 1.0 / area.abs();
    for y in miny..maxy {
        let fy = y as f32 + 0.5;
        for x in minx..maxx {
            let fx = x as f32 + 0.5;
            let mut l = [0.0f32; 3];
            let mut inside = true;
            for (k, (e0, e1, tl)) in edges.iter().enumerate() {
                let e = edge(*e0, *e1, fx, fy);
                if e < 0.0 || (e == 0.0 && !tl) {
                    inside = false;
                    break;
                }
                l[k] = e * inv;
            }
            if !inside {
                continue;
            }
            let iz = l[0] * a[2] + l[1] * b[2] + l[2] * c[2];
            let z = 1.0 / iz;
            if z > p.far {
                continue;
            }
            let di = y * frame.width + x;
            // Deviation: a tie in depth keeps the surface drawn first.
            if !no_depth && iz <= frame.depth[di] {
                continue;
            }
            let at = |k: usize| (l[0] * v[0].1[k] + l[1] * v[1].1[k] + l[2] * v[2].1[k]) * z;
            let t = match tex {
                Some(t) => texel(t, at(0), at(1)),
                None => [1.0; 4],
            };
            // A translucent map's clear texels are not drawn (POD's maps are clear or opaque,
            // `FUN_004997d0`). Deviation: cut at half; the chip blended a map's alpha.
            if t[3] < 0.5 {
                continue;
            }
            let mut col = [0, 1, 2].map(|k| (t[k] * at(2 + k) + at(6 + k)).min(1.0));
            if let Some((fc, keep)) = fog {
                let f = keep.powf(z);
                col = [0, 1, 2].map(|k| fc[k] + (col[k] - fc[k]) * f);
            }
            let o = &mut acc[(y - p.clip[1]) * w + x - p.clip[0]];
            if translucent {
                let alpha = at(5).clamp(0.0, 1.0);
                *o = [0, 1, 2].map(|k| col[k] * alpha + o[k] * (1.0 - alpha));
            } else {
                *o = col;
                if !no_depth {
                    frame.depth[di] = iz;
                }
            }
        }
    }
}
