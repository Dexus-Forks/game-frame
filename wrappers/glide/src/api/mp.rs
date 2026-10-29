//! The `gu` multi-pass texturing calls: two textures on one TMU, drawn as two passes of the
//! same triangle and blended in the frame buffer.
//!
//! Read from the original's code (*measured*, the Win95 `glide2x.dll` of Glide 2.43, at
//! 0x10004190 to 0x10005a94): `guMPInit`, `guMPTexSource` and `guMPTexCombineFunction` only
//! store their arguments. `guMPDrawTriangle` selects each texture with `guTexSource` and
//! calls `grDrawTriangle` with the same three vertices for each pass. So both passes read
//! the vertices' TMU 0 coordinates. Between the passes it rewrites the blend, depth, fog and
//! colour path registers, and at the end it puts those four back. The second texture stays
//! selected.

use super::draw::grDrawTriangle;
use super::gutex::guTexSource;
use super::shadow::{self, Shadow, send};
use crate::cmd::{Cmd, Vertex};
use crate::consts::*;
use std::sync::Mutex;

/// `GrMPTextureCombineFnc_t`.
const ADD: u32 = 0;
const MULTIPLY: u32 = 1;
const SUBTRACT: u32 = 6;
/// `GR_CMP_NEVER`.
const CMP_NEVER: u32 = 0;
const TEX_DECAL: u32 = 1;

struct Mp {
    ids: [i32; 2],
    func: u32,
}

static MP: Mutex<Mp> = Mutex::new(Mp { ids: [-1, -1], func: ADD });

fn with<R>(f: impl FnOnce(&mut Mp) -> R) -> R {
    f(&mut MP.lock().unwrap_or_else(|e| e.into_inner()))
}

#[unsafe(no_mangle)]
pub extern "system" fn guMPInit() {
    shim::call!("guMPInit");
    with(|m| *m = Mp { ids: [-1, -1], func: ADD });
}

/// Texture `id` (from `guTexAllocateMemory`) for pass 0 or pass 1.
#[unsafe(no_mangle)]
pub extern "system" fn guMPTexSource(pass: u32, id: i32) {
    shim::call!("guMPTexSource", pass, id);
    // The original stores without a check; only 0 and 1 are read.
    if pass < 2 {
        with(|m| m.ids[pass as usize] = id);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn guMPTexCombineFunction(f: u32) {
    shim::call!("guMPTexCombineFunction", f);
    with(|m| m.func = f);
}

/// The colour combine with its local colour no longer added or subtracted: the original
/// clears those three bits of the colour path for these passes. What is left of each
/// function is the scaled other colour, or zero where the other colour was zeroed.
fn without_local(mut cc: [u32; 5]) -> [u32; 5] {
    cc[0] = match cc[0] {
        FUNC_ZERO | FUNC_LOCAL | FUNC_LOCAL_ALPHA | FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL | FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA => FUNC_ZERO,
        _ => FUNC_SCALE_OTHER,
    };
    cc
}

/// The second and later passes: the depth test as the original sets it. It clears the depth
/// function to 0, which is `GR_CMP_NEVER`, and the depth writes, and sets bit 1 of the mode
/// register (the chroma key; no pixel reaches it). So while depth buffering is on, these
/// passes draw nothing. This is read from the code, not seen on a card.
fn later_pass_depth(s: &Shadow) {
    if s.depth_mode != DEPTHBUFFER_DISABLE {
        send(Cmd::DepthFunc(CMP_NEVER));
        send(Cmd::DepthMask(false));
    }
}

fn fogged(s: &Shadow) -> bool {
    s.fog_mode & 3 != 0
}

/// The original reports these and draws on.
fn check(s: &Shadow) {
    let blending = s.blend != [BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO];
    if blending || s.alpha_test[0] != 7 || s.chroma != 0 {
        shim::gap!("guMPDrawTriangle", "illegal state: blending, the alpha test or the chroma key is on");
    }
}

/// Draw a triangle with the two textures `guMPTexSource` named, combined by the function
/// `guMPTexCombineFunction` set.
///
/// - `MULTIPLY`: pass 1 draws texture 0 with the colour combine less its local part, and
///   without the fog colour. Pass 2 draws texture 1 as it is, multiplied into pass 1
///   (source factor: destination colour). With fog on, pass 3 adds the fog colour by the
///   fog amount.
/// - `ADD`, and the detail and trilinear ones: pass 1 draws texture 0 with the game's state.
///   Pass 2 adds texture 1 with the colour combine less its local part, and without the fog
///   colour.
/// - `SUBTRACT`: the original reports that it is not implemented and draws nothing. Other
///   values draw nothing.
///
/// Original: `guMPDrawTriangle` (glide2x.dll 2.43, 0x100056d0).
///
/// Deviation: the detail and trilinear functions scale each pass's texel by the level of
/// detail fraction in the TMU's texture combiner. Here only texture 0 is drawn, once: the
/// nearest level's texture without the detail (`grTexDetailControl` is ignored too).
/// Deviation: the original sets the texture combiner to decal for both passes and leaves it
/// so. Here the game's texture combine function is put back with the rest.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guMPDrawTriangle(a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    shim::call!("guMPDrawTriangle", crate::callargs::V(a), crate::callargs::V(b), crate::callargs::V(c));
    let (ids, func) = with(|m| (m.ids, m.func));
    let s = shadow::get();
    check(&s);
    match func {
        MULTIPLY => unsafe { multiply(ids, &s, a, b, c) },
        ADD => unsafe { add(ids, &s, a, b, c) },
        2..=5 => {
            guTexSource(ids[0]);
            unsafe { grDrawTriangle(a, b, c) };
        }
        SUBTRACT => shim::gap!("guMPDrawTriangle", "GR_MPTEXCOMBINE_SUBTRACT is not implemented in Glide 2.43 either: nothing drawn"),
        _ => {}
    }
}

unsafe fn multiply(ids: [i32; 2], s: &Shadow, a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    guTexSource(ids[0]);
    send(Cmd::TexCombine(TEX_DECAL));
    send(Cmd::ColorCombine(without_local(s.cc)));
    // Fog without its colour: the pass is darkened by the fog amount.
    send(Cmd::FogColor(0));
    unsafe { grDrawTriangle(a, b, c) };

    guTexSource(ids[1]);
    send(Cmd::TexCombine(TEX_DECAL));
    send(Cmd::ColorCombine([FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0]));
    send(Cmd::AlphaBlend([BLEND_SRC_COLOR, BLEND_ZERO, BLEND_ONE, BLEND_ZERO]));
    later_pass_depth(s);
    send(Cmd::FogMode(0));
    unsafe { grDrawTriangle(a, b, c) };

    if fogged(s) {
        // Only the fog colour, by the fog amount, added.
        send(Cmd::ColorCombine([FUNC_ZERO, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0]));
        send(Cmd::AlphaBlend([BLEND_ONE, BLEND_ONE, BLEND_ONE, BLEND_ONE]));
        send(Cmd::FogMode(s.fog_mode));
        send(Cmd::FogColor(s.fog_color));
        unsafe { grDrawTriangle(a, b, c) };
    }
    restore(s);
}

unsafe fn add(ids: [i32; 2], s: &Shadow, a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    guTexSource(ids[0]);
    send(Cmd::TexCombine(TEX_DECAL));
    unsafe { grDrawTriangle(a, b, c) };

    guTexSource(ids[1]);
    send(Cmd::TexCombine(TEX_DECAL));
    send(Cmd::AlphaBlend([BLEND_ONE, BLEND_ONE, BLEND_ONE, BLEND_ONE]));
    later_pass_depth(s);
    send(Cmd::FogColor(0));
    send(Cmd::ColorCombine(without_local(s.cc)));
    unsafe { grDrawTriangle(a, b, c) };
    restore(s);
}

/// Put back what the passes changed, as the game set it.
fn restore(s: &Shadow) {
    send(Cmd::AlphaBlend(s.blend));
    send(Cmd::DepthFunc(s.depth_func));
    send(Cmd::DepthMask(s.depth_mask != 0));
    send(Cmd::FogMode(s.fog_mode));
    send(Cmd::FogColor(s.fog_color));
    send(Cmd::ColorCombine(s.cc));
    send(Cmd::TexCombine(s.tex_fn));
}
