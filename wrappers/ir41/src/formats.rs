//! Which pictures the codec takes and gives (`ICM_DECOMPRESS_QUERY`), and the decoded YVU9
//! planes written in each output format. Plain Rust, tested on the host.

use indeo::Frame;

/// The fields of a `BITMAPINFOHEADER` the codec looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub width: i32,
    /// Positive for a bottom-up picture, negative for top-down.
    pub height: i32,
    pub bit_count: u16,
    pub compression: u32,
    /// For `BI_BITFIELDS`, the red, green and blue masks after the header.
    pub masks: [u32; 3],
}

pub const BI_RGB: u32 = 0;
pub const BI_BITFIELDS: u32 = 3;

pub const fn fourcc(s: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*s)
}

/// What the codec writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Rgb32,
    Rgb24,
    Rgb555,
    Rgb565,
    /// Intel's `IF09` and plain `YVU9`: the planes as decoded.
    Yvu9,
}

impl Header {
    /// Read a header from its bytes (the 40 of `BITMAPINFOHEADER`, and the masks after).
    pub fn parse(b: &[u8]) -> Option<Self> {
        let u32_at = |o: usize| b.get(o..o + 4).map(|v| u32::from_le_bytes(v.try_into().unwrap()));
        let compression = u32_at(16)?;
        Some(Self {
            width: u32_at(4)? as i32,
            height: u32_at(8)? as i32,
            bit_count: u16::from_le_bytes(b.get(14..16)?.try_into().unwrap()),
            compression,
            masks: if compression == BI_BITFIELDS {
                [u32_at(40).unwrap_or(0), u32_at(44).unwrap_or(0), u32_at(48).unwrap_or(0)]
            } else {
                [0; 3]
            },
        })
    }

    pub fn is_indeo41(&self) -> bool {
        matches!(self.compression, c if c == fourcc(b"IV41") || c == fourcc(b"iv41"))
    }
}

/// The output for a picture of `input`'s size, or `None` when the codec does not give it.
/// Deviation: Intel's codec also gives 8-bit palette pictures; this one declines them, so
/// Video for Windows asks for another format.
pub fn output(input: &Header, out: &Header) -> Option<Output> {
    if !input.is_indeo41() || out.width != input.width || out.height.unsigned_abs() != input.height.unsigned_abs() {
        return None;
    }
    match (out.compression, out.bit_count) {
        (BI_RGB, 32) => Some(Output::Rgb32),
        (BI_RGB, 24) => Some(Output::Rgb24),
        (BI_RGB, 16) => Some(Output::Rgb555),
        (BI_BITFIELDS, 16) if out.masks == [0x7C00, 0x03E0, 0x001F] => Some(Output::Rgb555),
        (BI_BITFIELDS, 16) if out.masks == [0xF800, 0x07E0, 0x001F] => Some(Output::Rgb565),
        (c, 9) if c == fourcc(b"IF09") || c == fourcc(b"YVU9") => Some(Output::Yvu9),
        _ => None,
    }
}

impl Output {
    /// The bytes of one row (DIB rows are padded to 4 bytes).
    pub fn stride(self, width: usize) -> usize {
        match self {
            Output::Rgb32 => 4 * width,
            Output::Rgb24 => (3 * width + 3) & !3,
            Output::Rgb555 | Output::Rgb565 => (2 * width + 3) & !3,
            Output::Yvu9 => width,
        }
    }

    /// The bytes of a whole picture.
    pub fn size(self, width: usize, height: usize) -> usize {
        match self {
            Output::Yvu9 => width * height + 2 * width.div_ceil(4) * height.div_ceil(4),
            _ => self.stride(width) * height,
        }
    }
}

/// Write `frame` into `dst` as `out`, the rows bottom-up when `bottom_up` (a positive
/// `biHeight`). Deviation: RGB comes from the BT.601 equations, not from Intel's own colour
/// conversion; the YVU9 planes are Intel's exactly.
pub fn write(frame: &Frame, out: Output, bottom_up: bool, dst: &mut [u8]) {
    let (w, h) = (frame.width, frame.height);
    if out == Output::Yvu9 {
        let src = frame.yvu9();
        let n = src.len().min(dst.len());
        dst[..n].copy_from_slice(&src[..n]);
        return;
    }
    let rgba = frame.rgba();
    let stride = out.stride(w);
    for y in 0..h {
        let row = if bottom_up { h - 1 - y } else { y };
        let Some(line) = dst.get_mut(row * stride..row * stride + stride) else { continue };
        for x in 0..w {
            let p = &rgba[4 * (y * w + x)..4 * (y * w + x) + 3];
            let (r, g, b) = (p[0] as u32, p[1] as u32, p[2] as u32);
            match out {
                Output::Rgb32 => line[4 * x..4 * x + 4].copy_from_slice(&[b as u8, g as u8, r as u8, 0]),
                Output::Rgb24 => line[3 * x..3 * x + 3].copy_from_slice(&[b as u8, g as u8, r as u8]),
                Output::Rgb555 => {
                    let v = (r >> 3) << 10 | (g >> 3) << 5 | b >> 3;
                    line[2 * x..2 * x + 2].copy_from_slice(&(v as u16).to_le_bytes());
                }
                Output::Rgb565 => {
                    let v = (r >> 3) << 11 | (g >> 2) << 5 | b >> 3;
                    line[2 * x..2 * x + 2].copy_from_slice(&(v as u16).to_le_bytes());
                }
                Output::Yvu9 => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(width: i32, height: i32, bit_count: u16, compression: u32) -> Header {
        Header { width, height, bit_count, compression, masks: [0; 3] }
    }

    #[test]
    fn the_codec_takes_iv41_and_gives_rgb_and_yvu9() {
        let iv41 = header(320, 240, 24, fourcc(b"IV41"));
        assert_eq!(output(&iv41, &header(320, 240, 24, BI_RGB)), Some(Output::Rgb24));
        assert_eq!(output(&iv41, &header(320, -240, 32, BI_RGB)), Some(Output::Rgb32));
        assert_eq!(output(&iv41, &header(320, 240, 9, fourcc(b"IF09"))), Some(Output::Yvu9));
        assert_eq!(output(&iv41, &header(320, 240, 8, BI_RGB)), None);
        assert_eq!(output(&iv41, &header(160, 240, 24, BI_RGB)), None);
        assert_eq!(output(&header(320, 240, 24, fourcc(b"IV50")), &header(320, 240, 24, BI_RGB)), None);
        let mut h565 = header(320, 240, 16, BI_BITFIELDS);
        h565.masks = [0xF800, 0x07E0, 0x001F];
        assert_eq!(output(&iv41, &h565), Some(Output::Rgb565));
    }

    #[test]
    fn rows_go_bottom_up_with_padding() {
        let f = Frame { width: 3, height: 2, y: vec![16, 16, 16, 235, 235, 235], v: vec![128], u: vec![128] };
        let mut dst = vec![0xAA; Output::Rgb24.size(3, 2)];
        write(&f, Output::Rgb24, true, &mut dst);
        // Row 0 of the picture (black) is the last row of a bottom-up DIB; each row is 12 bytes.
        assert_eq!(&dst[12..21], &[0; 9]);
        assert_eq!(&dst[..9], &[255; 9]);
    }
}
