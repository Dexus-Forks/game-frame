//! `grDrawTriangle`: a triangle of `GrVertex` records through the pixel pipeline into a
//! [`Frame`].
//!
//! Per pixel, in the order of the SST-1 specification's pipeline (sections 5.16 to 5.19): the
//! texel (s/w and t/w over 1/w), the chroma key on the colour unit's `other` input, the
//! colour and alpha combine units, the alpha test, fog, the depth test, alpha blending with
//! the pixel already there, the dither into 565, then the colour and depth writes.
//!
//! *Observed* (section 5.2 to 5.14): vertex x and y are 12.4 fixed point, so they are rounded
//! to 1/16 pixel here; the colours are iterated as 12.12 and the combine units take their
//! integer part, clamped to 0–255 (*inferred*); Z is 20.12 and the depth buffer takes its
//! integer part, clamped; 1/W is iterated and turned into the W buffer's floating value
//! ([`crate::frame::wfloat`]). All are interpolated linearly in screen space.
//!
//! Rules: a pixel is drawn when its centre (`x + 0.5`, `y + 0.5`) is inside the triangle, with
//! top-left edges in and bottom-right edges out. Deviation: the chip's own edge walk and its
//! iterators' rounding are *unknown*; the iterators are evaluated in floats at each pixel
//! centre, then cut to the chip's formats.

use crate::combine::{Inputs, argb, combine};
use crate::consts::*;
use crate::frame::{Frame, from_565, to_565, wfloat};
use crate::state::{State, compare};
use crate::texture::Texture;

/// `GrVertex` (Glide 2.x): screen x, y in pixels; colours 0–255; `ooz` 0–65535; `oow` = 1/w;
/// `sow`, `tow` = s/w, t/w for the one TMU modelled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub ooz: f32,
    pub oow: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub sow: f32,
    pub tow: f32,
}

const ATTRS: usize = 8;

fn attrs(v: &Vertex) -> [f32; ATTRS] {
    [v.r, v.g, v.b, v.a, v.ooz, v.oow, v.sow, v.tow]
}

/// One alpha blending factor (section 5.18.2), 0–1. Deviation: the chip's blending precision
/// is *unknown*; floats here, rounded to 8 bits.
fn blend_factor(f: u32, src: [f32; 4], dst: [f32; 3], is_dst: bool) -> [f32; 3] {
    // Deviation: no alpha buffer is modelled; the destination's alpha reads as 1.
    let (sa, da) = (src[3], 1.0);
    match f {
        BLEND_SRC_ALPHA => [sa; 3],
        BLEND_SRC_COLOR if is_dst => [src[0], src[1], src[2]],
        BLEND_SRC_COLOR => dst,
        BLEND_DST_ALPHA => [da; 3],
        BLEND_ONE => [1.0; 3],
        BLEND_ONE_MINUS_SRC_ALPHA => [1.0 - sa; 3],
        BLEND_ONE_MINUS_SRC_COLOR if is_dst => [1.0 - src[0], 1.0 - src[1], 1.0 - src[2]],
        BLEND_ONE_MINUS_SRC_COLOR => dst.map(|d| 1.0 - d),
        BLEND_ONE_MINUS_DST_ALPHA => [1.0 - da; 3],
        _ => [0.0; 3],
    }
}

fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Draw one triangle. Either winding is drawn (`grCullMode(GR_CULL_DISABLE)`, the default).
pub fn triangle(frame: &mut Frame, state: &State, texture: Option<&Texture>, v: [&Vertex; 3]) {
    let snap = |c: f32| (c * 16.0).round() / 16.0;
    let p = v.map(|v| (snap(v.x), snap(v.y)));
    let area = edge(p[0], p[1], p[2]);
    if !area.is_finite() || area == 0.0 {
        return;
    }
    // Wind so the area is positive; the edges then face inwards.
    let (p, v) = if area < 0.0 { ([p[0], p[2], p[1]], [v[0], v[2], v[1]]) } else { (p, v) };
    let area = area.abs();
    let at = v.map(attrs);
    let clip = state.clip;
    let lo = |f: fn(&(f32, f32)) -> f32| p.iter().map(f).fold(f32::INFINITY, f32::min).floor() as i32;
    let hi = |f: fn(&(f32, f32)) -> f32| p.iter().map(f).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    let x0 = lo(|q| q.0).max(clip[0]).max(0);
    let y0 = lo(|q| q.1).max(clip[1]).max(0);
    let x1 = hi(|q| q.0).min(clip[2]).min(frame.width as i32);
    let y1 = hi(|q| q.1).min(clip[3]).min(frame.height as i32);
    // Top-left rule: with y down and positive area, an edge is a top or left edge when it
    // runs right to left along the top or downwards on the left.
    let owns = |a: (f32, f32), b: (f32, f32)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        (dy == 0.0 && dx < 0.0) || dy > 0.0
    };
    let edges = [(1, 2), (2, 0), (0, 1)];
    let own = edges.map(|(i, j)| owns(p[i], p[j]));
    let constant = argb(state.constant);
    let fog_colour = argb(state.fog_colour);
    let key = argb(state.key);
    let w = frame.width;
    for y in y0..y1 {
        for x in x0..x1 {
            let c = (x as f32 + 0.5, y as f32 + 0.5);
            let mut l = [0.0f32; 3];
            let mut inside = true;
            for (k, &(i, j)) in edges.iter().enumerate() {
                let e = edge(p[i], p[j], c);
                if e < 0.0 || (e == 0.0 && !own[k]) {
                    inside = false;
                    break;
                }
                l[k] = e / area;
            }
            if !inside {
                continue;
            }
            let a: [f32; ATTRS] = std::array::from_fn(|k| l[0] * at[0][k] + l[1] * at[1][k] + l[2] * at[2][k]);
            let (ooz, oow) = (a[4], a[5]);
            let iterated = [a[0], a[1], a[2], a[3]].map(|c| c.floor().clamp(0.0, 255.0) as u8);
            let texel = match state.texture {
                TEXTURECOMBINE_ZERO => [0; 4],
                TEXTURECOMBINE_ONE => [255; 4],
                _ => match texture {
                    Some(t) if oow != 0.0 => t.sample(a[6] / oow, a[7] / oow, state.filter, state.clamp),
                    _ => [255; 4],
                },
            };
            let inputs = Inputs {
                iterated,
                texel,
                constant,
            };
            let (out, other) = combine(&state.colour, &state.alpha, &inputs);
            if state.chroma && other == [key[0], key[1], key[2]] {
                continue;
            }
            if !compare(state.alpha_test.0, out[3], state.alpha_test.1) {
                continue;
            }
            let wf = wfloat(oow);
            let depth = match state.depth_mode {
                DEPTHBUFFER_WBUFFER => wf,
                _ => ooz.floor().clamp(0.0, 65535.0) as u16,
            };
            let i = y as usize * w + x as usize;
            if state.depth_mode != DEPTHBUFFER_DISABLE && !compare(state.depth_function, depth, frame.depth[i]) {
                continue;
            }
            let fog = match state.fog_mode {
                FOG_WITH_TABLE => Some(crate::fog::table_alpha(&state.fog_table, wf)),
                FOG_WITH_ITERATED_ALPHA => Some(iterated[3]),
                _ => None,
            };
            let mut rgb = [out[0], out[1], out[2]];
            if let Some(f) = fog {
                for k in 0..3 {
                    rgb[k] = crate::fog::apply(rgb[k], fog_colour[k], f);
                }
            }
            let colour = if state.blend[0] == BLEND_ONE && state.blend[1] == BLEND_ZERO {
                rgb
            } else {
                let d8 = from_565(frame.colour[i]);
                let dst = d8.map(|c| c as f32 / 255.0);
                let src = [rgb[0], rgb[1], rgb[2], out[3]].map(|c| c as f32 / 255.0);
                let sf = blend_factor(state.blend[0], src, dst, false);
                let df = blend_factor(state.blend[1], src, dst, true);
                std::array::from_fn(|k| ((src[k] * sf[k] + dst[k] * df[k]) * 255.0).round().clamp(0.0, 255.0) as u8)
            };
            if state.colour_mask {
                frame.colour[i] = to_565(colour, x as usize, y as usize, state.dither);
            }
            if state.depth_mask && state.depth_mode != DEPTHBUFFER_DISABLE {
                frame.depth[i] = depth;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::state::Combine;

    fn v(x: f32, y: f32, oow: f32) -> Vertex {
        Vertex {
            x,
            y,
            oow,
            r: 255.0,
            g: 255.0,
            b: 255.0,
            a: 255.0,
            ..Vertex::default()
        }
    }

    fn count(f: &Frame) -> usize {
        f.colour.iter().filter(|&&p| p != 0).count()
    }

    #[test]
    fn two_triangles_sharing_an_edge_draw_each_pixel_once() {
        let mut f = Frame::new(8, 8);
        let mut s = State::default();
        s.dither = false;
        s.blend = [BLEND_ONE, BLEND_ONE, BLEND_ZERO, BLEND_ZERO];
        s.colour = Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, false);
        let (a, b, c, d) = (v(0.0, 0.0, 1.0), v(8.0, 0.0, 1.0), v(8.0, 8.0, 1.0), v(0.0, 8.0, 1.0));
        let grey = |mut q: Vertex| {
            q.r = 64.0;
            q.g = 64.0;
            q.b = 64.0;
            q
        };
        let (a, b, c, d) = (grey(a), grey(b), grey(c), grey(d));
        triangle(&mut f, &s, None, [&a, &b, &c]);
        triangle(&mut f, &s, None, [&a, &c, &d]);
        // Added twice anywhere, a pixel would be brighter than 64.
        assert_eq!(count(&f), 64);
        assert!(f.colour.iter().all(|&p| p == f.colour[0]));
    }

    #[test]
    fn the_w_buffer_keeps_the_nearer_face() {
        let mut f = Frame::new(4, 4);
        let mut s = State::default();
        s.dither = false;
        s.depth_mode = DEPTHBUFFER_WBUFFER;
        s.depth_mask = true;
        f.clear(0, 0xffff);
        let near = |mut q: Vertex| {
            q.r = 255.0;
            q.g = 0.0;
            q.b = 0.0;
            q.oow = 0.5;
            q
        };
        let quad = [v(0.0, 0.0, 0.1), v(4.0, 0.0, 0.1), v(0.0, 4.0, 0.1)];
        let close = quad.map(near);
        triangle(&mut f, &s, None, [&close[0], &close[1], &close[2]]);
        triangle(&mut f, &s, None, [&quad[0], &quad[1], &quad[2]]);
        assert_eq!(f.colour[0], 0xf800);
        assert_eq!(f.depth[0], wfloat(0.5));
    }

    #[test]
    fn chroma_key_drops_black_texels() {
        let mut f = Frame::new(2, 1);
        let mut s = State::default();
        s.dither = false;
        s.chroma = true;
        s.key = 0;
        s.colour = Combine::colour_function(CC_DECAL_TEXTURE).unwrap();
        // One black texel, one white.
        let t = Texture {
            width: 2,
            height: 1,
            texels: vec![0, 0, 0, 255, 255, 255, 255, 255],
        };
        let mut a = v(0.0, -1.0, 1.0);
        let mut b = v(4.0, -1.0, 1.0);
        let mut c = v(0.0, 3.0, 1.0);
        (a.sow, b.sow, c.sow) = (0.0, 512.0, 0.0);
        triangle(&mut f, &s, Some(&t), [&a, &b, &c]);
        assert_eq!(f.colour, [0, 0xffff]);
    }

    #[test]
    fn source_alpha_blending() {
        let mut f = Frame::new(1, 1);
        let mut s = State::default();
        s.dither = false;
        f.clear(0x00ffffff, 0xffff);
        s.blend = [BLEND_SRC_ALPHA, BLEND_ONE_MINUS_SRC_ALPHA, BLEND_ZERO, BLEND_ZERO];
        let black = |mut q: Vertex| {
            q.r = 0.0;
            q.g = 0.0;
            q.b = 0.0;
            q.a = 127.5;
            q
        };
        let t = [v(-1.0, -1.0, 1.0), v(3.0, -1.0, 1.0), v(-1.0, 3.0, 1.0)].map(black);
        triangle(&mut f, &s, None, [&t[0], &t[1], &t[2]]);
        // Half white: 565 levels 16, 32, 16 (0.5 × 31 = 15.5 rounds up).
        assert_eq!(f.colour[0], 16 << 11 | 32 << 5 | 16);
    }

    #[test]
    fn texture_coordinates_are_perspective_correct() {
        // A strip whose right end is four times as far: s/w and 1/w are linear on screen, s
        // is not. At the screen's middle s is a fifth of the way, not half.
        let mut f = Frame::new(100, 1);
        let mut s = State::default();
        s.dither = false;
        s.colour = Combine::colour_function(CC_DECAL_TEXTURE).unwrap();
        let t = Texture {
            width: 256,
            height: 1,
            texels: (0..256).flat_map(|i| [i as u8, 0, 0, 255]).collect(),
        };
        let (near, far) = (1.0, 0.25);
        let mut a = v(0.0, -1.0, near);
        let mut b = v(100.0, -1.0, far);
        let mut c = v(0.0, 3.0, near);
        (a.sow, b.sow, c.sow) = (0.0, 256.0 * far, 0.0);
        triangle(&mut f, &s, Some(&t), [&a, &b, &c]);
        let red = |x: usize| from_565(f.colour[x])[0] as f32 / 255.0;
        assert!((red(50) - 0.2).abs() < 0.03, "{}", red(50));
    }
}
