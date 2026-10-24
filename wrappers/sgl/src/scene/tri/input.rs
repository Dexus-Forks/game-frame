//! What the immediate mode reads from the game: its `SGLCONTEXT` and `SGLVERTEX` structures
//! (sgl.h, packed) and the flags of `u32Flags`. Plain Rust, tested on the host.

use crate::scene::Colour;

pub const GOURAUD: u32 = 0x1;
pub const TEXTURE: u32 = 0x2;
pub const HIGHLIGHT: u32 = 0x4;
pub const DECAL: u32 = 0x8;
pub const GLOBALTRANS: u32 = 0x20;
pub const WRAPU: u32 = 0x40;
pub const WRAPV: u32 = 0x80;
pub const VERTEXTRANS: u32 = 0x200;
pub const FACESIND3DFORMAT: u32 = 0x800;
pub const DISABLEZBUFFER: u32 = 0x2000;
pub const AVERAGECOLOUR: u32 = 0x4000;
/// Never read by SGLMID4.dll (*observed*), nor here.
pub const VERTEXFOG: u32 = 0x8000;
pub const OPAQUE: u32 = 0x40000;

/// `sizeof(SGLCONTEXT)` and `sizeof(SGLVERTEX)` (sgl.h, packed).
pub const CONTEXT_SIZE: usize = 0x90;
pub const VERTEX_SIZE: usize = 32;

fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn float(b: &[u8], at: usize) -> f32 {
    f32::from_bits(word(b, at))
}

/// The fields of `SGLCONTEXT` this library reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub fog_on: bool,
    pub fog: Colour,
    pub fog_density: u32,
    pub cull: bool,
    pub flags: u32,
    pub texture: i32,
    pub background: Colour,
    /// `eShadowLightVolMode`: 0 none, 1 shadows, 2 light volumes.
    pub shadow_mode: u32,
    /// The union at +0x38: `fShadowBrightness` (a float) in mode 1, `u32LightVolColour` (a
    /// `D3DCOLOR`) in mode 2. Kept as its bits; `scene::shadow::volume_colour` reads it.
    pub shadow_value: u32,
    pub flip: (bool, bool),
    pub uv_times_inv_w: bool,
    pub global_trans: u32,
}

impl Context {
    /// From the game's `SGLCONTEXT`, at least `CONTEXT_SIZE` bytes.
    pub fn read(b: &[u8]) -> Context {
        Context {
            fog_on: word(b, 0x00) != 0,
            fog: [float(b, 0x04), float(b, 0x08), float(b, 0x0c)],
            fog_density: word(b, 0x10),
            cull: word(b, 0x14) != 0,
            flags: word(b, 0x18),
            texture: word(b, 0x1c) as i32,
            background: [float(b, 0x28), float(b, 0x2c), float(b, 0x30)],
            shadow_mode: word(b, 0x34),
            shadow_value: word(b, 0x38),
            flip: (word(b, 0x3c) != 0, word(b, 0x40) != 0),
            uv_times_inv_w: word(b, 0x44) != 0,
            global_trans: word(b, 0x60),
        }
    }

    /// The flags as the triangle routines see them: a decal drops the highlight and both kinds
    /// of translucency.
    pub fn effective_flags(&self) -> u32 {
        if self.flags & (TEXTURE | DECAL) == TEXTURE | DECAL { self.flags & 0xfffffddb } else { self.flags }
    }
}

/// `SGLVERTEX`: a Direct3D `D3DTLVERTEX`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub inv_w: f32,
    /// `D3DCOLOR`: 0xAARRGGBB.
    pub colour: u32,
    pub specular: u32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn read(b: &[u8]) -> Vertex {
        Vertex {
            x: float(b, 0),
            y: float(b, 4),
            z: float(b, 8),
            inv_w: float(b, 12),
            colour: word(b, 16),
            specular: word(b, 20),
            u: float(b, 24),
            v: float(b, 28),
        }
    }
}

/// A `D3DCOLOR` as red, green, blue and alpha from 0 to 1.
pub fn rgba(c: u32) -> [f32; 4] {
    let f = |s: u32| ((c >> s) & 255) as f32 / 255.0;
    [f(16), f(8), f(0), f(24)]
}
