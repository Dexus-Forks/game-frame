//! What each chip family's API accepts, for tools that show the same scene the way each card
//! would have (a "renderer profile" switch). Facts from the APIs' SDKs only: what a given game
//! did with each card belongs with that game's code.

use crate::texel::TexelFormat;

/// The chip families of this crate, plus software rendering for comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Card {
    /// No 3D chip: the program draws every pixel.
    Software,
    /// 3dfx Voodoo Graphics through Glide 2.x.
    Voodoo,
    /// ATI 3D Rage (I, II) through ATI3DCIF.
    Rage,
    /// S3 ViRGE through the S3D Toolkit.
    Virge,
}

pub const ALL: [Card; 4] = [Card::Software, Card::Voodoo, Card::Rage, Card::Virge];

impl Card {
    pub fn name(self) -> &'static str {
        match self {
            Card::Software => "Software",
            Card::Voodoo => "3dfx Voodoo (Glide)",
            Card::Rage => "ATI 3D Rage (ATI3DCIF)",
            Card::Virge => "S3 ViRGE (S3D Toolkit)",
        }
    }

    /// Texture layouts the API takes (empty for software: any layout the program likes).
    pub fn texture_formats(self) -> &'static [TexelFormat] {
        use TexelFormat::*;
        match self {
            Card::Software => &[],
            Card::Voodoo => &[Rgb332, Yiq422, Alpha8, Intensity8, AlphaIntensity44, Palette8, Argb8332, Ayiq8422, Rgb565, Argb1555, Argb4444, AlphaIntensity88, AlphaPalette88],
            Card::Rage => &[Palette4, Palette8, Argb1555, Rgb565, Argb8888, Rgb332, Intensity8, Argb4444],
            Card::Virge => &[Argb8888, Argb4444, Argb1555, Palette8],
        }
    }

    /// The largest texture side the API accepts: Glide's `GR_LOD_256`; the others are not
    /// limited to 256 by their APIs (`None`: see the chip's documentation).
    pub fn max_texture_side(self) -> Option<usize> {
        match self {
            Card::Voodoo => Some(256),
            _ => None,
        }
    }

    /// Whether the card takes this layout as it is.
    pub fn accepts(self, f: TexelFormat) -> bool {
        self == Card::Software || self.texture_formats().contains(&f)
    }
}
