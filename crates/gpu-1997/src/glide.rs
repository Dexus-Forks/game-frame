//! 3dfx Glide 2.x: texture formats (`GrTextureFormat_t`), mip levels (`GrLOD_t`) and aspect
//! ratios (`GrAspectRatio_t`), with the values of the Glide 2.x SDK's `glide.h`.

use crate::texel::TexelFormat;

/// `GR_TEXFMT_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Format {
    Rgb332 = 0x0,
    Yiq422 = 0x1,
    Alpha8 = 0x2,
    Intensity8 = 0x3,
    AlphaIntensity44 = 0x4,
    P8 = 0x5,
    Argb8332 = 0x8,
    Ayiq8422 = 0x9,
    Rgb565 = 0xa,
    Argb1555 = 0xb,
    Argb4444 = 0xc,
    AlphaIntensity88 = 0xd,
    Ap88 = 0xe,
}

impl Format {
    pub fn from_u32(v: u32) -> Option<Format> {
        use Format::*;
        [Rgb332, Yiq422, Alpha8, Intensity8, AlphaIntensity44, P8, Argb8332, Ayiq8422, Rgb565, Argb1555, Argb4444, AlphaIntensity88, Ap88]
            .into_iter()
            .find(|f| *f as u32 == v)
    }

    pub fn texel(self) -> TexelFormat {
        use Format::*;
        match self {
            Rgb332 => TexelFormat::Rgb332,
            Yiq422 => TexelFormat::Yiq422,
            Alpha8 => TexelFormat::Alpha8,
            Intensity8 => TexelFormat::Intensity8,
            AlphaIntensity44 => TexelFormat::AlphaIntensity44,
            P8 => TexelFormat::Palette8,
            Argb8332 => TexelFormat::Argb8332,
            Ayiq8422 => TexelFormat::Ayiq8422,
            Rgb565 => TexelFormat::Rgb565,
            Argb1555 => TexelFormat::Argb1555,
            Argb4444 => TexelFormat::Argb4444,
            AlphaIntensity88 => TexelFormat::AlphaIntensity88,
            Ap88 => TexelFormat::AlphaPalette88,
        }
    }
}

/// `GR_LOD_256` = 0 down to `GR_LOD_1` = 8: the longer side is `256 >> lod`.
pub fn lod_side(lod: u32) -> Option<usize> {
    (lod <= 8).then(|| 256 >> lod)
}

/// `GR_ASPECT_8x1` = 0 … `GR_ASPECT_1x1` = 3 … `GR_ASPECT_1x8` = 6 (width × height).
pub fn size(lod: u32, aspect: u32) -> Option<(usize, usize)> {
    let side = lod_side(lod)?;
    if aspect > 6 {
        return None;
    }
    let shrink = |s: usize, by: u32| (s >> by).max(1);
    Some(if aspect <= 3 { (side, shrink(side, 3 - aspect)) } else { (shrink(side, aspect - 3), side) })
}

/// Texel bytes of a mip chain from `large_lod` down to `small_lod` (Glide's `smallLod` is the
/// larger number). Glide's `grTexCalcMemRequired` may add alignment; this is the data size.
pub fn chain_bytes(small_lod: u32, large_lod: u32, aspect: u32, format: Format) -> Option<usize> {
    (large_lod..=small_lod).map(|l| size(l, aspect).map(|(w, h)| format.texel().bytes(w, h))).sum()
}

/// `GR_TEXTABLE_NCC0`, `GR_TEXTABLE_NCC1`, `GR_TEXTABLE_PALETTE` for `grTexDownloadTable`.
pub const TEXTABLE_NCC0: u32 = 0;
pub const TEXTABLE_NCC1: u32 = 1;
pub const TEXTABLE_PALETTE: u32 = 2;

/// A Glide palette entry is `0x00RRGGBB`; this makes one from an RGB triplet.
pub fn palette_entry([r, g, b]: [u8; 3]) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | b as u32
}
