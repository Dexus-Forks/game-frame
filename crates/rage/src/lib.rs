#![doc = include_str!("../README.md")]
#![doc = ""]
//! ATI 3D Rage, ATI3DCIF: texture formats (`C3D_ETEXFMT`), drawing-surface formats
//! (`C3D_EPIXFMT`) and render-state ids (`C3D_ERSID`), from ATI's *3D RAGE Windows 95
//! Programmer's Guide* (SDK-C02700 rev. 1.30, 1997).

use gameframe_texel::TexelFormat;

/// `C3D_ETF_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum TexFormat {
    Ci4 = 1,
    Ci8 = 2,
    Rgb1555 = 3,
    Rgb565 = 4,
    Rgb8888 = 6,
    Rgb332 = 7,
    Y8 = 8,
    Yuv422 = 11,
    Rgb4444 = 15,
    /// Vector quantised, RAGE PRO only.
    Vq = 20,
}

impl TexFormat {
    pub fn from_u32(v: u32) -> Option<TexFormat> {
        use TexFormat::*;
        [Ci4, Ci8, Rgb1555, Rgb565, Rgb8888, Rgb332, Y8, Yuv422, Rgb4444, Vq].into_iter().find(|f| *f as u32 == v)
    }

    /// The layout, when this crate decodes it (not YUV 4:2:2 or VQ).
    pub fn texel(self) -> Option<TexelFormat> {
        use TexFormat::*;
        Some(match self {
            Ci4 => TexelFormat::Palette4,
            Ci8 => TexelFormat::Palette8,
            Rgb1555 => TexelFormat::Argb1555,
            Rgb565 => TexelFormat::Rgb565,
            Rgb8888 => TexelFormat::Argb8888,
            Rgb332 => TexelFormat::Rgb332,
            Y8 => TexelFormat::Intensity8,
            Rgb4444 => TexelFormat::Argb4444,
            Yuv422 | Vq => return None,
        })
    }
}

/// `C3D_EPF_*`: drawing-surface formats.
pub const PIXFMT: [(u32, &str); 6] =
    [(3, "RGB1555"), (4, "RGB565"), (5, "RGB8888"), (6, "RGB332"), (7, "Y8"), (8, "YUV422")];

/// `C3D_ERSID`: the second argument of `ATI3DCIF_ContextSetState`, by value.
pub const RENDER_STATES: [&str; 36] = [
    "FG_CLR", "VERTEX_TYPE", "PRIM_TYPE", "SOLID_CLR", "SHADE_MODE", "TMAP_EN", "TMAP_SELECT",
    "TMAP_LIGHT", "TMAP_PERSP_COR", "TMAP_FILTER", "TMAP_TEXOP", "ALPHA_SRC", "ALPHA_DST",
    "SURF_DRAW_PTR", "SURF_DRAW_PITCH", "SURF_DRAW_PF", "SURF_VPORT", "FOG_EN", "DITHER_EN",
    "Z_CMP_FCN", "Z_MODE", "SURF_Z_PTR", "SURF_Z_PITCH", "SURF_SCISSOR", "COMPOSITE_EN",
    "COMPOSITE_SELECT", "COMPOSITE_FNC", "COMPOSITE_FACTOR", "COMPOSITE_FILTER",
    "COMPOSITE_FACTOR_ALPHA", "LOD_BIAS_LEVEL", "ALPHA_DST_TEST_ENABLE", "ALPHA_DST_TEST_FNC",
    "ALPHA_DST_WRITE_SELECT", "ALPHA_DST_REFERENCE", "SPECULAR_EN",
];

/// The `C3D_ERS_*` name of a render-state id.
pub fn render_state_name(id: u32) -> Option<&'static str> {
    RENDER_STATES.get(id as usize).copied()
}

/// `C3D_EPRIM_*`: line, triangle, quad, rectangle, point.
pub const PRIM_TYPES: [&str; 5] = ["LINE", "TRI", "QUAD", "RECT", "POINT"];

/// The texel layouts ATI3DCIF takes (`C3D_ETEXFMT`, less YUV and VQ).
pub const TEXTURE_FORMATS: &[TexelFormat] = {
    use TexelFormat::*;
    &[Palette4, Palette8, Argb1555, Rgb565, Argb8888, Rgb332, Intensity8, Argb4444]
};
