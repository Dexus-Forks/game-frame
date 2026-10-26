//! The frame buffer: one 16-bit RGB565 colour buffer, as the Voodoo Graphics keeps it, and a
//! depth buffer beside it; the clear, and the read-out to RGBA through the gamma table.

use crate::consts::*;

/// The 4 × 4 ordered dither the Voodoo applies when it writes 8-bit colour into 565
/// (`grDitherMode(GR_DITHER_4x4)`, the default). Deviation: the matrix is the usual Bayer
/// one; the chip's own table is *unknown*.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

pub struct Frame {
    pub width: usize,
    pub height: usize,
    /// RGB565, rows top to bottom (`GR_ORIGIN_UPPER_LEFT`).
    pub colour: Vec<u16>,
    /// The depth of each pixel: `ooz` (0–65535) in Z-buffer mode, *w* (`1 / oow`) in W-buffer
    /// mode. Deviation: the Voodoo stores 16 bits, in W mode a 4.12 floating form of *w*;
    /// a float is kept here, so depth ties are rarer than on the chip.
    pub depth: Vec<f32>,
}

/// The depth value `grBufferClear` writes for `depth` (`0xffff` is the far value in both
/// modes: in W mode it stands for the largest *w*).
pub fn clear_depth(depth: u16, mode: u32) -> f32 {
    if mode == DEPTHBUFFER_WBUFFER && depth == 0xffff {
        f32::INFINITY
    } else {
        depth as f32
    }
}

/// 8-bit r, g, b (0–1) to 565 with the dither at pixel `x`, `y`.
pub fn to_565(c: [f32; 3], x: usize, y: usize, dither: bool) -> u16 {
    let d = if dither { BAYER[y & 3][x & 3] as f32 / 16.0 } else { 0.5 };
    let q = |v: f32, bits: u32| {
        let top = ((1u32 << bits) - 1) as f32;
        ((v.clamp(0.0, 1.0) * top + d).floor() as u32).min(top as u32)
    };
    (q(c[0], 5) << 11 | q(c[1], 6) << 5 | q(c[2], 5)) as u16
}

/// 565 to r, g, b 0–1, widened by bit replication.
pub fn from_565(p: u16) -> [f32; 3] {
    let r = (p >> 11) as u32 & 31;
    let g = (p >> 5) as u32 & 63;
    let b = p as u32 & 31;
    [(r << 3 | r >> 2) as f32 / 255.0, (g << 2 | g >> 4) as f32 / 255.0, (b << 3 | b >> 2) as f32 / 255.0]
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Frame {
        Frame {
            width,
            height,
            colour: vec![0; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    /// `grBufferClear(colour, alpha, depth)`: colour `0xAARRGGBB`; the alpha is not kept (no
    /// alpha buffer on a one-TMU board with depth).
    pub fn clear(&mut self, colour: u32, depth: u16, depth_mode: u32) {
        let c = crate::combine::argb(colour);
        let p = to_565([c[0], c[1], c[2]], 0, 0, false);
        self.colour.fill(p);
        self.depth.fill(clear_depth(depth, depth_mode));
    }

    /// The picture as RGBA, 8 bits a channel, through `grGammaCorrectionValue`'s curve
    /// (`c^(1/gamma)`, as the Glide 2.x reference manual gives it).
    pub fn rgba(&self, gamma: f32) -> Vec<u8> {
        let table: Vec<u8> = (0..256)
            .map(|i| {
                let v = (i as f32 / 255.0).powf(1.0 / gamma.max(0.01));
                (v * 255.0).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        let mut out = Vec::with_capacity(self.colour.len() * 4);
        for &p in &self.colour {
            let c = from_565(p);
            for v in c {
                out.push(table[(v * 255.0).round() as usize]);
            }
            out.push(255);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_and_black_survive_the_dither() {
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(to_565([1.0; 3], x, y, true), 0xffff);
                assert_eq!(to_565([0.0; 3], x, y, true), 0);
            }
        }
        assert_eq!(from_565(0xffff), [1.0; 3]);
    }

    #[test]
    fn a_mid_grey_dithers_between_two_levels() {
        let v = 100.0 / 255.0;
        let levels: std::collections::BTreeSet<u16> =
            (0..16).map(|i| to_565([v; 3], i & 3, i >> 2, true) >> 11).collect();
        assert_eq!(levels.len(), 2);
    }

    #[test]
    fn clear_and_read_out() {
        let mut f = Frame::new(2, 1);
        f.clear(0x00ff0000, 0xffff, DEPTHBUFFER_WBUFFER);
        assert_eq!(f.rgba(1.0), [255, 0, 0, 255, 255, 0, 0, 255]);
        assert_eq!(f.depth[0], f32::INFINITY);
        f.clear(0, 0xffff, DEPTHBUFFER_ZBUFFER);
        assert_eq!(f.depth[1], 65535.0);
    }
}
