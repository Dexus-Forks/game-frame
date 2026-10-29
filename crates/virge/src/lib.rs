#![doc = include_str!("../README.md")]
#![doc = ""]
//! S3 ViRGE, S3D Toolkit: texture and video formats, rendering types and state keys, from
//! `S3DTK.H` of the S3D ToolKit SDK 2.6, and the chip's drawing as the toolkit drives it
//! ([`draw`]).

pub mod draw;

pub use draw::{State, Vertex32, triangle_set};
pub use gameframe_raster::{Format, Target};
use gameframe_texel::TexelFormat;

/// Surface format bits (`sfFormat`) for textures.
pub const TEXARGB8888: u32 = 0x00;
pub const TEXARGB4444: u32 = 0x20;
pub const TEXARGB1555: u32 = 0x40;
pub const TEXPALETTIZED8: u32 = 0xc0;

/// Surface format bits for drawing surfaces.
pub const VIDEORGB8: u32 = 0x0;
pub const VIDEORGB15: u32 = 0x4;
pub const VIDEORGB24: u32 = 0x8;

/// Surface flags.
pub const SYSTEM: u32 = 0x8000_0000;
pub const TEXTURE: u32 = 0x4000_0000;
pub const Z16: u32 = 0x1000_0000;

/// Values of the `S3DTK_RENDERINGTYPE` state.
pub const GOURAUD: u32 = 0x0;
pub const LITTEXTURE: u32 = 0x0800_0000;
pub const UNLITTEXTURE: u32 = 0x1000_0000;
pub const LITTEXTUREPERSPECT: u32 = 0x2800_0000;
pub const UNLITTEXTUREPERSPECT: u32 = 0x3000_0000;

/// Values of the `S3DTK_ALPHABLENDING` state.
pub const ALPHAOFF: u32 = 0x0;
pub const ALPHATEXTURE: u32 = 0x8_0000;
pub const ALPHASOURCE: u32 = 0xc_0000;

/// `S3DTK_STATEKEY`: the second argument of `S3DTK_SetState`, by value.
pub const STATE_KEYS: [&str; 24] = [
    "VERSION", "VIDEOMODE", "VIDEOMEMORYADDRESS", "VIDEOMEMORYSIZE", "DRAWSURFACE",
    "DISPLAYSURFACE", "DISPLAYADDRESSUPDATED", "GRAPHICS_ENGINE_IDLE", "DMA_OFF", "DMA_ON",
    "RENDERINGTYPE", "ZBUFFERSURFACE", "ZBUFFERCOMPAREMODE", "ZBUFFERENABLE",
    "ZBUFFERUPDATEENABLE", "TEXTUREACTIVE", "TEXFILTERINGMODE", "TEXBLENDINGMODE",
    "TEXMAXMIPMAPLEVEL", "ALPHABLENDING", "FOGCOLOR", "D_LEVEL_SUPPLIED", "CLIPPING_AREA",
    "FLIP_WAIT",
];

/// `S3DTK_FUNCTIONLIST`: the slots of the table `S3DTK_CreateRenderer` fills, in order.
pub const FUNCTIONS: [&str; 8] = [
    "S3DTK_SetState", "S3DTK_GetState", "S3DTK_TriangleSet", "S3DTK_TriangleSetEx",
    "S3DTK_BitBlt", "S3DTK_BitBltTransparent", "S3DTK_RectFill", "S3DTK_GetLastError",
];

/// The texel layout of a texture surface's `sfFormat`, from its texture-format bits.
pub fn texel(sf_format: u32) -> Option<TexelFormat> {
    Some(match sf_format & 0xe0 {
        TEXARGB8888 => TexelFormat::Argb8888,
        TEXARGB4444 => TexelFormat::Argb4444,
        TEXARGB1555 => TexelFormat::Argb1555,
        TEXPALETTIZED8 => TexelFormat::Palette8,
        _ => return None,
    })
}

/// The `S3DTK_*` name of a state key.
pub fn state_name(key: u32) -> Option<&'static str> {
    STATE_KEYS.get(key as usize).copied()
}

/// The texel layouts the S3D Toolkit takes.
pub const TEXTURE_FORMATS: &[TexelFormat] = {
    use TexelFormat::*;
    &[Argb8888, Argb4444, Argb1555, Palette8]
};
