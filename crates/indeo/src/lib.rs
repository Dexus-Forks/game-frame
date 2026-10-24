//! Indeo Video Interactive 4.1 (`IV41`), decoded as Intel's own codec `IR41_32.DLL`
//! (R4.11.15.91) decodes it on its Pentium path, picture for picture. Clean room: every rule
//! was read from Intel's code or measured by running it under GameFrame's emulator, in the POD
//! project (its `docs/formats/video.md`, "The pictures"); no other decoder's source was read.
//! Intel's tables are not in this crate: `Tables::from_dll` reads them from the player's own
//! copy of the DLL.

mod bits;
mod blocks;
mod header;
mod huffman;
mod mb;
mod mc;
mod output;
mod picture;
mod swar;
mod tables;

pub use picture::Decoder;
pub use tables::Tables;

/// Why a picture or the tables could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Intel's `IR41_32.DLL` did not give the tables.
    IndeoTables(String),
    /// A picture this decoder cannot read.
    Picture(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IndeoTables(s) => write!(f, "Indeo tables: {s}"),
            Error::Picture(s) => write!(f, "Indeo picture: {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// A decoded picture in Intel's `IF09` layout (YVU9): the Y plane, then V, then U, each chroma
/// plane a quarter of the width and height (rounded up).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub v: Vec<u8>,
    pub u: Vec<u8>,
}

impl Frame {
    pub fn chroma_size(&self) -> (usize, usize) {
        (self.width.div_ceil(4), self.height.div_ceil(4))
    }

    /// The planes one after another, as `IF09` stores them.
    pub fn yvu9(&self) -> Vec<u8> {
        [&self.y[..], &self.v[..], &self.u[..]].concat()
    }

    /// RGBA pixels, by the BT.601 equations with each chroma sample covering 4 × 4 pixels.
    pub fn rgba(&self) -> Vec<u8> {
        let (cw, _) = self.chroma_size();
        let mut out = Vec::with_capacity(self.width * self.height * 4);
        for y in 0..self.height {
            for x in 0..self.width {
                let l = self.y[y * self.width + x] as f32 - 16.0;
                let c = (y / 4) * cw + x / 4;
                let (v, u) = (self.v[c] as f32 - 128.0, self.u[c] as f32 - 128.0);
                let px = |f: f32| (f.round().clamp(0.0, 255.0)) as u8;
                out.extend([
                    px(1.164 * l + 1.596 * v),
                    px(1.164 * l - 0.392 * u - 0.813 * v),
                    px(1.164 * l + 2.017 * u),
                    255,
                ]);
            }
        }
        out
    }
}

#[doc(hidden)]
pub mod testing {
    //! The internals the tests check.
    pub fn transform(addr: u32, m: &mut [u32]) -> bool {
        match addr {
            0x10009B60 => super::swar::slant_8x8(m),
            0x10009ED0 => super::swar::slant_4x4(m),
            0x10015310 => super::swar::haar_8x8(m),
            0x10015230 => super::swar::haar_1x8(m),
            0x10015130 => super::swar::haar_8x1(m),
            0x100155D0 => super::swar::none_8x8(m),
            _ => return false,
        }
        true
    }
}
