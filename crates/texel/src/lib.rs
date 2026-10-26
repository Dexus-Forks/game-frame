#![doc = include_str!("../README.md")]
#![doc = ""]
//! Texel layouts and their decoding to 8-bit RGBA. Multi-byte texels are little-endian, and
//! bit fields are named from the top bit down (`ARGB1555`: alpha in bit 15).

pub mod resample;

/// A table that turns an 8-bit YIQ texel into RGB: a chip's own compression, such as 3dfx's
/// NCC (`gameframe_voodoo::ncc::NccTable`).
pub trait Yiq {
    fn decode(&self, texel: u8) -> [u8; 3];
}

/// A picture in 8-bit RGBA, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// Every texel layout the 1997 chips in this crate accept.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TexelFormat {
    /// 8 bits: `RRRGGGBB`.
    Rgb332,
    /// 8 bits: 3dfx NCC, `YYYYIIQQ` through a table.
    Yiq422,
    /// 8 bits: alpha only (colour white).
    Alpha8,
    /// 8 bits: intensity (grey, opaque).
    Intensity8,
    /// 8 bits: `AAAAIIII`.
    AlphaIntensity44,
    /// 8 bits: an index into a 256-entry palette.
    Palette8,
    /// 4 bits: an index into a 16-entry palette, low nibble first.
    Palette4,
    /// 16 bits: alpha byte over `RRRGGGBB`.
    Argb8332,
    /// 16 bits: alpha byte over an NCC byte.
    Ayiq8422,
    /// 16 bits: `RRRRRGGGGGGBBBBB`.
    Rgb565,
    /// 16 bits: `ARRRRRGGGGGBBBBB`.
    Argb1555,
    /// 16 bits: `AAAARRRRGGGGBBBB`.
    Argb4444,
    /// 16 bits: alpha byte over intensity byte.
    AlphaIntensity88,
    /// 16 bits: alpha byte over a palette index.
    AlphaPalette88,
    /// 32 bits: bytes B, G, R, A in memory (`0xAARRGGBB` as a little-endian word).
    Argb8888,
}

impl TexelFormat {
    /// Bits per texel.
    pub fn bits(self) -> usize {
        use TexelFormat::*;
        match self {
            Palette4 => 4,
            Rgb332 | Yiq422 | Alpha8 | Intensity8 | AlphaIntensity44 | Palette8 => 8,
            Argb8888 => 32,
            _ => 16,
        }
    }

    /// Bytes for `width × height` texels.
    pub fn bytes(self, width: usize, height: usize) -> usize {
        (width * height * self.bits()).div_ceil(8)
    }
}

/// What a decode may need besides the texels.
#[derive(Clone, Copy, Default)]
pub struct Tables<'a> {
    /// RGB triplets for the palettised formats (16 or 256 used).
    pub palette: Option<&'a [[u8; 3]]>,
    /// The table for the YIQ formats (3dfx's NCC).
    pub ncc: Option<&'a dyn Yiq>,
}

fn rep(v: u32, bits: u32) -> u8 {
    // Widen a `bits`-bit value to 8 bits by repeating its top bits.
    let mut out = 0u32;
    let mut filled = 0;
    while filled < 8 {
        out = (out << bits) | v;
        filled += bits;
    }
    (out >> (filled - 8)) as u8
}

fn rgb332(v: u8) -> [u8; 3] {
    [rep((v >> 5) as u32, 3), rep(((v >> 2) & 7) as u32, 3), rep((v & 3) as u32, 2)]
}

/// Decode `width × height` texels, rows top to bottom. Fails when the data is short or a
/// needed table is missing.
pub fn decode(format: TexelFormat, width: usize, height: usize, data: &[u8], t: Tables) -> Result<Rgba, String> {
    use TexelFormat::*;
    let n = width * height;
    if data.len() < format.bytes(width, height) {
        return Err(format!("{format:?} {width}x{height} needs {} bytes, got {}", format.bytes(width, height), data.len()));
    }
    let pal = |i: usize| -> Result<[u8; 3], String> {
        let p = t.palette.ok_or("palette format without a palette")?;
        Ok(p.get(i).copied().unwrap_or([255, 0, 255]))
    };
    let ncc = || t.ncc.ok_or_else(|| "NCC format without a table".to_string());
    let w16 = |i: usize| u16::from_le_bytes([data[2 * i], data[2 * i + 1]]);
    let mut px = Vec::with_capacity(n * 4);
    for i in 0..n {
        let rgba: [u8; 4] = match format {
            Rgb332 => { let [r, g, b] = rgb332(data[i]); [r, g, b, 255] }
            Yiq422 => { let [r, g, b] = ncc()?.decode(data[i]); [r, g, b, 255] }
            Alpha8 => [255, 255, 255, data[i]],
            Intensity8 => [data[i], data[i], data[i], 255],
            AlphaIntensity44 => { let i8 = rep((data[i] & 15) as u32, 4); [i8, i8, i8, rep((data[i] >> 4) as u32, 4)] }
            Palette8 => { let [r, g, b] = pal(data[i] as usize)?; [r, g, b, 255] }
            Palette4 => {
                let b = data[i / 2];
                let [r, g, bl] = pal(if i % 2 == 0 { b & 15 } else { b >> 4 } as usize)?;
                [r, g, bl, 255]
            }
            Argb8332 => { let v = w16(i); let [r, g, b] = rgb332(v as u8); [r, g, b, (v >> 8) as u8] }
            Ayiq8422 => { let v = w16(i); let [r, g, b] = ncc()?.decode(v as u8); [r, g, b, (v >> 8) as u8] }
            Rgb565 => { let v = w16(i) as u32; [rep(v >> 11, 5), rep((v >> 5) & 63, 6), rep(v & 31, 5), 255] }
            Argb1555 => { let v = w16(i) as u32; [rep((v >> 10) & 31, 5), rep((v >> 5) & 31, 5), rep(v & 31, 5), if v & 0x8000 != 0 { 255 } else { 0 }] }
            Argb4444 => {
                let v = w16(i) as u32;
                [rep((v >> 8) & 15, 4), rep((v >> 4) & 15, 4), rep(v & 15, 4), rep(v >> 12, 4)]
            }
            AlphaIntensity88 => { let v = w16(i); let l = v as u8; [l, l, l, (v >> 8) as u8] }
            AlphaPalette88 => { let v = w16(i); let [r, g, b] = pal((v & 0xff) as usize)?; [r, g, b, (v >> 8) as u8] }
            Argb8888 => [data[4 * i + 2], data[4 * i + 1], data[4 * i], data[4 * i + 3]],
        };
        px.extend_from_slice(&rgba);
    }
    Ok(Rgba { width, height, pixels: px })
}
