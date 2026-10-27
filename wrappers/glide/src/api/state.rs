//! Render state calls: the colour and alpha combiners (and the `gu` shorthands for them),
//! blending, alpha test, depth, masks, culling, chroma key and fog. Each goes through
//! `shadow::send`, which keeps the game's copy for `grGlideGetState`. Colours the game packs
//! are turned to 0xAARRGGBB with the colour format it opened with (`sst::argb`).

use super::shadow::{self, send};
use super::sst::argb;
use crate::cmd::Cmd;
use crate::consts::*;

#[unsafe(no_mangle)]
pub extern "system" fn grColorCombine(func: u32, factor: u32, local: u32, other: u32, invert: u32) {
    shim::call!("grColorCombine", func, factor, local, other, invert);
    shadow::set(|s| s.delta0 = 0);
    send(Cmd::ColorCombine([func, factor, local, other, invert]));
}

/// `guColorCombineFunction`: each `GR_COLORCOMBINE_*` as the `grColorCombine` call it stands
/// for (the Glide 2.x reference manual's table).
#[unsafe(no_mangle)]
pub extern "system" fn guColorCombineFunction(f: u32) {
    shim::call!("guColorCombineFunction", f);
    let (func, factor, local, other, invert) = match f {
        CC_ZERO => (FUNC_ZERO, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, 0),
        CC_CCRGB => (FUNC_LOCAL, FACTOR_ZERO, LOCAL_CONSTANT, OTHER_CONSTANT, 0),
        CC_ITRGB | CC_ITRGB_DELTA0 => (FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0),
        CC_DECAL_TEXTURE => (FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_TEXTURE_TIMES_CCRGB => (FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_CONSTANT, OTHER_TEXTURE, 0),
        CC_TEXTURE_TIMES_ITRGB | CC_TEXTURE_TIMES_ITRGB_DELTA0 => {
            (FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE, 0)
        }
        CC_TEXTURE_TIMES_ITRGB_ADD_ALPHA => (FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_TEXTURE_TIMES_ALPHA => (FUNC_SCALE_OTHER, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_TEXTURE_TIMES_ALPHA_ADD_ITRGB => (FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_TEXTURE_ADD_ITRGB => (FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_TEXTURE_SUB_ITRGB => (FUNC_SCALE_OTHER_MINUS_LOCAL, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_CCRGB_BLEND_ITRGB_ON_TEXALPHA => {
            (FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL, FACTOR_TEXTURE_ALPHA, LOCAL_ITERATED, OTHER_CONSTANT, 0)
        }
        CC_DIFF_SPEC_A => (FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_DIFF_SPEC_B => (FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, 0),
        CC_ONE => (FUNC_ZERO, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, 1),
        _ => {
            crate::log::log!("guColorCombineFunction {f}: unknown, ignored");
            return;
        }
    };
    grColorCombine(func, factor, local, other, invert);
    // The DELTA0 forms flat-shade with grConstantColorValue4's colour as the iterated one.
    let delta0 = matches!(f, CC_ITRGB_DELTA0 | CC_TEXTURE_TIMES_ITRGB_DELTA0);
    shadow::set(|s| s.delta0 = delta0 as u32);
}

/// `guAlphaSource`: POD calls it with `GR_ALPHASOURCE_ITERATED_ALPHA` after every
/// `grSstOpen`; `grAlphaCombine` itself is not imported.
#[unsafe(no_mangle)]
pub extern "system" fn guAlphaSource(mode: u32) {
    shim::call!("guAlphaSource", mode);
    let c = match mode {
        ALPHASOURCE_CC_ALPHA => [FUNC_LOCAL, FACTOR_ZERO, LOCAL_CONSTANT, OTHER_CONSTANT, 0],
        ALPHASOURCE_ITERATED_ALPHA => [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
        ALPHASOURCE_TEXTURE_ALPHA => [FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0],
        ALPHASOURCE_TEXTURE_ALPHA_TIMES_ITERATED_ALPHA => [FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE, 0],
        _ => return,
    };
    send(Cmd::AlphaCombine(c));
}

/// The alpha combiner. Deviation: `GR_COMBINE_LOCAL_DEPTH` (alpha from the depth value) is
/// taken as the iterated alpha.
#[unsafe(no_mangle)]
pub extern "system" fn grAlphaCombine(func: u32, factor: u32, local: u32, other: u32, invert: u32) {
    shim::call!("grAlphaCombine", func, factor, local, other, invert);
    if local == 2 {
        shim::gap!("grAlphaCombine local depth", "GR_COMBINE_LOCAL_DEPTH taken as iterated alpha");
    }
    send(Cmd::AlphaCombine([func, factor, if local == 2 { LOCAL_ITERATED } else { local }, other, invert]));
}

/// Only matters for `GR_COMBINE_LOCAL_ITERATED` alpha-controlled lighting; POD passes 0.
#[unsafe(no_mangle)]
pub extern "system" fn grAlphaControlsITRGBLighting(enable: u32) {
    shim::call!("grAlphaControlsITRGBLighting", enable);
    if enable != 0 {
        shim::gap!("grAlphaControlsITRGBLighting", "TRUE is not handled");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grConstantColorValue(c: u32) {
    shim::call!("grConstantColorValue", c);
    send(Cmd::ConstantColor(argb(c)));
}

#[unsafe(no_mangle)]
pub extern "system" fn grAlphaBlendFunction(rgb_sf: u32, rgb_df: u32, alpha_sf: u32, alpha_df: u32) {
    shim::call!("grAlphaBlendFunction", rgb_sf, rgb_df, alpha_sf, alpha_df);
    send(Cmd::AlphaBlend([rgb_sf, rgb_df, alpha_sf, alpha_df]));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthBufferMode(mode: u32) {
    shim::call!("grDepthBufferMode", mode);
    send(Cmd::DepthMode(mode));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthBufferFunction(f: u32) {
    shim::call!("grDepthBufferFunction", f);
    send(Cmd::DepthFunc(f));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthMask(enable: u32) {
    shim::call!("grDepthMask", enable);
    send(Cmd::DepthMask(enable != 0));
}

#[unsafe(no_mangle)]
pub extern "system" fn grChromakeyMode(mode: u32) {
    shim::call!("grChromakeyMode", mode);
    send(Cmd::ChromaMode(mode != 0));
}

#[unsafe(no_mangle)]
pub extern "system" fn grChromakeyValue(c: u32) {
    shim::call!("grChromakeyValue", c);
    send(Cmd::ChromaValue(argb(c)));
}

#[unsafe(no_mangle)]
pub extern "system" fn grFogMode(mode: u32) {
    shim::call!("grFogMode", mode);
    send(Cmd::FogMode(mode));
}

#[unsafe(no_mangle)]
pub extern "system" fn grFogColorValue(c: u32) {
    shim::call!("grFogColorValue", c);
    send(Cmd::FogColor(argb(c)));
}

/// `GR_FOG_TABLE_SIZE` (64) bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grFogTable(table: *const u8) {
    shim::call!("grFogTable", crate::callargs::Data(table, 64));
    if table.is_null() {
        return;
    }
    let mut t = Box::new([0u8; 64]);
    unsafe { std::ptr::copy_nonoverlapping(table, t.as_mut_ptr(), 64) };
    send(Cmd::FogTable(t));
}

/// `GrCmpFnc_t`, the same numbers as the depth compare.
#[unsafe(no_mangle)]
pub extern "system" fn grAlphaTestFunction(f: u32) {
    shim::call!("grAlphaTestFunction", f);
    let r = shadow::get().alpha_test[1];
    send(Cmd::AlphaTest([f & 7, r]));
}

#[unsafe(no_mangle)]
pub extern "system" fn grAlphaTestReferenceValue(v: u32) {
    shim::call!("grAlphaTestReferenceValue", v);
    let f = shadow::get().alpha_test[0];
    send(Cmd::AlphaTest([f, v & 0xff]));
}

#[unsafe(no_mangle)]
pub extern "system" fn grColorMask(rgb: u32, alpha: u32) {
    shim::call!("grColorMask", rgb, alpha);
    send(Cmd::ColorMask(rgb != 0, alpha != 0));
}

/// `GrCullMode_t`: tested by the drawing calls before a triangle is recorded.
#[unsafe(no_mangle)]
pub extern "system" fn grCullMode(mode: u32) {
    shim::call!("grCullMode", mode);
    shadow::set(|s| s.cull = mode);
}

/// A signed 16-bit level added to each pixel's depth.
#[unsafe(no_mangle)]
pub extern "system" fn grDepthBiasLevel(level: i32) {
    shim::call!("grDepthBiasLevel", level);
    send(Cmd::DepthBias(level as i16 as i32));
}

/// Deviation: nothing is dithered. The frame buffer has 8 bits per channel, more than the
/// Voodoo's 16-bit one that dithering hid the banding of.
#[unsafe(no_mangle)]
pub extern "system" fn grDitherMode(mode: u32) {
    shim::call!("grDitherMode", mode);
    shadow::set(|s| s.dither = mode);
}

/// `grConstantColorValue4(a, r, g, b)`, each 0–255: the colour a `*_DELTA0` colour combine
/// draws with in place of the iterated one.
#[unsafe(no_mangle)]
pub extern "system" fn grConstantColorValue4(a: f32, r: f32, g: f32, b: f32) {
    shim::call!("grConstantColorValue4", a, r, g, b);
    shadow::set(|s| s.color4 = [a, r, g, b]);
}

/// Blending, alpha test, chroma key, depth buffering and fog off (the Glide 2.x reference
/// manual).
#[unsafe(no_mangle)]
pub extern "system" fn grDisableAllEffects() {
    shim::call!("grDisableAllEffects");
    grAlphaBlendFunction(BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO);
    grAlphaTestFunction(7);
    grChromakeyMode(0);
    grDepthBufferMode(DEPTHBUFFER_DISABLE);
    grFogMode(0);
}

/// `GR_HINT_STWHINT` says which TMUs have their own *w*. Its other hints (FIFO checks, FPU
/// precision, mipmap dither) have nothing to tune here.
#[unsafe(no_mangle)]
pub extern "system" fn grHints(kind: u32, flags: u32) {
    shim::call!("grHints", kind, flags);
    if kind == HINT_STWHINT {
        shadow::set(|s| s.stw_hint = flags);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grRenderBuffer(buffer: u32) {
    shim::call!("grRenderBuffer", buffer);
    send(Cmd::RenderBuffer(if buffer == BUFFER_FRONT { BUFFER_FRONT } else { BUFFER_BACK }));
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grGlideGetState(state: *mut u8) {
    shim::call!("grGlideGetState", state);
    if !state.is_null() {
        unsafe { shadow::save(state) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grGlideSetState(state: *const u8) {
    shim::call!("grGlideSetState", state);
    if !state.is_null() {
        unsafe { shadow::restore(state) };
    }
}
