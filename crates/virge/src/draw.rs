//! The chip's drawing as the S3D Toolkit drives it: one `S3DTK_TriangleSet` call drawn into
//! the draw surface, with the state `S3DTK_SetState` set.
//!
//! Vertices are `S3DTK_VERTEX_TEX`, 32 bytes (*measured* in POD's S3 build: X, Y, Z, W floats,
//! then B, G, R, A bytes, then D, U, V floats; the shadow fans are black with A = 128). The draw
//! surface is an `S3DTK_SURFACE` (`sfOffset`, `sfWidth`, `sfHeight`, `sfFormat`); the ViRGE's
//! 3D engine draws 8-, 15- or 24-bit pixels, so a 16-bit game surface is 5-5-5.
//! Texturing, fog and the Z buffer are not modelled: a textured call is drawn Gouraud and
//! reported as a [`Gap`].

use crate::{ALPHASOURCE, GOURAUD, VIDEORGB8, VIDEORGB15};
use gameframe_raster::{Format, Target, Vertex, polygon, triangle};

/// `S3DTK_TRILIST`, `S3DTK_TRISTRIP`, `S3DTK_TRIFAN`.
pub const TRILIST: u32 = 0;
pub const TRISTRIP: u32 = 1;
pub const TRIFAN: u32 = 2;

/// `S3DTK_STATEKEY` values the drawing reads.
pub mod key {
    pub const DRAWSURFACE: usize = 4;
    pub const RENDERINGTYPE: usize = 10;
    pub const ZBUFFERENABLE: usize = 13;
    pub const TEXTUREACTIVE: usize = 15;
    pub const ALPHABLENDING: usize = 19;
    pub const FOGCOLOR: usize = 20;
}

/// `S3DTK_VERTEX_TEX`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
    pub d: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex32 {
    /// From the record's 32 bytes, little-endian.
    pub fn from_bytes(b: &[u8; 32]) -> Vertex32 {
        let f = |o: usize| f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        Vertex32 { x: f(0), y: f(4), z: f(8), w: f(12), b: b[16], g: b[17], r: b[18], a: b[19], d: f(20), u: f(24), v: f(28) }
    }
}

/// The drawing surface's pixel format, from `sfFormat`: `None` for 24 bits.
pub fn surface_format(sf_format: u32) -> Option<Format> {
    match sf_format & 0xc {
        VIDEORGB15 => Some(Format::Rgb555),
        VIDEORGB8 => Some(Format::Index8),
        _ => None,
    }
}

/// The renderer's state: each `S3DTK_SetState` key's last value. `FOGCOLOR` starts at
/// `0xffffffff` (fog off).
#[derive(Clone, Debug)]
pub struct State {
    pub values: [u32; 24],
}

impl Default for State {
    fn default() -> State {
        let mut values = [0; 24];
        values[key::FOGCOLOR] = 0xffff_ffff;
        State { values }
    }
}

impl State {
    /// `S3DTK_SetState`. Keys past the table are ignored.
    pub fn set(&mut self, key: u32, value: u32) {
        if let Some(v) = self.values.get_mut(key as usize) {
            *v = value;
        }
    }
}

/// What a call asked for that the model does not draw (it draws the rest).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub what: &'static str,
    pub value: u32,
}

/// `S3DTK_TriangleSet`: `verts` as a list, strip or fan (`kind`) into `target`.
pub fn triangle_set(s: &State, target: &mut Target, verts: &[Vertex32], kind: u32) -> Vec<Gap> {
    let mut gaps = Vec::new();
    if s.values[key::RENDERINGTYPE] != GOURAUD {
        gaps.push(Gap { what: "S3DTK_RENDERINGTYPE: textures are not drawn, Gouraud instead", value: s.values[key::RENDERINGTYPE] });
    }
    if s.values[key::FOGCOLOR] != 0xffff_ffff {
        gaps.push(Gap { what: "S3DTK_FOGCOLOR: fog is not drawn", value: s.values[key::FOGCOLOR] });
    }
    let format = target.format;
    let verts: Vec<Vertex> = verts
        .iter()
        .map(|v| Vertex { x: v.x, y: v.y, a: [v.r as f32, v.g as f32, v.b as f32, v.a as f32, 0.0, 0.0, 0.0, 0.0] })
        .collect();
    let blend = s.values[key::ALPHABLENDING] == ALPHASOURCE;
    let mut shade = |at: &[f32; 8], old: u32| {
        let src = [at[0], at[1], at[2]];
        if !blend || format == Format::Index8 {
            return Some(format.pack(src));
        }
        // Deviation: the chip's blending precision is *unknown*; floats, cut to the format.
        let a = (at[3] / 255.0).clamp(0.0, 1.0);
        let dst = format.rgb(old);
        Some(format.pack([0, 1, 2].map(|i| src[i] * a + dst[i] * (1.0 - a))))
    };
    match kind {
        TRILIST => verts.chunks_exact(3).for_each(|c| triangle(target, [c[0], c[1], c[2]], &mut shade)),
        TRISTRIP => verts.windows(3).for_each(|c| triangle(target, [c[0], c[1], c[2]], &mut shade)),
        _ => polygon(target, &verts, &mut shade),
    }
    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shadow_fan_halves_what_is_there() {
        // POD's shadow: black, A = 128, source alpha, a fan, on a 5-5-5 surface.
        let mut s = State::default();
        s.set(key::ALPHABLENDING as u32, ALPHASOURCE);
        let mut buf = [0xffu8; 4 * 4 * 2];
        let mut t = Target::new(&mut buf, 8, 4, 4, Format::Rgb555);
        let v = |x, y| Vertex32 { x, y, a: 128, ..Default::default() };
        let gaps = triangle_set(&s, &mut t, &[v(0.0, 0.0), v(4.0, 0.0), v(4.0, 4.0), v(0.0, 4.0)], TRIFAN);
        assert!(gaps.is_empty());
        // 255 × (1 − 128/255) = 127 → 15 of 31.
        assert_eq!(t.get(1, 1), 15 << 10 | 15 << 5 | 15);
    }

    #[test]
    fn the_record_reads_in_order() {
        let mut b = [0u8; 32];
        b[0..4].copy_from_slice(&3.5f32.to_le_bytes());
        b[16..20].copy_from_slice(&[1, 2, 3, 4]);
        let v = Vertex32::from_bytes(&b);
        assert_eq!((v.x, v.b, v.g, v.r, v.a), (3.5, 1, 2, 3, 4));
    }
}
