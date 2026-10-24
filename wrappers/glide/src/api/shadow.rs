//! The render state as the game has set it, kept on the game's side as well as the render
//! thread's. `grGlideGetState` copies it out and `grGlideSetState` sends it all again. The
//! drawing calls read the parts that act before a triangle is recorded: the origin, culling,
//! the delta-0 colour, the STW hint, and the clip window that `gu…WithClip` clips to.
//!
//! A `GrState` is 256 bytes in Glide 2.43: `grGlideGetState` copies 0x40 words (*measured*,
//! the Win95 `glide2x.dll` at 0x10004410). `Shadow` fits in it. The fog table is not part of
//! it, as 64 bytes of table would not fit beside the rest.

use crate::cmd::{Cmd, Source, TexInfo, push};
use crate::consts::*;
use std::sync::Mutex;

pub const GRSTATE_BYTES: usize = 256;
const MAGIC: u32 = 0x4746_5332; // "GFS2"

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    magic: u32,
    pub cc: [u32; 5],
    pub ac: [u32; 5],
    pub constant: u32,
    pub blend: [u32; 4],
    pub depth_mode: u32,
    pub depth_func: u32,
    pub depth_mask: u32,
    pub chroma: u32,
    pub key: u32,
    pub fog_mode: u32,
    pub fog_color: u32,
    /// The clip window as the game gave it, in its own origin.
    pub clip: [u32; 4],
    pub tex_fn: u32,
    pub filter: [u32; 2],
    pub clamp: [u32; 2],
    /// The last `grTexSource`: start, smallLod, largeLod, aspect, format; `source_set` is 1
    /// once there is one.
    pub source: [u32; 5],
    pub source_set: u32,
    pub alpha_test: [u32; 2],
    pub color_mask: [u32; 2],
    pub cull: u32,
    pub depth_bias: i32,
    pub render_buffer: u32,
    pub mipmap: [u32; 2],
    pub lod_bias: f32,
    pub multibase: u32,
    pub bases: [u32; 4],
    pub ncc: u32,
    pub origin: u32,
    /// The iterated colour is `color4` (`GR_COLORCOMBINE_*_DELTA0`).
    pub delta0: u32,
    /// `grConstantColorValue4`: a, r, g, b, 0–255.
    pub color4: [f32; 4],
    pub stw_hint: u32,
    pub dither: u32,
}

const _: () = assert!(std::mem::size_of::<Shadow>() <= GRSTATE_BYTES);

impl Shadow {
    const fn new() -> Shadow {
        Shadow {
            magic: MAGIC,
            cc: [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
            ac: [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
            constant: 0xffff_ffff,
            blend: [BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO],
            depth_mode: DEPTHBUFFER_DISABLE,
            depth_func: 1,
            depth_mask: 0,
            chroma: 0,
            key: 0,
            fog_mode: 0,
            fog_color: 0,
            clip: [0, 0, 640, 480],
            tex_fn: 1,
            filter: [FILTER_BILINEAR, FILTER_BILINEAR],
            clamp: [0, 0],
            source: [0; 5],
            source_set: 0,
            alpha_test: [7, 0],
            color_mask: [1, 1],
            cull: 0,
            depth_bias: 0,
            render_buffer: BUFFER_BACK,
            mipmap: [0, 0],
            lod_bias: 0.0,
            multibase: 0,
            bases: [0; 4],
            ncc: 0,
            origin: 0,
            delta0: 0,
            color4: [255.0; 4],
            stw_hint: 0,
            dither: 2,
        }
    }

    pub fn source(&self) -> Option<Source> {
        let [start, small_lod, large_lod, aspect, format] = self.source;
        (self.source_set != 0).then(|| Source {
            start,
            info: TexInfo { small_lod, large_lod, aspect, format },
            bases: (self.multibase != 0).then_some(self.bases),
            ncc: self.ncc as u8,
        })
    }
}

static SHADOW: Mutex<Shadow> = Mutex::new(Shadow::new());

pub fn get() -> Shadow {
    *SHADOW.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set<R>(f: impl FnOnce(&mut Shadow) -> R) -> R {
    f(&mut SHADOW.lock().unwrap_or_else(|e| e.into_inner()))
}

/// After `grSstOpen` the clip window covers the screen. The rest of the state carries over,
/// as the render thread's does.
pub fn reset(w: u32, h: u32) {
    set(|s| s.clip = [0, 0, w, h]);
}

/// Record a state command in the shadow, then send it.
pub fn send(c: Cmd) {
    set(|s| match &c {
        Cmd::ColorCombine(v) => s.cc = *v,
        Cmd::AlphaCombine(v) => s.ac = *v,
        Cmd::ConstantColor(v) => s.constant = *v,
        Cmd::AlphaBlend(v) => s.blend = *v,
        Cmd::DepthMode(v) => s.depth_mode = *v,
        Cmd::DepthFunc(v) => s.depth_func = *v,
        Cmd::DepthMask(v) => s.depth_mask = *v as u32,
        Cmd::ChromaMode(v) => s.chroma = *v as u32,
        Cmd::ChromaValue(v) => s.key = *v,
        Cmd::FogMode(v) => s.fog_mode = *v,
        Cmd::FogColor(v) => s.fog_color = *v,
        Cmd::TexCombine(v) => s.tex_fn = *v,
        Cmd::TexFilter(a, b) => s.filter = [*a, *b],
        Cmd::TexClamp(a, b) => s.clamp = [*a, *b],
        Cmd::AlphaTest(v) => s.alpha_test = *v,
        Cmd::ColorMask(r, a) => s.color_mask = [*r as u32, *a as u32],
        Cmd::DepthBias(v) => s.depth_bias = *v,
        Cmd::RenderBuffer(v) => s.render_buffer = *v,
        Cmd::MipMap(m, b) => s.mipmap = [*m, *b as u32],
        Cmd::LodBias(v) => s.lod_bias = *v,
        _ => {}
    });
    push(c);
}

/// The clip window in the render thread's origin (upper left).
fn clip_cmd(s: &Shadow) -> Cmd {
    let [x0, y0, x1, y1] = s.clip;
    if s.origin == ORIGIN_LOWER_LEFT {
        let h = crate::api::sst::screen().1;
        Cmd::Clip([x0, h.saturating_sub(y1), x1, h.saturating_sub(y0)])
    } else {
        Cmd::Clip(s.clip)
    }
}

pub fn clip(c: [u32; 4]) {
    let s = set(|s| {
        s.clip = c;
        *s
    });
    push(clip_cmd(&s));
}

pub fn origin(o: u32) {
    let s = set(|s| {
        s.origin = o;
        *s
    });
    push(clip_cmd(&s));
}

/// Select a texture: the shadow's source with the multibase addresses and NCC table now set.
pub fn source(start: u32, info: TexInfo) {
    let s = set(|s| {
        s.source = [start, info.small_lod, info.large_lod, info.aspect, info.format];
        s.source_set = 1;
        *s
    });
    if let Some(src) = s.source() {
        push(Cmd::TexSource(src));
    }
}

/// Send the source again after multibase or the NCC choice changed.
pub fn resend_source() {
    if let Some(src) = get().source() {
        push(Cmd::TexSource(src));
    }
}

/// `grGlideGetState`.
pub unsafe fn save(dst: *mut u8) {
    let s = get();
    unsafe {
        std::ptr::write_bytes(dst, 0, GRSTATE_BYTES);
        std::ptr::copy_nonoverlapping((&s as *const Shadow).cast::<u8>(), dst, std::mem::size_of::<Shadow>());
    }
}

/// `grGlideSetState`: every part sent again. A buffer this library did not fill is ignored.
pub unsafe fn restore(src: *const u8) {
    let s = unsafe { src.cast::<Shadow>().read_unaligned() };
    if s.magic != MAGIC {
        shim::gap!("grGlideSetState", "a GrState this library did not fill: ignored");
        return;
    }
    set(|d| *d = s);
    let b = |v: u32| v != 0;
    for c in [
        Cmd::ColorCombine(s.cc),
        Cmd::AlphaCombine(s.ac),
        Cmd::ConstantColor(s.constant),
        Cmd::AlphaBlend(s.blend),
        Cmd::DepthMode(s.depth_mode),
        Cmd::DepthFunc(s.depth_func),
        Cmd::DepthMask(b(s.depth_mask)),
        Cmd::ChromaMode(b(s.chroma)),
        Cmd::ChromaValue(s.key),
        Cmd::FogMode(s.fog_mode),
        Cmd::FogColor(s.fog_color),
        Cmd::TexCombine(s.tex_fn),
        Cmd::TexFilter(s.filter[0], s.filter[1]),
        Cmd::TexClamp(s.clamp[0], s.clamp[1]),
        Cmd::AlphaTest(s.alpha_test),
        Cmd::ColorMask(b(s.color_mask[0]), b(s.color_mask[1])),
        Cmd::DepthBias(s.depth_bias),
        Cmd::RenderBuffer(s.render_buffer),
        Cmd::MipMap(s.mipmap[0], b(s.mipmap[1])),
        Cmd::LodBias(s.lod_bias),
        clip_cmd(&s),
    ] {
        push(c);
    }
    resend_source();
}
