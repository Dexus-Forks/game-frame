//! A texture as the TMU samples it: RGBA texels, Glide's s and t (0–256 across the larger
//! side), point or bilinear, wrapped or clamped.

use crate::consts::*;
use gameframe_gpu_1997::glide::Format;
use gameframe_gpu_1997::Rgba;
use gameframe_gpu_1997::texel::Tables;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    /// RGBA, 8 bits a channel, rows top to bottom.
    pub texels: Vec<u8>,
}

impl Texture {
    pub fn from_rgba(p: &Rgba) -> Texture {
        Texture {
            width: p.width,
            height: p.height,
            texels: p.pixels.clone(),
        }
    }

    /// One level from texels in a Glide format (`grTexDownloadMipMap`'s data), with the
    /// palette `grTexDownloadTable(GR_TEXTABLE_PALETTE)` gave for `GR_TEXFMT_P_8` (RGB).
    pub fn from_glide(format: Format, width: usize, height: usize, data: &[u8], palette: Option<&[[u8; 3]]>) -> Result<Texture, String> {
        let tables = Tables { palette, ncc: None };
        let p = gameframe_gpu_1997::texel::decode(format.texel(), width, height, data, tables)?;
        Ok(Texture::from_rgba(&p))
    }

    fn texel(&self, x: i32, y: i32) -> [f32; 4] {
        let i = (y as usize * self.width + x as usize) * 4;
        let t = &self.texels[i..i + 4];
        [t[0], t[1], t[2], t[3]].map(|v| v as f32 / 255.0)
    }

    /// The texel at s, t (Glide's units: 256 across the larger side; a texel's centre is at
    /// half a texel). Deviation: the TMU's own fixed-point s, t and its bilinear weights
    /// (4 bits a side, *inferred* from the reference manual's description) are floats here.
    pub fn sample(&self, s: f32, t: f32, filter: u32, clamp: [u32; 2]) -> [f32; 4] {
        if self.width == 0 || self.height == 0 {
            return [0.0; 4];
        }
        let scale = self.width.max(self.height) as f32 / 256.0;
        let (u, v) = (s * scale, t * scale);
        let wrap = |i: i32, n: usize, mode: u32| -> i32 {
            if mode == TEXTURECLAMP_CLAMP {
                i.clamp(0, n as i32 - 1)
            } else {
                i.rem_euclid(n as i32)
            }
        };
        let at = |x: i32, y: i32| self.texel(wrap(x, self.width, clamp[0]), wrap(y, self.height, clamp[1]));
        if filter != TEXTUREFILTER_BILINEAR {
            return at(u.floor() as i32, v.floor() as i32);
        }
        let (u, v) = (u - 0.5, v - 0.5);
        let (x0, y0) = (u.floor(), v.floor());
        let (fx, fy) = (u - x0, v - y0);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let (a, b, c, d) = (at(x0, y0), at(x0 + 1, y0), at(x0, y0 + 1), at(x0 + 1, y0 + 1));
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * fx;
            let bottom = c[k] + (d[k] - c[k]) * fx;
            top + (bottom - top) * fy
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker() -> Texture {
        // 2 × 2: white, black / black, white.
        let w = [255, 255, 255, 255];
        let b = [0, 0, 0, 255];
        Texture {
            width: 2,
            height: 2,
            texels: [w, b, b, w].concat(),
        }
    }

    #[test]
    fn point_sampling_covers_the_texture_in_256() {
        let t = checker();
        assert_eq!(t.sample(10.0, 10.0, TEXTUREFILTER_POINT_SAMPLED, [0, 0])[0], 1.0);
        assert_eq!(t.sample(200.0, 10.0, TEXTUREFILTER_POINT_SAMPLED, [0, 0])[0], 0.0);
        // Wrapped: 256 + 10 is the first texel again.
        assert_eq!(t.sample(266.0, 10.0, TEXTUREFILTER_POINT_SAMPLED, [0, 0])[0], 1.0);
    }

    #[test]
    fn bilinear_between_texel_centres() {
        let t = checker();
        // Halfway between the two centres of the top row (64 and 192): grey.
        let g = t.sample(128.0, 64.0, TEXTUREFILTER_BILINEAR, [1, 1]);
        assert!((g[0] - 0.5).abs() < 1e-6);
        // At a centre: the texel itself.
        assert_eq!(t.sample(64.0, 64.0, TEXTUREFILTER_BILINEAR, [1, 1])[0], 1.0);
    }

    #[test]
    fn rgb565_from_glide() {
        let t = Texture::from_glide(Format::Rgb565, 1, 1, &[0x00, 0xf8], None).unwrap();
        assert_eq!(t.texels, [255, 0, 0, 255]);
    }
}
