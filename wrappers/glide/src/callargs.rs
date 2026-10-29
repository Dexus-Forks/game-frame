//! What Glide's pointers point to, as the call log (`shim::calls`, `WRAPPER_CALLS`) writes it.
//! An address differs between the original and the port, so the log writes the values:
//! a vertex as the fields the state reads (`used`), a polygon as its vertices, a `GrTexInfo`
//! as its fields, and texture data, palettes, fog tables and frame-buffer pixels as their size
//! and hash.

use crate::api::shadow;
use crate::api::tex::GrTexInfo;
use crate::layout;
use crate::used::{self, Fields};
use crate::vertex::Vertex;
use shim::calls::{Arg, Bytes};
use std::fmt::Write as _;
use voodoo::formats::{Format, TEXTABLE_PALETTE, size};

/// The fields of a vertex the Voodoo reads under the state the game has set (`used`).
fn fields() -> Fields {
    let s = shadow::get();
    used::used(&used::State {
        cc: s.cc,
        ac: s.ac,
        chroma: s.chroma != 0,
        delta0: s.delta0 != 0,
        depth_mode: s.depth_mode,
        fog_mode: s.fog_mode,
        stw_hint: s.stw_hint,
    })
}

/// The vertex a `*const GrVertex` points to: the fields the state reads.
pub struct V(pub *const Vertex);
impl Arg for V {
    fn put(&self, out: &mut String) {
        match self.0.is_null() {
            true => out.push_str("null"),
            false => used::write(&unsafe { self.0.read_unaligned() }, fields(), out),
        }
    }
}

/// The vertices of a polygon, in the order drawn.
pub struct Vs(pub Vec<Vertex>);
impl Arg for Vs {
    fn put(&self, out: &mut String) {
        let f = fields();
        out.push('[');
        for (k, v) in self.0.iter().enumerate() {
            if k > 0 {
                out.push(' ');
            }
            used::write(v, f, out);
        }
        out.push(']');
    }
}

/// A `GrTexInfo`'s fields; with `data`, its mip levels as their size and hash.
pub struct Info(pub *const GrTexInfo, pub bool);
impl Arg for Info {
    fn put(&self, out: &mut String) {
        let Some(i) = (unsafe { self.0.as_ref() }) else { return out.push_str("null") };
        let _ = write!(out, "{{lods {:#x}-{:#x} aspect {:#x} format {:#x}", i.large_lod, i.small_lod, i.aspect, i.format);
        if self.1 {
            let n = Format::from_u32(i.format).and_then(|f| layout::host_offset(i.large_lod, i.small_lod + 1, i.aspect, f));
            out.push_str(" data ");
            Data(i.data, n.unwrap_or(0)).put(out);
        }
        out.push('}');
    }
}

/// `n` bytes at a pointer, as their size and hash.
pub struct Data(pub *const u8, pub usize);
impl Arg for Data {
    fn put(&self, out: &mut String) {
        match self.0.is_null() || self.1 == 0 {
            true => self.0.put(out),
            false => Bytes(unsafe { std::slice::from_raw_parts(self.0, self.1) }).put(out),
        }
    }
}

/// Rows `first`..=`last` of one mip level, as `grTexDownloadMipMapLevelPartial` reads them.
pub fn level(data: *const u8, lod: u32, aspect: u32, format: u32, rows: Option<(i32, i32)>) -> Data {
    let n = match (Format::from_u32(format), size(lod, aspect)) {
        (Some(f), Some((w, h))) => {
            let (first, last) = rows.unwrap_or((0, h as i32 - 1));
            f.texel().bytes(w, 1) * (last - first + 1).max(0) as usize
        }
        _ => 0,
    };
    Data(data, n)
}

/// A palette's words `first`..=`last`, or an NCC table's 112 bytes.
pub fn table(kind: u32, data: *const u32, first: i32, last: i32) -> Data {
    if kind == TEXTABLE_PALETTE {
        let (first, last) = (first.clamp(0, 255) as usize, last.clamp(0, 255) as usize);
        Data(unsafe { data.add(first) }.cast(), (last + 1).saturating_sub(first) * 4)
    } else {
        Data(data.cast(), 112)
    }
}

#[derive(Clone, Copy)]
/// A region of pixels the game hands over, `w` by `h` at `bpp` bytes, rows `stride` apart.
pub struct Region(pub *const u8, pub u32, pub u32, pub i32, pub usize);
impl Arg for Region {
    fn put(&self, out: &mut String) {
        let Region(src, w, h, stride, bpp) = *self;
        if src.is_null() {
            return out.push_str("null");
        }
        let mut bytes = Vec::with_capacity(w as usize * h as usize * bpp);
        for y in 0..h as isize {
            let row = unsafe { src.offset(y * stride as isize) };
            bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(row, w as usize * bpp) });
        }
        Bytes(&bytes).put(out);
    }
}
