//! Textures: the 16-bit surface formats 1997 cards offered through
//! `IDirect3DDevice::EnumTextureFormats`, and sampling them.

use crate::consts::*;

/// A texture surface's pixel format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// `RRRRRGGGGGGBBBBB`, no alpha.
    Rgb565,
    /// `ARRRRRGGGGGBBBBB`: one bit of alpha.
    Argb1555,
    /// `AAAARRRRGGGGBBBB`.
    Argb4444,
}

impl Format {
    /// Does the format carry alpha (`DDPF_ALPHAPIXELS`)?
    pub fn has_alpha(self) -> bool {
        self != Format::Rgb565
    }

    /// A texel as `[r, g, b, a]`, 0 to 255, each field widened by repeating its top bits.
    pub fn unpack(self, t: u16) -> [f32; 4] {
        let t = t as u32;
        let w = |v: u32, bits: u32| {
            let v = v << (8 - bits);
            (v | v >> bits) as f32
        };
        match self {
            Format::Rgb565 => [w(t >> 11 & 31, 5), w(t >> 5 & 63, 6), w(t & 31, 5), 255.0],
            Format::Argb1555 => [
                w(t >> 10 & 31, 5),
                w(t >> 5 & 31, 5),
                w(t & 31, 5),
                if t & 0x8000 != 0 { 255.0 } else { 0.0 },
            ],
            Format::Argb4444 => [w(t >> 8 & 15, 4), w(t >> 4 & 15, 4), w(t & 15, 4), w(t >> 12, 4)],
        }
    }
}

/// A texture: one level, `width` × `height` texels, row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub format: Format,
    pub texels: Vec<u16>,
    /// The surface's source colour key (`DDCKEY_SRCBLT`), as a texel value: on a card with
    /// `D3DPTEXTURECAPS_TRANSPARENCY`, texels equal to it are not drawn.
    pub key: Option<u16>,
}

fn address(i: i64, n: usize, mode: u32) -> usize {
    let n = n as i64;
    (match mode {
        ADDRESS_CLAMP => i.clamp(0, n - 1),
        ADDRESS_MIRROR => {
            let m = i.rem_euclid(2 * n);
            if m < n { m } else { 2 * n - 1 - m }
        }
        _ => i.rem_euclid(n),
    }) as usize
}

impl Texture {
    fn texel(&self, x: i64, y: i64, mode: u32) -> u16 {
        self.texels[address(y, self.height, mode) * self.width + address(x, self.width, mode)]
    }

    /// The texture at `(u, v)` (0 to 1 across it), `[r, g, b, a]` 0 to 255, and whether a
    /// colour-keyed texel was hit (nearest: that texel; linear: the nearest of the four).
    /// Texel centres are at half-integer coordinates; a linear filter weighs the four texels
    /// around the point by their distance.
    pub fn sample(&self, u: f32, v: f32, filter: u32, mode: u32) -> ([f32; 4], bool) {
        let (x, y) = (u * self.width as f32, v * self.height as f32);
        let keyed = |t: u16| self.key == Some(t);
        if !matches!(filter, FILTER_LINEAR | FILTER_MIPLINEAR | FILTER_LINEARMIPNEAREST | FILTER_LINEARMIPLINEAR) {
            let t = self.texel(x.floor() as i64, y.floor() as i64, mode);
            return (self.format.unpack(t), keyed(t));
        }
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (ax, ay) = (fx - x0, fy - y0);
        let (x0, y0) = (x0 as i64, y0 as i64);
        let t = [
            self.texel(x0, y0, mode),
            self.texel(x0 + 1, y0, mode),
            self.texel(x0, y0 + 1, mode),
            self.texel(x0 + 1, y0 + 1, mode),
        ];
        let w = [(1.0 - ax) * (1.0 - ay), ax * (1.0 - ay), (1.0 - ax) * ay, ax * ay];
        let mut out = [0f32; 4];
        for (t, w) in t.iter().zip(w) {
            let c = self.format.unpack(*t);
            for k in 0..4 {
                out[k] += c[k] * w;
            }
        }
        let nearest = t[(ax >= 0.5) as usize + 2 * (ay >= 0.5) as usize];
        (out, keyed(nearest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_widen_their_fields() {
        assert_eq!(Format::Rgb565.unpack(0xffff), [255.0; 4]);
        assert_eq!(Format::Argb1555.unpack(0x7c00), [255.0, 0.0, 0.0, 0.0]);
        assert_eq!(Format::Argb4444.unpack(0xff0f), [255.0, 0.0, 255.0, 255.0]);
    }

    #[test]
    fn addressing_wraps_mirrors_and_clamps() {
        assert_eq!(address(-1, 4, ADDRESS_WRAP), 3);
        assert_eq!(address(4, 4, ADDRESS_MIRROR), 3);
        assert_eq!(address(9, 4, ADDRESS_CLAMP), 3);
    }

    #[test]
    fn linear_filtering_blends_neighbours() {
        let t = Texture { width: 2, height: 1, format: Format::Rgb565, texels: vec![0, 0xffff], key: None };
        let (c, _) = t.sample(0.5, 0.5, FILTER_LINEAR, ADDRESS_CLAMP);
        assert!((c[0] - 127.5).abs() < 1.0);
        let (c, _) = t.sample(0.25, 0.5, FILTER_NEAREST, ADDRESS_CLAMP);
        assert_eq!(c[0], 0.0);
    }
}
