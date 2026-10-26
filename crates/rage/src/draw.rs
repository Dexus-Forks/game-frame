//! The chip's drawing as ATI3DCIF drives it: the render state `ATI3DCIF_ContextSetState`
//! sets, and one `ATI3DCIF_RenderPrimList` call drawn into the surface.
//!
//! Vertices are `C3D_VTCF` (x, y, z, s, t, w, r, g, b, a floats; *measured* in POD's ATI
//! build, whose shadow quads have r = g = b = 0 and a = 80). Colours are 0–255. The surface is
//! the game's DirectDraw back buffer (`SURF_DRAW_PTR`), `SURF_DRAW_PITCH` pixels to a row, in
//! RGB565 when `SURF_DRAW_PF` is not set (*inferred*: POD never sets it and its surface is 565).
//! Texturing, fog, dithering, the Z buffer and the scissor are not modelled.

use gameframe_raster::{Format, Target, Vertex, polygon};

/// `C3D_ERSID` values the drawing reads.
pub mod state {
    pub const PRIM_TYPE: u32 = 2;
    pub const SOLID_CLR: u32 = 3;
    pub const SHADE_MODE: u32 = 4;
    pub const ALPHA_SRC: u32 = 11;
    pub const ALPHA_DST: u32 = 12;
    pub const SURF_DRAW_PTR: u32 = 13;
    pub const SURF_DRAW_PITCH: u32 = 14;
    pub const SURF_DRAW_PF: u32 = 15;
}

/// `C3D_EPRIM_*` the drawing takes.
pub const PRIM_TRI: u32 = 1;
pub const PRIM_QUAD: u32 = 2;
/// `C3D_ESH_*`.
pub const SHADE_SOLID: u32 = 1;
pub const SHADE_FLAT: u32 = 2;
pub const SHADE_SMOOTH: u32 = 3;
/// `C3D_EASRC_*` / `C3D_EADST_*` blend factors.
pub const ALPHA_ZERO: u32 = 0;
pub const ALPHA_ONE: u32 = 1;
pub const ALPHA_SRC_ALPHA: u32 = 4;
pub const ALPHA_INV_SRC_ALPHA: u32 = 5;

/// `C3D_VTCF`: a vertex with texture coordinates and a floating colour.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vtcf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32,
    pub t: f32,
    pub w: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Vtcf {
    /// From the ten floats of the record, in order.
    pub fn from_words(w: [f32; 10]) -> Vtcf {
        let [x, y, z, s, t, ww, r, g, b, a] = w;
        Vtcf { x, y, z, s, t, w: ww, r, g, b, a }
    }
}

/// A context's render state: every state's last value (the word `C3D_PRSDATA` points at), by
/// `C3D_ERSID`.
#[derive(Clone, Debug)]
pub struct Context {
    pub values: [u32; 40],
}

impl Default for Context {
    fn default() -> Context {
        let mut values = [0; 40];
        values[state::PRIM_TYPE as usize] = PRIM_TRI;
        values[state::SHADE_MODE as usize] = SHADE_SMOOTH;
        values[state::ALPHA_SRC as usize] = ALPHA_ONE;
        values[state::ALPHA_DST as usize] = ALPHA_ZERO;
        Context { values }
    }
}

impl Context {
    pub fn get(&self, id: u32) -> u32 {
        self.values.get(id as usize).copied().unwrap_or(0)
    }

    /// `ATI3DCIF_ContextSetState`. Ids past the table are ignored.
    pub fn set(&mut self, id: u32, value: u32) {
        if let Some(v) = self.values.get_mut(id as usize) {
            *v = value;
        }
    }

    /// The surface's pixel format: `C3D_EPF_RGB1555` (3) or RGB565 (4, and when not set).
    pub fn format(&self) -> Format {
        match self.get(state::SURF_DRAW_PF) {
            3 => Format::Rgb555,
            _ => Format::Rgb565,
        }
    }
}

/// A blend factor (`C3D_EASRC_*` for the source, `C3D_EADST_*` for the destination: 0 zero,
/// 1 one, 2 the other colour, 3 one minus it, 4 source alpha, 5 one minus it, 6 destination
/// alpha, 7 one minus it). Deviation: the surface has no alpha channel; destination alpha
/// reads as 1.
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

/// What a call asked for that the model does not draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub what: &'static str,
    pub value: u32,
}

/// `ATI3DCIF_RenderPrimList`: `verts` as triangles or quads (`PRIM_TYPE`) into `target`.
pub fn render_prim_list(c: &Context, target: &mut Target, verts: &[Vtcf]) -> Result<(), Gap> {
    // Colours are clamped to 0–255 at each vertex before they are interpolated. Deviation:
    // POD gives its cars' light cones values up to 256 and alphas up to 717 (*measured*); how
    // the Rage took values above 255 is *unknown* (docs: the S3 build, whose vertices hold
    // bytes, draws the same cones as fading beams, so saturation at set-up is *inferred*).
    let col = |v: f32| v.clamp(0.0, 255.0);
    let verts: Vec<Vertex> = verts
        .iter()
        .map(|v| Vertex { x: v.x, y: v.y, a: [col(v.r), col(v.g), col(v.b), col(v.a), 0.0, 0.0, 0.0, 0.0] })
        .collect();
    let format = target.format;
    let shade_mode = c.get(state::SHADE_MODE);
    let solid = c.get(state::SOLID_CLR);
    let (src_f, dst_f) = (c.get(state::ALPHA_SRC), c.get(state::ALPHA_DST));
    let per = match c.get(state::PRIM_TYPE) {
        PRIM_TRI => 3,
        PRIM_QUAD => 4,
        p => return Err(Gap { what: "C3D_EPRIM: primitive type not drawn", value: p }),
    };
    for prim in verts.chunks_exact(per) {
        // Deviation: a flat primitive takes its first vertex's colour; which vertex the chip
        // takes is *unknown*.
        let flat = prim[0].a;
        let mut shade = |at: &[f32; 8], old: u32| {
            let col = match shade_mode {
                // C3D_COLOR: r, g, b, a bytes.
                SHADE_SOLID => [solid & 0xff, solid >> 8 & 0xff, solid >> 16 & 0xff, solid >> 24].map(|x| x as f32),
                SHADE_FLAT => [flat[0], flat[1], flat[2], flat[3]],
                _ => [at[0], at[1], at[2], at[3]],
            };
            let src = [col[0], col[1], col[2]];
            let a = (col[3] / 255.0).clamp(0.0, 1.0);
            if (src_f, dst_f) == (ALPHA_ONE, ALPHA_ZERO) {
                return Some(format.pack(src));
            }
            let dst = format.rgb(old);
            let (fs, fd) = (factor(src_f, dst, a), factor(dst_f, src, a));
            // Deviation: the chip's blending precision is *unknown*; floats, cut to the format.
            Some(format.pack([0, 1, 2].map(|i| src[i] * fs[i] + dst[i] * fd[i])))
        };
        polygon(target, prim, &mut shade);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(a: f32) -> Vec<Vtcf> {
        let v = |x, y| Vtcf { x, y, a, ..Default::default() };
        vec![v(0.0, 0.0), v(4.0, 0.0), v(4.0, 4.0), v(0.0, 4.0)]
    }

    #[test]
    fn a_shadow_quad_darkens_what_is_there() {
        // POD's shadow: black, alpha 80, SRC_ALPHA / INV_SRC_ALPHA, as a quad.
        let mut c = Context::default();
        c.set(state::PRIM_TYPE, PRIM_QUAD);
        c.set(state::ALPHA_SRC, ALPHA_SRC_ALPHA);
        c.set(state::ALPHA_DST, ALPHA_INV_SRC_ALPHA);
        let mut buf = [0xffu8; 4 * 4 * 2];
        let mut t = Target::new(&mut buf, 8, 4, 4, Format::Rgb565);
        render_prim_list(&c, &mut t, &quad(80.0)).unwrap();
        // White times (1 - 80/255): 175 → 21 of 31 red, 43 of 63 green.
        assert_eq!(t.get(1, 1), 21 << 11 | 43 << 5 | 21);
    }

    #[test]
    fn opaque_draws_the_colour() {
        let mut c = Context::default();
        c.set(state::PRIM_TYPE, PRIM_QUAD);
        let mut buf = [0u8; 4 * 4 * 2];
        let mut t = Target::new(&mut buf, 8, 4, 4, Format::Rgb565);
        let mut q = quad(255.0);
        q.iter_mut().for_each(|v| v.r = 255.0);
        render_prim_list(&c, &mut t, &q).unwrap();
        assert_eq!(t.get(2, 2), 31 << 11);
    }

    #[test]
    fn an_unknown_primitive_is_a_gap() {
        let mut c = Context::default();
        c.set(state::PRIM_TYPE, 4);
        let mut buf = [0u8; 8];
        let mut t = Target::new(&mut buf, 4, 2, 2, Format::Rgb565);
        assert!(render_prim_list(&c, &mut t, &quad(0.0)).is_err());
    }
}
