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

impl Format {
    pub fn bytes(self) -> usize {
        if self == Format::Index8 { 1 } else { 2 }
    }

    /// A pixel to 8-bit RGB (indices give themselves in red).
    pub fn rgb(self, v: u32) -> [f32; 3] {
        let c = match self {
            Format::Rgb565 => pixels::rgb565(v as u16),
            Format::Rgb555 => pixels::rgb555(v as u16),
            Format::Index8 => [v as u8, 0, 0],
        };
        c.map(|x| x as f32)
    }

    /// 8-bit RGB (0–255, clamped) to a pixel.
    pub fn pack(self, [r, g, b]: [f32; 3]) -> u32 {
        let c = |x: f32| x.clamp(0.0, 255.0) as u32;
        match self {
            Format::Rgb565 => (c(r) >> 3) << 11 | (c(g) >> 2) << 5 | c(b) >> 3,
            Format::Rgb555 => (c(r) >> 3) << 10 | (c(g) >> 3) << 5 | c(b) >> 3,
            Format::Index8 => c(r),
        }
    }
}

/// A frame buffer the rasteriser writes into: memory the caller owns.
pub struct Target {
    pub ptr: *mut u8,
    /// Bytes per row.
    pub pitch: usize,
    pub w: usize,
    pub h: usize,
    pub format: Format,
}

impl Target {
    pub fn get(&self, x: usize, y: usize) -> u32 {
        let o = y * self.pitch + x * self.format.bytes();
        unsafe {
            if self.format == Format::Index8 {
                self.ptr.add(o).read() as u32
            } else {
                (self.ptr.add(o) as *const u16).read_unaligned() as u32
            }
        }
    }

    pub fn set(&mut self, x: usize, y: usize, v: u32) {
        let o = y * self.pitch + x * self.format.bytes();
        unsafe {
            if self.format == Format::Index8 {
                self.ptr.add(o).write(v as u8)
            } else {
                (self.ptr.add(o) as *mut u16).write_unaligned(v as u16)
            }
        }
    }
}
