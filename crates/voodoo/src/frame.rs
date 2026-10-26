//! The frame buffer: one 16-bit RGB565 colour buffer and one 16-bit depth buffer, as the
//! Voodoo Graphics keeps them; the W buffer's floating format; the clear; the dither; the
//! read-out to RGBA through the gamma curve.


/// The 4 × 4 ordered dither. *Observed* in the Voodoo2 specification, revision 1.16
/// (`fogMode`'s diagram): bit 3 = y0 xor x0, bit 2 = y0, bit 1 = y1 xor x1, bit 0 = y1.
/// *Inferred*: the SST-1's colour dither (section 5.19, `fbzMode` bit 11 = 0) uses the same
/// matrix. Deviation: how the chip adds it before dropping the low bits is *unknown*; here
/// the value 0–15 is added as sixteenths of one 565 step.
pub fn dither(x: usize, y: usize) -> u32 {
    let (x0, x1, y0, y1) = (x & 1, x >> 1 & 1, y & 1, y >> 1 & 1);
    ((y0 ^ x0) << 3 | y0 << 2 | (y1 ^ x1) << 1 | y1) as u32
}

pub struct Frame {
    pub width: usize,
    pub height: usize,
    /// RGB565, rows top to bottom (`GR_ORIGIN_UPPER_LEFT`).
    pub colour: Vec<u16>,
    /// The depth buffer: `ooz`'s integer part in Z mode, [`wfloat`] in W mode.
    pub depth: Vec<u16>,
}

/// The W buffer's 16-bit value for `oow`. *Observed* in the SST-1 specification, section
/// 5.19 (the depth-buffering diagram): the iterated 1/W in 16.32 fixed point; an integer
/// part other than 0 underflows (and clamps to 0, the nearest); otherwise the exponent is
/// where the first one is found in bits 31–16, the 12-bit mantissa follows that one, and the
/// value is the exponent over the inverted mantissa, so it rises with distance.
/// *Unknown*: the diagram takes the mantissa from bits 30–16 only; here it is the 12 bits
/// after the leading one from all 32 fraction bits, and 1/W below 2^-16 is the farthest,
/// `0xffff`.
pub fn wfloat(oow: f32) -> u16 {
    if oow.is_nan() || oow >= 1.0 {
        return 0;
    }
    let w = (oow.max(0.0) as f64 * 4294967296.0) as u64 as u32;
    if w >> 16 == 0 {
        return 0xffff;
    }
    let exp = w.leading_zeros();
    let mant = (w << exp << 1) >> 20;
    (exp << 12 | (!mant & 0xfff)) as u16
}

/// 8-bit r, g, b to 565 with the dither at pixel `x`, `y`.
pub fn to_565(c: [u8; 3], x: usize, y: usize, dither_on: bool) -> u16 {
    let d = if dither_on { dither(x, y) } else { 8 };
    let q = |v: u8, bits: u32| {
        let top = (1u32 << bits) - 1;
        ((v as u32 * top * 16 + d * 255) / (255 * 16)).min(top)
    };
    (q(c[0], 5) << 11 | q(c[1], 6) << 5 | q(c[2], 5)) as u16
}

/// 565 to 8-bit r, g, b, widened by bit replication (as the SST-1 specification's texture
/// format table widens 565 texels, section 5.50; *inferred* for the frame buffer).
pub fn from_565(p: u16) -> [u8; 3] {
    let r = (p >> 11) as u32 & 31;
    let g = (p >> 5) as u32 & 63;
    let b = p as u32 & 31;
    [(r << 3 | r >> 2) as u8, (g << 2 | g >> 4) as u8, (b << 3 | b >> 2) as u8]
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Frame {
        Frame {
            width,
            height,
            colour: vec![0; width * height],
            depth: vec![0xffff; width * height],
        }
    }

    /// `grBufferClear(colour, alpha, depth)`: colour `0xAARRGGBB`, written undithered; the
    /// depth value as it is (`0xffff` is the far value in both modes). The alpha is not kept:
    /// with a depth buffer there are no alpha planes (section 5.19, bit 18).
    pub fn clear(&mut self, colour: u32, depth: u16) {
        let c = crate::combine::argb(colour);
        let p = to_565([c[0], c[1], c[2]], 0, 0, false);
        self.colour.fill(p);
        self.depth.fill(depth);
    }

    /// The picture as RGBA, 8 bits a channel, through `grGammaCorrectionValue`'s curve
    /// (`c^(1/gamma)`, the Glide 2.x reference manual).
    pub fn rgba(&self, gamma: f32) -> Vec<u8> {
        let table: Vec<u8> = (0..256)
            .map(|i| {
                let v = (i as f32 / 255.0).powf(1.0 / gamma.max(0.01));
                (v * 255.0).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        let mut out = Vec::with_capacity(self.colour.len() * 4);
        for &p in &self.colour {
            for v in from_565(p) {
                out.push(table[v as usize]);
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
    fn the_dither_matrix() {
        let rows: Vec<[u32; 4]> = (0..4).map(|y| std::array::from_fn(|x| dither(x, y))).collect();
        assert_eq!(rows, [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]);
    }

    #[test]
    fn white_and_black_survive_the_dither() {
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(to_565([255; 3], x, y, true), 0xffff);
                assert_eq!(to_565([0; 3], x, y, true), 0);
            }
        }
        assert_eq!(from_565(0xffff), [255; 3]);
    }

    #[test]
    fn a_mid_grey_dithers_between_two_levels() {
        let levels: std::collections::BTreeSet<u16> =
            (0..16).map(|i| to_565([100; 3], i & 3, i >> 2, true) >> 11).collect();
        assert_eq!(levels.len(), 2);
    }

    #[test]
    fn wfloat_rises_with_distance() {
        assert_eq!(wfloat(1.0), 0);
        assert_eq!(wfloat(2.0), 0);
        // 1/W = 0.5: the first one in bit 31, mantissa 0, inverted 0xfff.
        assert_eq!(wfloat(0.5), 0x0fff);
        assert_eq!(wfloat(0.75), 0x07ff);
        assert_eq!(wfloat(0.25), 0x1fff);
        assert_eq!(wfloat(1e-6), 0xffff);
        let mut last = 0;
        for i in 1..2000 {
            let v = wfloat(1.0 / (1.0 + i as f32 * 0.5));
            assert!(v >= last);
            last = v;
        }
    }

    #[test]
    fn clear_and_read_out() {
        let mut f = Frame::new(2, 1);
        f.clear(0x00ff0000, 0xffff);
        assert_eq!(f.rgba(1.0), [255, 0, 0, 255, 255, 0, 0, 255]);
        assert_eq!(f.depth, [0xffff; 2]);
    }
}
