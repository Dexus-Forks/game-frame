//! A software Glide 2.x context: the calls a game makes, by their Glide names, onto the
//! pixel pipeline of this crate. A port that has traced its game's Glide calls makes the same
//! calls here, in the same order, and gets the picture.
//!
//! The context draws at `scale` times the resolution the game opened, so a 640 × 480 game
//! can be drawn at 1280 × 960: vertex x and y, the clip window and the frame grow, the rest
//! (texture coordinates, depth, colours) is what the game passed.
//! Deviation: at a scale above 1 the 12.4 vertex snap is at the output's sixteenths, not the
//! game's; at 1 it is the chip's.
//!
//! One TMU is modelled. Texture memory is a map from the start address the game passes to
//! `grTexDownloadMipMap` to the decoded largest level; `grTexSource` with the same address
//! selects it. Mipmaps are not modelled (the largest level is always used).

use crate::consts::*;
use crate::frame::Frame;
use crate::state::{Combine, State};
use crate::texture::Texture;
use crate::tri::{Vertex, triangle};
use crate::formats::{Format, size};
use std::collections::HashMap;

/// `GrTexInfo`: the levels, the aspect ratio, the format and the texels of the largest level
/// first.
#[derive(Clone, Copy, Debug)]
pub struct TexInfo<'a> {
    pub small_lod: u32,
    pub large_lod: u32,
    pub aspect: u32,
    pub format: u32,
    pub data: &'a [u8],
}

pub struct Glide {
    pub state: State,
    pub frame: Frame,
    /// The resolution the game opened, and the output's scale over it.
    pub width: usize,
    pub height: usize,
    pub scale: f32,
    /// `grGammaCorrectionValue`.
    pub gamma: f32,
    pub(crate) textures: HashMap<u32, Texture>,
    /// The texels as downloaded, for the palette formats: the TMU looks the palette up as it
    /// samples ("four texels are looked up simultaneously, each an independent 8-bit address",
    /// SST-1 specification section 5.56.2, *observed*), so a new palette recolours them.
    pub(crate) raw: HashMap<u32, (Format, usize, usize, Vec<u8>)>,
    pub(crate) source: Option<u32>,
    pub(crate) palette: Vec<[u8; 3]>,
    /// `guTexAllocateMemory`'s textures and the texture memory left (`gu`).
    pub(crate) gu: crate::gu::TexMemory,
}

impl Glide {
    /// `grSstOpen` at `width` × `height`, drawn at `scale` times that.
    pub fn open(width: usize, height: usize, scale: f32) -> Glide {
        let (ow, oh) = ((width as f32 * scale).round() as usize, (height as f32 * scale).round() as usize);
        Glide {
            state: State {
                clip: [0, 0, ow as i32, oh as i32],
                ..State::default()
            },
            frame: Frame::new(ow, oh),
            width,
            height,
            scale,
            gamma: 1.0,
            textures: HashMap::new(),
            raw: HashMap::new(),
            source: None,
            palette: vec![[0; 3]; 256],
            gu: crate::gu::TexMemory::default(),
        }
    }

    #[allow(non_snake_case)]
    pub fn grColorCombine(&mut self, function: u32, factor: u32, local: u32, other: u32, invert: bool) {
        self.state.colour = Combine::new(function, factor, local, other, invert);
    }

    #[allow(non_snake_case)]
    pub fn grAlphaCombine(&mut self, function: u32, factor: u32, local: u32, other: u32, invert: bool) {
        self.state.alpha = Combine::new(function, factor, local, other, invert);
    }

    /// `guColorCombineFunction`: an unknown function leaves the state as it was.
    #[allow(non_snake_case)]
    pub fn guColorCombineFunction(&mut self, f: u32) {
        if let Some(c) = Combine::colour_function(f) {
            self.state.colour = c;
        }
    }

    /// `guAlphaSource` (`GR_ALPHASOURCE_*`, 0–3), as the Glide 2.x reference manual gives the
    /// `grAlphaCombine` each stands for.
    #[allow(non_snake_case)]
    pub fn guAlphaSource(&mut self, mode: u32) {
        let c = |a, b, l, o| Combine::new(a, b, l, o, false);
        self.state.alpha = match mode {
            0 => c(FUNC_LOCAL, FACTOR_ZERO, LOCAL_CONSTANT, OTHER_CONSTANT),
            1 => c(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT),
            2 => c(FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE),
            3 => c(FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE),
            _ => return,
        };
    }

    /// `guTexCombineFunction` on the one TMU: decal, zero or one; others are taken as decal.
    #[allow(non_snake_case)]
    pub fn guTexCombineFunction(&mut self, _tmu: u32, f: u32) {
        self.state.texture = f;
    }

    #[allow(non_snake_case)]
    pub fn grConstantColorValue(&mut self, argb: u32) {
        self.state.constant = argb;
    }

    #[allow(non_snake_case)]
    pub fn grAlphaBlendFunction(&mut self, rgb_src: u32, rgb_dst: u32, alpha_src: u32, alpha_dst: u32) {
        self.state.blend = [rgb_src, rgb_dst, alpha_src, alpha_dst];
    }

    #[allow(non_snake_case)]
    pub fn grAlphaTestFunction(&mut self, function: u32) {
        self.state.alpha_test.0 = function;
    }

    #[allow(non_snake_case)]
    pub fn grAlphaTestReferenceValue(&mut self, value: u8) {
        self.state.alpha_test.1 = value;
    }

    /// `GR_CHROMAKEY_DISABLE` 0, `GR_CHROMAKEY_ENABLE` 1.
    #[allow(non_snake_case)]
    pub fn grChromakeyMode(&mut self, mode: u32) {
        self.state.chroma = mode != 0;
    }

    #[allow(non_snake_case)]
    pub fn grChromakeyValue(&mut self, rgb: u32) {
        self.state.key = rgb;
    }

    #[allow(non_snake_case)]
    pub fn grDepthBufferMode(&mut self, mode: u32) {
        self.state.depth_mode = mode;
    }

    #[allow(non_snake_case)]
    pub fn grDepthBufferFunction(&mut self, function: u32) {
        self.state.depth_function = function;
    }

    #[allow(non_snake_case)]
    pub fn grDepthMask(&mut self, on: bool) {
        self.state.depth_mask = on;
    }

    #[allow(non_snake_case)]
    pub fn grDitherMode(&mut self, mode: u32) {
        self.state.dither = mode != 0;
    }

    #[allow(non_snake_case)]
    pub fn grFogMode(&mut self, mode: u32) {
        self.state.fog_mode = mode;
    }

    #[allow(non_snake_case)]
    pub fn grFogColorValue(&mut self, argb: u32) {
        self.state.fog_colour = argb;
    }

    #[allow(non_snake_case)]
    pub fn grFogTable(&mut self, table: &[u8; 64]) {
        self.state.fog_table = *table;
    }

    /// `grTexFilterMode`: one filter is kept, the magnification one (no mipmaps, so a texel
    /// is never minified across levels).
    #[allow(non_snake_case)]
    pub fn grTexFilterMode(&mut self, _tmu: u32, _min: u32, mag: u32) {
        self.state.filter = mag;
    }

    #[allow(non_snake_case)]
    pub fn grTexClampMode(&mut self, _tmu: u32, s: u32, t: u32) {
        self.state.clamp = [s, t];
    }

    /// `grClipWindow(minx, miny, maxx, maxy)` in the game's pixels.
    #[allow(non_snake_case)]
    pub fn grClipWindow(&mut self, min_x: u32, min_y: u32, max_x: u32, max_y: u32) {
        let s = |v: u32| (v as f32 * self.scale).round() as i32;
        self.state.clip = [s(min_x), s(min_y), s(max_x), s(max_y)];
    }

    #[allow(non_snake_case)]
    pub fn grGammaCorrectionValue(&mut self, gamma: f32) {
        self.gamma = gamma;
    }

    /// `grBufferClear(colour, alpha, depth)`: the FASTFILL command, which fills the clip
    /// window and writes the colour and depth buffers only where `fbzMode`'s masks allow
    /// (SST-1 specification sections 5.21 and 5.23, *observed*): so with `grColorMask(false,
    /// …)` only the depth buffer is cleared.
    #[allow(non_snake_case)]
    pub fn grBufferClear(&mut self, argb: u32, _alpha: u8, depth: u16) {
        let [x0, y0, x1, y1] = self.state.clip;
        let colour = self.state.colour_mask;
        let depth_on = self.state.depth_mask;
        self.frame.clear_rect([x0, y0, x1, y1], colour.then_some(argb), depth_on.then_some(depth));
    }

    /// `grColorMask(rgb, alpha)`. The alpha planes are not modelled.
    #[allow(non_snake_case)]
    pub fn grColorMask(&mut self, rgb: bool, _alpha: bool) {
        self.state.colour_mask = rgb;
    }

    /// `grTexDownloadTable(tmu, GR_TEXTABLE_PALETTE, table)`: 256 words `0x00RRGGBB`, for the
    /// `GR_TEXFMT_P_8` textures downloaded after it.
    #[allow(non_snake_case)]
    pub fn grTexDownloadTable(&mut self, _tmu: u32, _kind: u32, table: &[u32]) {
        self.palette = table.iter().map(|&w| [(w >> 16) as u8, (w >> 8) as u8, w as u8]).collect();
        let keys: Vec<u32> = self.raw.keys().copied().collect();
        for k in keys {
            let (format, w, h, data) = self.raw[&k].clone();
            if let Ok(t) = Texture::from_glide(format, w, h, &data, Some(&self.palette)) {
                self.textures.insert(k, t);
            }
        }
    }

    /// A level downloaded at `start`: decoded, and kept as it is when it is a palette format.
    pub(crate) fn download(&mut self, start: u32, format: Format, w: usize, h: usize, data: &[u8]) {
        let paletted = format == Format::P8 || format == Format::Ap88;
        let palette = paletted.then_some(self.palette.as_slice());
        if let Ok(t) = Texture::from_glide(format, w, h, data, palette) {
            self.textures.insert(start, t);
            if paletted {
                self.raw.insert(start, (format, w, h, data.to_vec()));
            } else {
                self.raw.remove(&start);
            }
        }
    }

    /// `grTexDownloadMipMap(tmu, start, evenOdd, &info)`: the largest level decoded and kept
    /// at `start`. A format or size the texel decoder does not know is ignored.
    #[allow(non_snake_case)]
    pub fn grTexDownloadMipMap(&mut self, _tmu: u32, start: u32, _even_odd: u32, info: &TexInfo) {
        let (Some(format), Some((w, h))) = (Format::from_u32(info.format), size(info.large_lod, info.aspect)) else {
            return;
        };
        self.download(start, format, w, h, info.data);
    }

    /// A decoded texture put at `start` directly, for ports whose pictures are already RGBA.
    pub fn put_texture(&mut self, start: u32, texture: Texture) {
        self.textures.insert(start, texture);
    }

    /// `grTexSource(tmu, start, evenOdd, &info)`.
    #[allow(non_snake_case)]
    pub fn grTexSource(&mut self, _tmu: u32, start: u32) {
        self.source = Some(start);
    }

    /// `grDrawTriangle(a, b, c)`, vertices in the game's pixels.
    #[allow(non_snake_case)]
    pub fn grDrawTriangle(&mut self, a: &Vertex, b: &Vertex, c: &Vertex) {
        let s = self.scale;
        let up = |v: &Vertex| Vertex {
            x: v.x * s,
            y: v.y * s,
            ..*v
        };
        let texture = self.source.and_then(|k| self.textures.get(&k));
        triangle(&mut self.frame, &self.state, texture, [&up(a), &up(b), &up(c)]);
    }

    /// `grDrawPolygon(n, ilist, vlist)`: a convex polygon of the vertices `vlist[ilist[i]]`,
    /// drawn as a fan from the first (*inferred*: the Glide 2.x reference manual draws a
    /// convex polygon; a fan gives the same pixels for a convex one).
    #[allow(non_snake_case)]
    pub fn grDrawPolygon(&mut self, ilist: &[usize], vlist: &[Vertex]) {
        let verts: Vec<Vertex> = ilist.iter().filter_map(|&i| vlist.get(i).copied()).collect();
        self.fan(&verts);
    }

    /// A polygon as a fan from its first vertex, as games draw them with `grDrawTriangle`.
    pub fn fan(&mut self, verts: &[Vertex]) {
        for i in 1..verts.len().saturating_sub(1) {
            self.grDrawTriangle(&verts[0], &verts[i], &verts[i + 1]);
        }
    }

    /// The output picture in RGBA through the gamma curve.
    pub fn picture(&self) -> Vec<u8> {
        self.frame.rgba(self.gamma)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_textured_quad_at_twice_the_size() {
        let mut g = Glide::open(4, 4, 2.0);
        g.grDepthBufferMode(DEPTHBUFFER_WBUFFER);
        g.grDepthMask(true);
        g.grDitherMode(0);
        g.grBufferClear(0, 0, 0xffff);
        g.guColorCombineFunction(CC_DECAL_TEXTURE);
        // A GR_LOD_1 (1 × 1) RGB565 texture, pure green.
        let info = TexInfo {
            small_lod: 8,
            large_lod: 8,
            aspect: 3,
            format: Format::Rgb565 as u32,
            data: &[0xe0, 0x07],
        };
        g.grTexDownloadMipMap(0, 0x100, 3, &info);
        g.grTexSource(0, 0x100);
        let v = |x, y| Vertex {
            x,
            y,
            oow: 0.5,
            ..Vertex::default()
        };
        g.fan(&[v(0.0, 0.0), v(4.0, 0.0), v(4.0, 4.0), v(0.0, 4.0)]);
        assert_eq!(g.frame.width, 8);
        let p = g.picture();
        assert!(p.chunks(4).all(|c| c == [0, 255, 0, 255]));
    }

    #[test]
    fn the_clip_window_scales() {
        let mut g = Glide::open(640, 480, 1.5);
        g.grClipWindow(0, 0, 640, 480);
        assert_eq!(g.state.clip, [0, 0, 960, 720]);
    }
}
