//! Texture maps. POD converts each texture page to SGL's intermediate map (RGBA bytes, alpha
//! 0 for opaque and 255 for clear; black made clear on translucent maps), then calls
//! `sgl_preprocess_texture` into a second map and `sgl_create_texture` from that
//! (`FUN_004997d0`: map types 0 and 3, sizes 128 and 256).
//!
//! The "preprocessed" map here is the converted picture kept by this library: its pointer goes
//! into the game's map, and `sgl_create_texture` takes it back by that pointer.

use super::{sgl, trace};
use crate::scene::graph::Texture;

/// `sgl_intermediate_map`: `id` ("IMAP" for one the game made), the size, the pixels.
#[repr(C)]
pub struct IntermediateMap {
    id: [u8; 4],
    x_dim: i32,
    y_dim: i32,
    pixels: *mut u8,
}

/// `sgl_map_trans16`, `sgl_map_trans16_mm`.
fn translucent(map_type: i32) -> bool {
    matches!(map_type, 3 | 4)
}

/// The game's map as `Texture` RGBA (alpha 255 opaque).
unsafe fn convert(map_type: i32, m: &IntermediateMap) -> Option<Texture> {
    if m.pixels.is_null() || m.x_dim <= 0 || m.y_dim <= 0 || m.x_dim > 4096 || m.y_dim > 4096 {
        return None;
    }
    let (w, h) = (m.x_dim as u32, m.y_dim as u32);
    let src = unsafe { std::slice::from_raw_parts(m.pixels, (w * h * 4) as usize) };
    let trans = translucent(map_type);
    let mut rgba = src.to_vec();
    for p in rgba.chunks_exact_mut(4) {
        p[3] = if trans { 255 - p[3] } else { 255 };
    }
    Some(Texture { w, h, rgba, translucent: trans })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_preprocess_texture(
    map_type: i32, map_size: i32, generate_mipmap: i32, dither: i32, src: *const IntermediateMap, _filtered: *const *const IntermediateMap,
    processed: *mut IntermediateMap,
) -> i32 {
    trace!("sgl_preprocess_texture", "{map_type} {map_size} {generate_mipmap} {dither} {src:?} -> {processed:?}");
    if src.is_null() || processed.is_null() {
        return crate::scene::ERR_BAD_PARAMETER;
    }
    let Some(t) = (unsafe { convert(map_type, &*src) }) else { return crate::scene::ERR_BAD_PARAMETER };
    let (w, h) = (t.w as i32, t.h as i32);
    let mut buf = t.rgba.into_boxed_slice();
    let ptr = buf.as_mut_ptr();
    sgl().preprocessed.insert(ptr as usize, buf);
    unsafe { processed.write(IntermediateMap { id: *b"PREP", x_dim: w, y_dim: h, pixels: ptr }) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_create_texture(
    map_type: i32, map_size: i32, generate_mipmap: i32, dither: i32, pixel_data: *const IntermediateMap, _filtered: *const *const IntermediateMap,
) -> i32 {
    if pixel_data.is_null() {
        return crate::scene::ERR_BAD_PARAMETER;
    }
    let m = unsafe { &*pixel_data };
    let mut s = sgl();
    let tex = if &m.id == b"PREP" {
        s.preprocessed.remove(&(m.pixels as usize)).map(|buf| Texture { w: m.x_dim as u32, h: m.y_dim as u32, rgba: buf.into_vec(), translucent: translucent(map_type) })
    } else {
        unsafe { convert(map_type, m) }
    };
    let Some(tex) = tex else { return crate::scene::ERR_BAD_PARAMETER };
    let (w, h) = (tex.w, tex.h);
    // The graph keeps what the traversal needs; the pixels go to the render thread.
    let name = s.scene.create_texture(Texture { rgba: Vec::new(), ..tex.clone() });
    // The driver charges by map type and size index, not by the picture's size.
    s.texmem.take(name as u32, map_type, map_size);
    drop(s);
    crate::render::send(crate::render::Cmd::Texture(name as u32, tex));
    trace!("sgl_create_texture", "{map_type} {map_size} {generate_mipmap} {dither} {w}x{h} -> {name}");
    name
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_delete_texture(texture_name: i32) {
    trace!("sgl_delete_texture", "{texture_name}");
    let deleted = {
        let mut s = sgl();
        s.texmem.give_back(texture_name as u32);
        s.scene.delete_texture(texture_name)
    };
    if deleted {
        crate::render::send(crate::render::Cmd::DeleteTexture(texture_name as u32));
    }
}

/// For textures loaded from BMP files with `LoadBMPTexture`, which POD does not import: none.
#[unsafe(no_mangle)]
pub extern "C" fn FreeAllBMPTextures() {
    trace!("FreeAllBMPTextures");
}
