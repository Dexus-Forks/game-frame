//! Texture memory and the OpenGL textures made from it. Downloads land in a copy of the TMU's
//! memory at the game's addresses; a texture is decoded (with `gpu-1997`) the first time it is
//! drawn after its bytes or the palette change.

use crate::cmd::TexInfo;
use crate::gl::*;
use crate::log::log_once;
use gpu_1997::glide::{Format, size};
use gpu_1997::texel::{Tables, decode};
use std::collections::HashMap;

/// More than any Voodoo had (2 or 4 MB per TMU): the game lays memory out itself.
const MEMORY: usize = 16 << 20;

struct Texture {
    id: u32,
    start: usize,
    end: usize,
    stale: bool,
}

pub struct Textures {
    memory: Vec<u8>,
    palette: Vec<[u8; 3]>,
    cache: HashMap<(u32, TexInfo), Texture>,
}

impl Textures {
    pub fn new() -> Self {
        Textures { memory: vec![0; MEMORY], palette: vec![[0; 3]; 256], cache: HashMap::new() }
    }

    pub fn download(&mut self, start: u32, data: &[u8]) {
        let s = start as usize;
        let e = (s + data.len()).min(MEMORY);
        if s >= e {
            return;
        }
        self.memory[s..e].copy_from_slice(&data[..e - s]);
        for t in self.cache.values_mut() {
            if t.start < e && s < t.end {
                t.stale = true;
            }
        }
    }

    /// `GR_TEXTABLE_PALETTE`: `0x00RRGGBB` words.
    pub fn palette(&mut self, words: &[u32; 256]) {
        self.palette = words.iter().map(|w| [(w >> 16) as u8, (w >> 8) as u8, *w as u8]).collect();
        for ((_, info), t) in self.cache.iter_mut() {
            if matches!(Format::from_u32(info.format), Some(Format::P8 | Format::Ap88)) {
                t.stale = true;
            }
        }
    }

    /// Bind the texture at `start` described by `info` to unit 0, decoding it if needed.
    /// Returns its largest level's size, or `None` if it cannot be decoded.
    pub unsafe fn bind(&mut self, gl: &Gl, start: u32, info: TexInfo) -> Option<(usize, usize)> {
        let format = Format::from_u32(info.format)?;
        let (w, h) = size(info.large_lod, info.aspect)?;
        let bytes = format.texel().bytes(w, h);
        let s = start as usize;
        if s + bytes > MEMORY {
            return None;
        }
        let t = self.cache.entry((start, info)).or_insert_with(|| {
            let mut id = 0;
            unsafe { (gl.GenTextures)(1, &mut id) };
            Texture { id, start: s, end: s + bytes, stale: true }
        });
        unsafe { (gl.BindTexture)(TEXTURE_2D, t.id) };
        if t.stale {
            let tables = Tables { palette: Some(&self.palette), ncc: None };
            match decode(format.texel(), w, h, &self.memory[s..s + bytes], tables) {
                Ok(img) => unsafe {
                    (gl.PixelStorei)(UNPACK_ALIGNMENT, 1);
                    (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, w as i32, h as i32, 0, RGBA, UNSIGNED_BYTE, img.pixels.as_ptr().cast());
                },
                Err(e) => log_once!("texture decode failed: {e}"),
            }
            t.stale = false;
        }
        Some((w, h))
    }
}
