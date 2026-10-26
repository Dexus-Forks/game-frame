//! `grLfbWriteRegion`: pixels from the host written into the colour buffer.
//!
//! *Observed* in the SST-1 specification, section 5.20: with the pixel pipeline bypassed
//! (`lfbMode` bit 8 = 0) linear frame buffer writes go "to the specified buffer unconditionally"
//! with no dither, depth test, alpha test, blending, fog, clipping or write masks, and only the
//! buffers the format names are written (the 16- and 32-bit colour formats leave the depth
//! buffer alone). *Inferred*: `grLfbWriteRegion` writes with the pipeline bypassed (the Glide 2.x
//! reference manual has it copy a region, and has `grLfbWriteRegion` ignore the render state).
//!
//! Deviation: there is one colour buffer here, so the front and back buffers are the same; at a
//! scale above 1 each pixel fills a block of the output.

use crate::frame::to_565;
use crate::glide::Glide;

/// `GR_LFB_SRC_FMT_*`.
pub const LFB_SRC_FMT_565: u32 = 0;
pub const LFB_SRC_FMT_555: u32 = 1;
pub const LFB_SRC_FMT_1555: u32 = 2;
pub const LFB_SRC_FMT_888: u32 = 4;
pub const LFB_SRC_FMT_8888: u32 = 5;

/// One source pixel as 565, or `None` for a format not modelled.
fn pixel(format: u32, data: &[u8], at: usize) -> Option<u16> {
    let h = |i: usize| u16::from_le_bytes([data[i], data[i + 1]]);
    let w = |i: usize| u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
    let widen = |v: u16, bits: u32| ((v as u32) << (8 - bits) | (v as u32) >> (2 * bits - 8)) as u8;
    Some(match format {
        LFB_SRC_FMT_565 => h(at * 2),
        LFB_SRC_FMT_555 | LFB_SRC_FMT_1555 => {
            let p = h(at * 2);
            let c = [widen(p >> 10 & 31, 5), widen(p >> 5 & 31, 5), widen(p & 31, 5)];
            to_565(c, 0, 0, false)
        }
        LFB_SRC_FMT_888 | LFB_SRC_FMT_8888 => {
            let p = w(at * 4);
            to_565([(p >> 16) as u8, (p >> 8) as u8, p as u8], 0, 0, false)
        }
        _ => return None,
    })
}

#[allow(non_snake_case)]
impl Glide {
    /// `grLfbWriteRegion(dstBuffer, dstX, dstY, srcFormat, srcWidth, srcHeight, srcStride,
    /// srcData)`, the stride in bytes. Returns false for a format not modelled.
    #[allow(clippy::too_many_arguments)]
    pub fn grLfbWriteRegion(&mut self, _buffer: u32, x: u32, y: u32, format: u32, width: u32, height: u32, stride: usize, data: &[u8]) -> bool {
        let bpp = match format {
            LFB_SRC_FMT_565 | LFB_SRC_FMT_555 | LFB_SRC_FMT_1555 => 2,
            LFB_SRC_FMT_888 | LFB_SRC_FMT_8888 => 4,
            _ => return false,
        };
        let s = self.scale;
        let (fw, fh) = (self.frame.width, self.frame.height);
        for row in 0..height as usize {
            for col in 0..width as usize {
                let off = row * stride + col * bpp;
                if off + bpp > data.len() {
                    continue;
                }
                let Some(p) = pixel(format, &data[row * stride..], col) else {
                    return false;
                };
                let (gx, gy) = (x as usize + col, y as usize + row);
                let (ox0, oy0) = ((gx as f32 * s) as usize, (gy as f32 * s) as usize);
                let (ox1, oy1) = (((gx + 1) as f32 * s) as usize, ((gy + 1) as f32 * s) as usize);
                for oy in oy0..oy1.min(fh) {
                    for ox in ox0..ox1.min(fw) {
                        self.frame.colour[oy * fw + ox] = p;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::*;

    #[test]
    fn a_region_bypasses_the_pipeline() {
        let mut g = Glide::open(4, 2, 1.0);
        // Masks and clipping do not apply to a bypassed write.
        g.grColorMask(false, false);
        g.grClipWindow(0, 0, 1, 1);
        g.grDepthBufferMode(DEPTHBUFFER_WBUFFER);
        let red = 0xf800u16.to_le_bytes();
        let data = [red, red, [0, 0], [0, 0]].concat();
        assert!(g.grLfbWriteRegion(1, 1, 1, LFB_SRC_FMT_565, 2, 1, 8, &data));
        assert_eq!(g.frame.colour[4..8], [0, 0xf800, 0xf800, 0]);
        assert_eq!(g.frame.depth, [0xffff; 8]);
    }

    #[test]
    fn scaled_pixels_fill_blocks() {
        let mut g = Glide::open(1, 1, 2.0);
        assert!(g.grLfbWriteRegion(1, 0, 0, LFB_SRC_FMT_565, 1, 1, 2, &0x001fu16.to_le_bytes()));
        assert_eq!(g.frame.colour, [0x001f; 4]);
    }
}
