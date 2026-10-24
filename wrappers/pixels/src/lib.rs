//! Pixel formats of 1990s PC games, converted to 8-bit RGB or RGBA.
//!
//! Nothing here is specific to one game. It came from the POD project with the wrappers.

/// A picture in 8-bit RGBA, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

fn expand5(v: u16) -> u8 {
    ((v << 3) | (v >> 2)) as u8
}

fn expand6(v: u16) -> u8 {
    ((v << 2) | (v >> 4)) as u8
}

/// 15-bit colour, `0RRRRRGGGGGBBBBB`.
pub fn rgb555(v: u16) -> [u8; 3] {
    [expand5((v >> 10) & 31), expand5((v >> 5) & 31), expand5(v & 31)]
}

/// 16-bit colour, `RRRRRGGGGGGBBBBB`.
pub fn rgb565(v: u16) -> [u8; 3] {
    [expand5(v >> 11), expand6((v >> 5) & 63), expand5(v & 31)]
}

/// Little-endian 16-bit pixels through a colour function.
pub fn from_u16(width: usize, height: usize, data: &[u8], colour: fn(u16) -> [u8; 3]) -> Rgba {
    let mut pixels = Vec::with_capacity(width * height * 4);
    for p in data.chunks_exact(2).take(width * height) {
        let [r, g, b] = colour(u16::from_le_bytes([p[0], p[1]]));
        pixels.extend_from_slice(&[r, g, b, 255]);
    }
    Rgba { width, height, pixels }
}

/// 8-bit indices through a palette of RGB triplets.
pub fn from_indexed(width: usize, height: usize, data: &[u8], palette: &[[u8; 3]]) -> Rgba {
    let mut pixels = Vec::with_capacity(width * height * 4);
    for &i in data.iter().take(width * height) {
        let [r, g, b] = palette.get(i as usize).copied().unwrap_or([255, 0, 255]);
        pixels.extend_from_slice(&[r, g, b, 255]);
    }
    Rgba { width, height, pixels }
}

/// A palette of 256 RGB triplets (8-bit values) from 768 bytes.
pub fn palette_rgb(bytes: &[u8]) -> Vec<[u8; 3]> {
    bytes.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

/// 32-bit pixels in the order B, G, R, A.
pub fn from_bgra(width: usize, height: usize, data: &[u8]) -> Rgba {
    let mut pixels = Vec::with_capacity(width * height * 4);
    for p in data.chunks_exact(4).take(width * height) {
        pixels.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
    }
    Rgba { width, height, pixels }
}

/// A Windows DIB (`BITMAPINFOHEADER`, palette, rows bottom-up padded to 4 bytes), 8 or 24
/// bits, uncompressed. Returns the picture and the number of bytes after the last row.
pub fn from_dib(blob: &[u8]) -> Result<(Rgba, usize), String> {
    let u32_at = |o: usize| blob.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let u16_at = |o: usize| blob.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let short = || "DIB too short".to_string();
    let (hsize, w, h) = (u32_at(0).ok_or_else(short)?, u32_at(4).ok_or_else(short)? as i32, u32_at(8).ok_or_else(short)? as i32);
    let (planes, bits, comp) = (u16_at(12).ok_or_else(short)?, u16_at(14).ok_or_else(short)?, u32_at(16).ok_or_else(short)?);
    if hsize != 40 || planes != 1 || comp != 0 || !(bits == 8 || bits == 24) {
        return Err(format!("unsupported DIB: header {hsize}, planes {planes}, {bits} bits, compression {comp}"));
    }
    let colours = match u32_at(32).ok_or_else(short)? {
        0 if bits == 8 => 256,
        n => n as usize,
    };
    let (width, height) = (w as usize, h.unsigned_abs() as usize);
    let pal_at = 40;
    let pixels_at = pal_at + colours * 4;
    let stride = (width * bits as usize / 8 + 3) & !3;
    let need = pixels_at + stride * height;
    if need > blob.len() {
        return Err(format!("DIB {width}x{height} needs {need} bytes, has {}", blob.len()));
    }
    let palette: Vec<[u8; 3]> = (0..colours).map(|i| {
        let c = &blob[pal_at + i * 4..pal_at + i * 4 + 3];
        [c[2], c[1], c[0]]
    }).collect();
    let mut pixels = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        let src = if h > 0 { height - 1 - y } else { y };
        let row = &blob[pixels_at + src * stride..];
        for x in 0..width {
            let [r, g, b] = if bits == 24 {
                [row[x * 3 + 2], row[x * 3 + 1], row[x * 3]]
            } else {
                palette[row[x] as usize]
            };
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    Ok((Rgba { width, height, pixels }, blob.len() - need))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(rgb555(0x7FFF), [255, 255, 255]);
        assert_eq!(rgb555(0x7C00), [255, 0, 0]);
        assert_eq!(rgb565(0xFFFF), [255, 255, 255]);
        assert_eq!(rgb565(0x07E0), [0, 255, 0]);
    }

    #[test]
    fn dib_24_bottom_up() {
        let mut d = vec![0u8; 40];
        d[0] = 40;
        d[4] = 1;
        d[8] = 2;
        d[12] = 1;
        d[14] = 24;
        d.extend_from_slice(&[0, 0, 255, 0]); // bottom row: red, padded
        d.extend_from_slice(&[255, 0, 0, 0]); // top row: blue, padded
        let (p, extra) = from_dib(&d).unwrap();
        assert_eq!(extra, 0);
        assert_eq!(&p.pixels[0..4], &[0, 0, 255, 255]);
        assert_eq!(&p.pixels[4..8], &[255, 0, 0, 255]);
    }
}
