//! Texture memory and the OpenGL textures made from it. Downloads land in a copy of the TMU's
//! memory at the game's addresses; a texture is decoded (with `gameframe-voodoo`) the first time it is
//! drawn after its bytes, the palette or its NCC table change. Every level from the largest to
//! the smallest is decoded, from the addresses `layout` gives, so mipmapping can use them.

use crate::cmd::Source;
use crate::gl::*;
use crate::layout;
use crate::log::log_once;
use voodoo::formats::{Format, size};
use voodoo::ncc::NccTable;
use texel::{Tables, decode};
use std::collections::HashMap;

/// More than any Voodoo had (2 or 4 MB per TMU): the game lays memory out itself.
const MEMORY: usize = 16 << 20;

struct Texture {
    id: u32,
    /// The bytes of TMU memory each level reads.
    ranges: Vec<(usize, usize)>,
    stale: bool,
}

pub struct Textures {
    memory: Vec<u8>,
    palette: Vec<[u8; 3]>,
    ncc: [Option<NccTable>; 2],
    cache: HashMap<Source, Texture>,
}

fn is_ncc(f: Option<Format>) -> bool {
    matches!(f, Some(Format::Yiq422 | Format::Ayiq8422))
}

impl Textures {
    pub fn new() -> Self {
        Textures { memory: vec![0; MEMORY], palette: vec![[0; 3]; 256], ncc: [None, None], cache: HashMap::new() }
    }

    pub fn download(&mut self, start: u32, data: &[u8]) {
        let s = start as usize;
        let e = (s + data.len()).min(MEMORY);
        if s >= e {
            return;
        }
        self.memory[s..e].copy_from_slice(&data[..e - s]);
        for t in self.cache.values_mut() {
            if t.ranges.iter().any(|&(a, b)| a < e && s < b) {
                t.stale = true;
            }
        }
    }

    /// `GR_TEXTABLE_PALETTE`: `0x00RRGGBB` words from entry `first` on.
    pub fn palette(&mut self, first: usize, words: &[u32]) {
        for (k, w) in words.iter().enumerate().take(256 - first.min(256)) {
            self.palette[first + k] = [(w >> 16) as u8, (w >> 8) as u8, *w as u8];
        }
        for (src, t) in self.cache.iter_mut() {
            if matches!(Format::from_u32(src.info.format), Some(Format::P8 | Format::Ap88)) {
                t.stale = true;
            }
        }
    }

    /// NCC table 0 or 1 from a `GuNccTable`.
    pub fn ncc(&mut self, which: u8, bytes: &[u8]) {
        self.ncc[which as usize & 1] = NccTable::from_gu_bytes(bytes);
        for (src, t) in self.cache.iter_mut() {
            if is_ncc(Format::from_u32(src.info.format)) && src.ncc == which {
                t.stale = true;
            }
        }
    }

    /// Bind the texture `src` to unit 0, decoding it if needed. Returns its largest level's
    /// size, or `None` if it cannot be decoded.
    pub unsafe fn bind(&mut self, gl: &Gl, mut src: Source) -> Option<(usize, usize)> {
        let format = Format::from_u32(src.info.format)?;
        if !is_ncc(Some(format)) {
            src.ncc = 0;
        }
        let i = src.info;
        let (w, h) = size(i.large_lod, i.aspect)?;
        let levels = i.large_lod..=i.small_lod.max(i.large_lod).min(8);
        let mut ranges = Vec::new();
        for lod in levels.clone() {
            let at = layout::level_address(src.start, src.bases, i.large_lod, lod, i.aspect, format)?;
            let n = layout::host_level_bytes(lod, i.aspect, format)?;
            if at + n > MEMORY {
                return None;
            }
            ranges.push((at, at + n));
        }
        let t = self.cache.entry(src).or_insert_with(|| {
            let mut id = 0;
            unsafe { (gl.GenTextures)(1, &mut id) };
            Texture { id, ranges, stale: true }
        });
        unsafe { (gl.BindTexture)(TEXTURE_2D, t.id) };
        if t.stale {
            let tables = Tables { palette: Some(&self.palette), ncc: self.ncc[src.ncc as usize & 1].as_ref().map(|t| t as &dyn texel::Yiq) };
            unsafe {
                (gl.PixelStorei)(UNPACK_ALIGNMENT, 1);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_BASE_LEVEL, 0);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAX_LEVEL, t.ranges.len() as i32 - 1);
            }
            for (k, (lod, &(a, b))) in levels.zip(&t.ranges).enumerate() {
                let (lw, lh) = size(lod, i.aspect)?;
                match decode(format.texel(), lw, lh, &self.memory[a..b], tables) {
                    Ok(img) => unsafe {
                        (gl.TexImage2D)(TEXTURE_2D, k as i32, RGBA8 as i32, lw as i32, lh as i32, 0, RGBA, UNSIGNED_BYTE, img.pixels.as_ptr().cast());
                    },
                    Err(e) => log_once!("texture decode failed: {e}"),
                }
            }
            t.stale = false;
        }
        Some((w, h))
    }
}
