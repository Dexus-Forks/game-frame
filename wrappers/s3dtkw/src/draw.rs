//! Drawing one `S3DTK_TriangleSet` call into the game's surface.
//!
//! Vertices are `S3DTK_VERTEX_TEX`, 32 bytes (*measured*: the game's records dump as X, Y, Z,
//! W floats, then B, G, R, A bytes, then D, U, V floats; the shadow fans are black with A =
//! 128). The draw surface is an `S3DTK_SURFACE` (`sfOffset`, `sfWidth`, `sfHeight`,
//! `sfFormat`); POD gives `S3DTK_VIDEORGB15` (4), the only 16-bit format the ViRGE's 3D
//! engine draws, so the DirectDraw surfaces of this build are 5-5-5 (`DDRAW_PIXELS=555`).
//!
//! In a race on Beltane every call POD makes is Gouraud-shaded (`RENDERINGTYPE` 0, *measured*:
//! 15,000 calls, no texture surface ever made); textured faces go to the game's own software
//! renderer. Texturing is left out: a textured call is logged and drawn Gouraud.

use crate::table::State;
use gpu_1997::s3;
use raster::{Format, Target, Vertex, polygon, triangle};

/// `S3DTK_TRILIST`, `S3DTK_TRISTRIP`, `S3DTK_TRIFAN` (POD uses the fan).
const LIST: u32 = 0;
const STRIP: u32 = 1;

const DRAWSURFACE: usize = 4;
const RENDERINGTYPE: usize = 10;
const ALPHABLENDING: usize = 19;
const FOGCOLOR: usize = 20;

pub unsafe fn triangles(s: &State, list: *const u32, n: u32, kind: u32) {
    let surf = s.values[DRAWSURFACE] as *const u32;
    if surf.is_null() || list.is_null() {
        return;
    }
    let word = |i: usize| unsafe { surf.add(i).read_unaligned() };
    let (ptr, w, h, format) = (word(0) as *mut u8, word(1) as usize, word(2) as usize, word(3));
    let format = match format & 0xc {
        s3::VIDEORGB15 => Format::Rgb555,
        s3::VIDEORGB8 => Format::Index8,
        _ => {
            shim::gap!("surface format", "{format:#x} (24-bit) is not drawn");
            return;
        }
    };
    let mut t = Target { ptr, pitch: w * format.bytes(), w, h, format };
    if s.values[RENDERINGTYPE] != s3::GOURAUD {
        shim::gap!("S3DTK_RENDERINGTYPE textured", "type {:#x}: textures are not drawn, Gouraud instead", s.values[RENDERINGTYPE]);
    }
    if s.values[FOGCOLOR] != 0xffff_ffff {
        shim::gap!("S3DTK_FOGCOLOR", "fog colour {:#x}: fog is not drawn", s.values[FOGCOLOR]);
    }
    let verts: Vec<Vertex> = (0..n as usize)
        .map(|i| {
            let v = unsafe { list.add(i).read_unaligned() } as *const u8;
            let f = |o: usize| unsafe { (v.add(o) as *const f32).read_unaligned() };
            let b = |o: usize| unsafe { v.add(o).read() } as f32;
            Vertex { x: f(0), y: f(4), a: [b(0x12), b(0x11), b(0x10), b(0x13), 0.0, 0.0, 0.0, 0.0] }
        })
        .collect();
    let blend = s.values[ALPHABLENDING] == s3::ALPHASOURCE;
    let mut shade = |at: &[f32; 8], old: u32| {
        let src = [at[0], at[1], at[2]];
        if !blend || format == Format::Index8 {
            return Some(format.pack(src));
        }
        let a = (at[3] / 255.0).clamp(0.0, 1.0);
        let dst = format.rgb(old);
        Some(format.pack([0, 1, 2].map(|i| src[i] * a + dst[i] * (1.0 - a))))
    };
    match kind {
        LIST => verts.chunks_exact(3).for_each(|c| triangle(&mut t, [c[0], c[1], c[2]], &mut shade)),
        STRIP => verts.windows(3).for_each(|c| triangle(&mut t, [c[0], c[1], c[2]], &mut shade)),
        _ => polygon(&mut t, &verts, &mut shade),
    }
}
