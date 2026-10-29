//! One pixel through the pipeline, in the order Direct3D 3's documentation describes the
//! states: depth test, stipple, texture and its blend with the vertex colour, specular, fog,
//! alpha test, then blending with the pixel already in the target.

use crate::consts::*;
use crate::raster::Fragment;
use crate::state::{States, compare};
use crate::texture::{Format, Texture};

/// The states a triangle's pixels read, looked up once per triangle.
pub struct Pipe<'a> {
    z: Option<(u32, bool)>,
    stipple: Option<&'a States>,
    texture: Option<&'a Texture>,
    filter: u32,
    address: u32,
    tblend: u32,
    transparency: bool,
    specular: bool,
    fog: Option<[f32; 3]>,
    alpha_test: Option<(u32, f32)>,
    stippled_alpha: bool,
    blend: Option<(u32, u32)>,
    scale: f32,
}

/// The 4 × 4 ordered threshold used for stippled alpha.
const BAYER: [f32; 16] = [0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0];

fn rgb(c: u32) -> [f32; 3] {
    [(c >> 16 & 0xff) as f32, (c >> 8 & 0xff) as f32, (c & 0xff) as f32]
}

impl<'a> Pipe<'a> {
    pub fn new(s: &'a States, texture: Option<&'a Texture>, transparency: bool, scale: f32) -> Pipe<'a> {
        let on = |t| s.on(t);
        // Deviation: the minification filter is not told apart from the magnification one
        // (no mip levels); TEXTUREMAG chooses for both.
        let address = if on(rs::WRAPU) || on(rs::WRAPV) { ADDRESS_WRAP } else { s.get(rs::TEXTUREADDRESS) };
        // ALPHAREF: an 8-bit value, or DirectX 3's 16.16 fixed-point fraction when above 255.
        let r = s.get(rs::ALPHAREF);
        let alpha_ref = if r > 255 { (r as f32 / 65536.0 * 255.0).min(255.0) } else { r as f32 };
        Pipe {
            z: on(rs::ZENABLE).then(|| (s.get(rs::ZFUNC), on(rs::ZWRITEENABLE))),
            stipple: on(rs::STIPPLEENABLE).then_some(s),
            texture,
            filter: s.get(rs::TEXTUREMAG),
            address,
            tblend: s.get(rs::TEXTUREMAPBLEND),
            transparency,
            specular: on(rs::SPECULARENABLE),
            fog: on(rs::FOGENABLE).then(|| rgb(s.get(rs::FOGCOLOR))),
            alpha_test: on(rs::ALPHATESTENABLE).then(|| (s.get(rs::ALPHAFUNC), alpha_ref)),
            stippled_alpha: on(rs::STIPPLEDALPHA),
            blend: on(rs::BLENDENABLE).then(|| (s.get(rs::SRCBLEND), s.get(rs::DESTBLEND))),
            scale,
        }
    }

    /// The new 565 pixel for a fragment and whether its depth is written, or `None` when the
    /// pixel is left as it is.
    pub fn shade(&self, f: &Fragment, old: u16, old_z: f32) -> Option<(u16, bool)> {
        let mut write_z = false;
        if let Some((func, write)) = self.z {
            if !compare(func, f.z, old_z) {
                return None;
            }
            write_z = write;
        }
        // The stipple pattern is in the game's pixels, so it keeps its size when scaled.
        let (gx, gy) = ((f.x as f32 / self.scale) as usize, (f.y as f32 / self.scale) as usize);
        if self.stipple.is_some_and(|s| !s.stipple(gx, gy)) {
            return None;
        }
        let a = &f.a;
        let (cv, av) = ([a[2], a[3], a[4]], a[5]);
        let (mut c, mut alpha) = (cv, av);
        if let Some(t) = self.texture {
            let (ct, keyed) = t.sample(a[0], a[1], self.filter, self.address);
            if keyed && self.transparency {
                return None;
            }
            let at = ct[3];
            let has = t.format.has_alpha();
            let mul = |x: f32, y: f32| x * y / 255.0;
            (c, alpha) = match self.tblend {
                TBLEND_DECAL | TBLEND_COPY | TBLEND_DECALMASK => ([ct[0], ct[1], ct[2]], if has { at } else { av }),
                TBLEND_DECALALPHA => (
                    [0, 1, 2].map(|k| cv[k] + (ct[k] - cv[k]) * at / 255.0),
                    av,
                ),
                TBLEND_MODULATEALPHA => ([0, 1, 2].map(|k| mul(ct[k], cv[k])), if has { mul(at, av) } else { av }),
                TBLEND_ADD => ([0, 1, 2].map(|k| (ct[k] + cv[k]).min(255.0)), av),
                // MODULATE, MODULATEMASK and anything else.
                _ => ([0, 1, 2].map(|k| mul(ct[k], cv[k])), if has { at } else { av }),
            };
        }
        if self.specular {
            for k in 0..3 {
                c[k] = (c[k] + a[6 + k]).min(255.0);
            }
        }
        if let Some(fc) = self.fog {
            let fog = (a[9] / 255.0).clamp(0.0, 1.0);
            for k in 0..3 {
                c[k] = c[k] * fog + fc[k] * (1.0 - fog);
            }
        }
        if let Some((func, r)) = self.alpha_test {
            if !compare(func, alpha.round(), r) {
                return None;
            }
        }
        // Deviation: the documentation says stippled alpha draws translucency as a pattern
        // but not which; a 4 × 4 ordered threshold stands in.
        if self.stippled_alpha && alpha / 16.0 <= BAYER[(gy % 4) * 4 + gx % 4] {
            return None;
        }
        if let Some((src, dst)) = self.blend {
            let d = Format::Rgb565.unpack(old);
            let (fs, fd) = factors(src, dst, &c, alpha, &d);
            for k in 0..3 {
                c[k] = c[k] * fs[k] + d[k] * fd[k];
            }
        }
        // Deviation: no dither (DITHERENABLE); each field is cut to its top bits.
        Some((Format::Rgb565.pack565(c), write_z))
    }
}

/// The source and destination factors of `D3DBLEND`, per channel, 0 to 1. The target has no
/// alpha, so its alpha reads as 1.
fn factors(src: u32, dst: u32, s: &[f32; 3], sa: f32, d: &[f32; 4]) -> ([f32; 3], [f32; 3]) {
    let a = (sa / 255.0).clamp(0.0, 1.0);
    let f = |b: u32| -> [f32; 3] {
        match b {
            BLEND_ZERO => [0.0; 3],
            BLEND_SRCCOLOR => s.map(|x| x / 255.0),
            BLEND_INVSRCCOLOR => s.map(|x| 1.0 - x / 255.0),
            BLEND_SRCALPHA | BLEND_BOTHSRCALPHA => [a; 3],
            BLEND_INVSRCALPHA | BLEND_BOTHINVSRCALPHA => [1.0 - a; 3],
            BLEND_DESTALPHA => [1.0; 3],
            BLEND_INVDESTALPHA => [0.0; 3],
            BLEND_DESTCOLOR => [d[0] / 255.0, d[1] / 255.0, d[2] / 255.0],
            BLEND_INVDESTCOLOR => [1.0 - d[0] / 255.0, 1.0 - d[1] / 255.0, 1.0 - d[2] / 255.0],
            // min(source alpha, 1 − destination alpha), with the destination's alpha 1.
            BLEND_SRCALPHASAT => [0.0; 3],
            _ => [1.0; 3],
        }
    };
    // BOTHSRCALPHA and BOTHINVSRCALPHA as SRCBLEND set the destination factor too.
    let dst_f = match src {
        BLEND_BOTHSRCALPHA => [1.0 - a; 3],
        BLEND_BOTHINVSRCALPHA => [a; 3],
        _ => f(dst),
    };
    (f(src), dst_f)
}

impl Format {
    /// 8-bit RGB (clamped) to RGB565, each field cut to its top bits.
    pub fn pack565(self, [r, g, b]: [f32; 3]) -> u16 {
        let c = |x: f32| x.clamp(0.0, 255.0) as u16;
        (c(r) >> 3) << 11 | (c(g) >> 2) << 5 | c(b) >> 3
    }
}
