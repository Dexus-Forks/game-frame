//! Frame buffers and their pixel formats.

/// The pixel layouts of the drawing surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 16 bits, `RRRRRGGGGGGBBBBB`.
    Rgb565,
    /// 16 bits, `xRRRRRGGGGGBBBBB`.
    Rgb555,
    /// 8 bits, a palette index (no blending).
    Index8,
}

fn expand5(v: u32) -> u8 {
    ((v << 3) | (v >> 2)) as u8
}

fn expand6(v: u32) -> u8 {
    ((v << 2) | (v >> 4)) as u8
}

impl Format {
    pub fn bytes(self) -> usize {
        if self == Format::Index8 { 1 } else { 2 }
    }

    /// A pixel to 8-bit RGB, each field widened by repeating its top bits (indices give
    /// themselves in red).
    pub fn rgb(self, v: u32) -> [f32; 3] {
        let c = match self {
            Format::Rgb565 => [expand5(v >> 11 & 31), expand6(v >> 5 & 63), expand5(v & 31)],
            Format::Rgb555 => [expand5(v >> 10 & 31), expand5(v >> 5 & 31), expand5(v & 31)],
            Format::Index8 => [v as u8, 0, 0],
        };
        c.map(|x| x as f32)
    }

    /// 8-bit RGB (0–255, clamped) to a pixel, each field cut to its top bits.
    pub fn pack(self, [r, g, b]: [f32; 3]) -> u32 {
        let c = |x: f32| x.clamp(0.0, 255.0) as u32;
        match self {
            Format::Rgb565 => (c(r) >> 3) << 11 | (c(g) >> 2) << 5 | c(b) >> 3,
            Format::Rgb555 => (c(r) >> 3) << 10 | (c(g) >> 3) << 5 | c(b) >> 3,
            Format::Index8 => c(r),
        }
    }
}

/// A frame buffer the rasteriser writes into: little-endian pixels in memory the caller owns.
pub struct Target<'a> {
    pub data: &'a mut [u8],
    /// Bytes per row.
    pub pitch: usize,
    pub w: usize,
    pub h: usize,
    pub format: Format,
}

impl<'a> Target<'a> {
    /// `h` is cut to the rows `data` holds.
    pub fn new(data: &'a mut [u8], pitch: usize, w: usize, h: usize, format: Format) -> Target<'a> {
        let rows = (data.len() + pitch).saturating_sub(w * format.bytes()).checked_div(pitch).unwrap_or(0);
        Target { data, pitch, w, h: h.min(rows), format }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        let o = y * self.pitch + x * self.format.bytes();
        match self.format {
            Format::Index8 => self.data[o] as u32,
            _ => u16::from_le_bytes([self.data[o], self.data[o + 1]]) as u32,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, v: u32) {
        let o = y * self.pitch + x * self.format.bytes();
        match self.format {
            Format::Index8 => self.data[o] = v as u8,
            _ => self.data[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes()),
        }
    }
}
