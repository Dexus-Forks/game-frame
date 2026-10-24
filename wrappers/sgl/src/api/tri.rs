//! The immediate-mode calls (`sgltri_*`, sgl.h). The game sends faces already on the screen
//! each frame, under a context. The calls read the game's `SGLCONTEXT`, faces and `SGLVERTEX`
//! array. They add to the frame in `scene::tri`. `sgltri_render` hands the frame to
//! `immediate`. Each call reports the errors sgl.h lists for it. `sgl_get_errors` reads them
//! back.

use super::{error, sgl, trace};
use crate::scene::tri::{CONTEXT_SIZE, Context, FACESIND3DFORMAT, VERTEX_SIZE, Vertex};
use crate::scene::{ERR_BAD_PARAMETER, walk::View};

/// The most vertices one call may use: the chip's own buffer held far fewer.
const MAX_VERTICES: u32 = 1 << 20;

unsafe fn context(p: *const u8) -> Option<Context> {
    (!p.is_null()).then(|| Context::read(unsafe { std::slice::from_raw_parts(p, CONTEXT_SIZE) }))
}

/// The faces, `N` indices each: `int[N]`, or for triangles with `SGLTT_FACESIND3DFORMAT`,
/// `SGLD3DFACE` (three 16-bit indices and 16-bit strip flags, which are not needed here).
unsafe fn faces<const N: usize>(p: *const u8, n: usize, d3d: bool) -> Vec<[u32; N]> {
    let stride = if d3d { 8 } else { 4 * N };
    let b = unsafe { std::slice::from_raw_parts(p, n * stride) };
    b.chunks_exact(stride)
        .map(|f| {
            std::array::from_fn(|i| if d3d { u16::from_le_bytes([f[i * 2], f[i * 2 + 1]]) as u32 } else { u32::from_le_bytes(f[i * 4..i * 4 + 4].try_into().unwrap()) })
        })
        .collect()
}

/// Add `n` faces of `N` vertices, as `sgltri_triangles` and `sgltri_quads` do.
unsafe fn add<const N: usize>(name: &str, ctx: *const u8, n: i32, pf: *const u8, pv: *const u8) {
    if n == 0 {
        error(0);
        return;
    }
    let Some(c) = (unsafe { context(ctx) }) else { return error(ERR_BAD_PARAMETER) };
    if pf.is_null() || pv.is_null() || n < 0 {
        return error(ERR_BAD_PARAMETER);
    }
    let d3d = N == 3 && c.flags & FACESIND3DFORMAT != 0;
    let fs = unsafe { faces::<N>(pf, n as usize, d3d) };
    // Deviation: a face that names a vertex past MAX_VERTICES is skipped.
    let count = fs.iter().flatten().copied().filter(|&i| i < MAX_VERTICES).max().map_or(0, |m| m as usize + 1);
    let vb = unsafe { std::slice::from_raw_parts(pv, count * VERTEX_SIZE) };
    let vs: Vec<Vertex> = vb.chunks_exact(VERTEX_SIZE).map(Vertex::read).collect();
    let mut s = sgl();
    let translucent_map = s.scene.textures.get(&(c.texture as u32)).is_some_and(|t| t.translucent);
    s.tri.add(&c, &fs, &vs, translucent_map);
    drop(s);
    shim::log_once!("{name}: first call, {n} faces, flags {:#x}, texture {}", c.flags, c.texture);
    error(0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_startofframe(ctx: *mut u8) {
    trace!("sgltri_startofframe", "{ctx:?}");
    let Some(c) = (unsafe { context(ctx) }) else { return error(ERR_BAD_PARAMETER) };
    let mut s = sgl();
    s.tri.start(&c);
    // The internal fields SGL fills in: the regions the device covers and their inverse size
    // (32 × 32 pixel regions, as `sglhw.ini` gives them).
    let (w, h) = s.device.unwrap_or((640, 480));
    let words: [u32; 6] = [0, ((w + 31) / 32 - 1) as u32, 0, ((h + 31) / 32 - 1) as u32, (1.0f32 / 32.0).to_bits(), (1.0f32 / 32.0).to_bits()];
    for (i, v) in words.iter().enumerate() {
        unsafe { ctx.add(0x48 + i * 4).cast::<u32>().write_unaligned(*v) };
    }
    error(0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_triangles(ctx: *const u8, n: i32, faces: *const u8, verts: *const u8) {
    trace!("sgltri_triangles", "{ctx:?} {n} {faces:?} {verts:?}");
    unsafe { add::<3>("sgltri_triangles", ctx, n, faces, verts) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_quads(ctx: *const u8, n: i32, faces: *const u8, verts: *const u8) {
    trace!("sgltri_quads", "{ctx:?} {n} {faces:?} {verts:?}");
    unsafe { add::<4>("sgltri_quads", ctx, n, faces, verts) };
}

/// Deviation: shadow and light volumes given as triangles are counted, not drawn.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_shadow(ctx: *const u8, n: i32, faces: *const u8, verts: *const u8, _bbox: *const f32) {
    trace!("sgltri_shadow", "{ctx:?} {n} {faces:?} {verts:?}");
    if ctx.is_null() || faces.is_null() || verts.is_null() || n < 0 {
        return error(ERR_BAD_PARAMETER);
    }
    sgl().tri.shadows += n as u32;
    shim::log_once!("sgltri_shadow: {n} faces, not drawn");
    error(0);
}

fn show(view: View) {
    let dev = sgl().device.unwrap_or((640, 480));
    let e = crate::immediate::render(view, dev);
    error(e);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_render(ctx: *const u8) {
    trace!("sgltri_render", "{ctx:?}");
    let Some(c) = (unsafe { context(ctx) }) else { return error(ERR_BAD_PARAMETER) };
    let view = {
        let mut s = sgl();
        let dev = s.device.unwrap_or((640, 480));
        let view = s.tri.view(&c, dev);
        if s.frames == 0 {
            shim::log!("first immediate-mode frame: {} faces sent, {} drawn, {} batches", s.tri.sent, s.tri.kept, view.batches.len());
        }
        s.frames += 1;
        s.last_tri = Some(view.clone());
        view
    };
    show(view);
}

/// Draw the last frame again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_rerender(ctx: *const u8) {
    trace!("sgltri_rerender", "{ctx:?}");
    let view = sgl().last_tri.clone();
    match view {
        Some(v) => show(v),
        None => error(ERR_BAD_PARAMETER),
    }
}

/// 0, `IRC_RENDER_COMPLETE`: the frame went to the render thread when `sgltri_render` returned.
#[unsafe(no_mangle)]
pub extern "C" fn sgltri_isrendercomplete(ctx: *const u8, timeout: u32) -> i32 {
    trace!("sgltri_isrendercomplete", "{ctx:?} {timeout}");
    0
}
