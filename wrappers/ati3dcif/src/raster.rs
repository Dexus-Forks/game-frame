//! Drawing one `ATI3DCIF_RenderPrimList` call into the game's surface.
//!
//! Vertices are `C3D_VTCF` (*measured*: the game's records dump as x, y, z, s, t, w, r, g, b,
//! a floats; the shadow quads have r = g = b = 0 and a = 80). Colours are 0–255. The surface
//! is the DirectDraw back buffer (`SURF_DRAW_PTR`), `SURF_DRAW_PITCH` pixels to a row, 16-bit
//! RGB565: the game never sets `SURF_DRAW_PF`, and its surface is 565 (*inferred*).

use crate::state::*;
use raster::{Format, Target, Vertex, polygon};
use shim::log_once;

/// `C3D_EPRIM_*`.
const TRI: u32 = 1;
const QUAD: u32 = 2;
/// `C3D_ESH_*`.
const SOLID: u32 = 1;
const FLAT: u32 = 2;

/// A blend factor (`C3D_EASRC_*` for the source, `C3D_EADST_*` for the destination: 0 zero,
/// 1 one, 2 the other colour, 3 one minus it, 4 source alpha, 5 one minus it, 6 destination
/// alpha, 7 one minus it). The destination has no alpha channel: taken as 1.
fn factor(f: u32, other: [f32; 3], src_a: f32) -> [f32; 3] {
    let k = |v: f32| [v; 3];
    match f {
        0 => k(0.0),
        1 => k(1.0),
        2 => other.map(|c| c / 255.0),
        3 => other.map(|c| 1.0 - c / 255.0),
        4 => k(src_a),
        5 => k(1.0 - src_a),
        6 => k(1.0),
        _ => k(0.0),
    }
}

pub unsafe fn prim(c: &Context, vs: &[*const u32]) {
    let ptr = c.get(SURF_DRAW_PTR) as *mut u8;
    let pitch = c.get(SURF_DRAW_PITCH) as usize;
    if ptr.is_null() || pitch == 0 {
        return;
    }
    let mut t = Target { ptr, pitch: pitch * 2, w: pitch, h: pitch * 3 / 4, format: Format::Rgb565 };
    let f = |v: *const u32, i: usize| f32::from_bits(unsafe { v.add(i).read_unaligned() });
    // Colours are clamped to 0–255 at each vertex before they are interpolated: the game gives
    // the cars' light cones values up to 256 and alphas up to 717 (*measured*), and the S3
    // build, whose vertices hold bytes, draws the same cones as fading beams (*inferred*: the
    // Rage saturated vertex colours at set-up).
    let col = |v: *const u32, i: usize| f(v, i).clamp(0.0, 255.0);
    let verts: Vec<Vertex> = vs
        .iter()
        .map(|&v| Vertex { x: f(v, 0), y: f(v, 1), a: [col(v, 6), col(v, 7), col(v, 8), col(v, 9), 0.0, 0.0, 0.0, 0.0] })
        .collect();
    let shade_mode = c.get(SHADE_MODE);
    let solid = c.get(SOLID_CLR);
    let flat = verts.first().map(|v| v.a).unwrap_or_default();
    let (src_f, dst_f) = (c.get(ALPHA_SRC), c.get(ALPHA_DST));
    let mut shade = |at: &[f32; 8], old: u32| {
        let col = match shade_mode {
            // C3D_COLOR: r, g, b, a bytes.
            SOLID => [solid & 0xff, solid >> 8 & 0xff, solid >> 16 & 0xff, solid >> 24].map(|x| x as f32),
            FLAT => [flat[0], flat[1], flat[2], flat[3]],
            _ => [at[0], at[1], at[2], at[3]],
        };
        let src = [col[0], col[1], col[2]];
        let a = (col[3] / 255.0).clamp(0.0, 1.0);
        if (src_f, dst_f) == (1, 0) {
            return Some(Format::Rgb565.pack(src));
        }
        let dst = Format::Rgb565.rgb(old);
        let (fs, fd) = (factor(src_f, dst, a), factor(dst_f, src, a));
        Some(Format::Rgb565.pack([0, 1, 2].map(|i| src[i] * fs[i] + dst[i] * fd[i])))
    };
    let per = match c.get(PRIM_TYPE) {
        TRI => 3,
        QUAD => 4,
        p => {
            log_once!("primitive type {p} is not drawn");
            return;
        }
    };
    for chunk in verts.chunks_exact(per) {
        polygon(&mut t, chunk, &mut shade);
    }
}
