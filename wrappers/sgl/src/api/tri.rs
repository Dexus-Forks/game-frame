//! The immediate-mode calls (`sgltri_*`, sgl.h). The game sends faces already on the screen
//! each frame, under a context. The calls read the game's `SGLCONTEXT`, faces and `SGLVERTEX`
//! array. They add to the frame in `scene::tri`. `sgltri_render` hands the frame to
//! `immediate`. Each call reports the errors sgl.h lists for it. `sgl_get_errors` reads them
//! back.

use super::{error, sgl, trace};
use crate::scene::shadow::shadow_args;
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

/// The vertices up to the highest index the faces name. Deviation: a face that names a vertex
/// past MAX_VERTICES is skipped.
unsafe fn vertices<const N: usize>(fs: &[[u32; N]], pv: *const u8) -> Vec<Vertex> {
    let count = fs.iter().flatten().copied().filter(|&i| i < MAX_VERTICES).max().map_or(0, |m| m as usize + 1);
    let vb = unsafe { std::slice::from_raw_parts(pv, count * VERTEX_SIZE) };
    vb.chunks_exact(VERTEX_SIZE).map(Vertex::read).collect()
}

/// For the call log (`WRAPPER_CALLS`): an `SGLCONTEXT` as the fields this library reads.
struct Ctx(*const u8);
impl shim::calls::Arg for Ctx {
    fn put(&self, out: &mut String) {
        match unsafe { context(self.0) } {
            Some(c) => out.push_str(&format!("{c:?}")),
            None => out.push_str("null"),
        }
    }
}

#[derive(Clone, Copy)]
/// For the call log: `n` faces of `N` vertices, each written as its vertices' values.
struct Faces<const N: usize>(*const u8, i32, *const u8, *const u8);
impl<const N: usize> shim::calls::Arg for Faces<N> {
    fn put(&self, out: &mut String) {
        let Faces(ctx, n, pf, pv) = *self;
        let Some(c) = (unsafe { context(ctx) }) else { return out.push_str("null") };
        if pf.is_null() || pv.is_null() || n <= 0 {
            return out.push_str("[]");
        }
        let fs = unsafe { faces::<N>(pf, n as usize, N == 3 && c.flags & FACESIND3DFORMAT != 0) };
        let vs = unsafe { vertices(&fs, pv) };
        out.push('[');
        for (k, f) in fs.iter().enumerate() {
            out.push_str(if k == 0 { "[" } else { " [" });
            for (j, i) in f.iter().enumerate() {
                if j > 0 {
                    out.push(' ');
                }
                match vs.get(*i as usize) {
                    Some(v) => out.push_str(&format!("{v:?}")),
                    None => out.push_str(&format!("#{i}")),
                }
            }
            out.push(']');
        }
        out.push(']');
    }
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
    let vs = unsafe { vertices(&fs, pv) };
    let mut s = sgl();
    let translucent_map = s.scene.textures.get(&(c.texture as u32)).is_some_and(|t| t.translucent);
    s.tri.add(&c, &fs, &vs, translucent_map);
    drop(s);
    shim::log_once!("{name}: first call, {n} faces, flags {:#x}, texture {}", c.flags, c.texture);
    error(0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_startofframe(ctx: *mut u8) {
    shim::call!("sgltri_startofframe", ctx);
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
    shim::call!("sgltri_triangles", Ctx(ctx), n, Faces::<3>(ctx, n, faces, verts));
    trace!("sgltri_triangles", "{ctx:?} {n} {faces:?} {verts:?}");
    unsafe { add::<3>("sgltri_triangles", ctx, n, faces, verts) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_quads(ctx: *const u8, n: i32, faces: *const u8, verts: *const u8) {
    shim::call!("sgltri_quads", Ctx(ctx), n, Faces::<4>(ctx, n, faces, verts));
    trace!("sgltri_quads", "{ctx:?} {n} {faces:?} {verts:?}");
    unsafe { add::<4>("sgltri_quads", ctx, n, faces, verts) };
}

/// One convex shadow or light volume, from the planes of up to 100 faces (`int[3]` each),
/// limited to the screen regions `bbox` (`{x0, y0, x1, y1}`) covers (Original: `sgltri_shadow`
/// at `10003080`). The arguments are checked as the driver checks them
/// (`scene::shadow::shadow_args`). The volume is drawn with the frame (`scene::shadow`).
/// Deviation: `SGLTT_FACESIND3DFORMAT` is not read here; whether the driver reads it for
/// shadows was not traced. The bounding box is not used (`Frame::shadow`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_shadow(ctx: *const u8, n: i32, faces: *const u8, verts: *const u8, _bbox: *const f32) {
    shim::call!("sgltri_shadow", Ctx(ctx), n, Faces::<3>(ctx, n, faces, verts), _bbox);
    trace!("sgltri_shadow", "{ctx:?} {n} {faces:?} {verts:?}");
    let c = unsafe { context(ctx) };
    if let Some(e) = shadow_args(n, c.map(|c| c.shadow_mode), !faces.is_null() && !verts.is_null()) {
        return error(e);
    }
    let Some(c) = c else { return error(ERR_BAD_PARAMETER) };
    let fs = unsafe { self::faces::<3>(faces, n as usize, false) };
    let vs = unsafe { vertices(&fs, verts) };
    sgl().tri.shadow(&c, &fs, &vs);
    shim::log_once!("sgltri_shadow: first call, {n} faces, mode {}", c.shadow_mode);
    error(0);
}

fn show(view: View) {
    let dev = sgl().device.unwrap_or((640, 480));
    let e = crate::immediate::render(view, dev);
    error(e);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_render(ctx: *const u8) {
    shim::call!("sgltri_render", Ctx(ctx));
    trace!("sgltri_render", "{ctx:?}");
    let Some(c) = (unsafe { context(ctx) }) else { return error(ERR_BAD_PARAMETER) };
    let view = {
        let mut s = sgl();
        let view = s.tri.view(&c);
        if s.frames == 0 {
            shim::log!("first immediate-mode frame: {} faces sent, {} drawn, {} batches", s.tri.sent, s.tri.kept, view.batches.len());
        }
        s.frames += 1;
        s.last_tri = Some(view.clone());
        view
    };
    show(view);
    shim::calls::end_frame();
}

/// Draw the last frame again. The driver ignores the context and restarts the chip on the
/// display list it already built; with no earlier frame that draws nothing and still reports
/// error 0 (Original: `sgltri_rerender` at `10032430`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgltri_rerender(ctx: *const u8) {
    shim::call!("sgltri_rerender", ctx);
    trace!("sgltri_rerender", "{ctx:?}");
    let view = sgl().last_tri.clone();
    match view {
        Some(v) => show(v),
        None => error(0),
    }
}

/// 0, `IRC_RENDER_COMPLETE`: the frame went to the render thread when `sgltri_render` returned.
#[unsafe(no_mangle)]
pub extern "C" fn sgltri_isrendercomplete(ctx: *const u8, timeout: u32) -> i32 {
    shim::call!("sgltri_isrendercomplete", ctx, timeout);
    trace!("sgltri_isrendercomplete", "{ctx:?} {timeout}");
    0
}
