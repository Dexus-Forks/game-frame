//! Render state calls: the colour and alpha combiners (and the `gu` shorthands for them),
//! blending, depth, chroma key and fog.

use crate::cmd::{Cmd, push};
use crate::consts::*;

#[unsafe(no_mangle)]
pub extern "system" fn grColorCombine(func: u32, factor: u32, local: u32, other: u32, invert: u32) {
    push(Cmd::ColorCombine([func, factor, local, other, invert]));
}

/// `guColorCombineFunction`: each `GR_COLORCOMBINE_*` as the `grColorCombine` call it stands
/// for (the Glide 2.x reference manual's table).
#[unsafe(no_mangle)]
pub extern "system" fn guColorCombineFunction(f: u32) {
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
}

/// `guAlphaSource`: POD calls it with `GR_ALPHASOURCE_ITERATED_ALPHA` after every
/// `grSstOpen`; `grAlphaCombine` itself is not imported.
#[unsafe(no_mangle)]
pub extern "system" fn guAlphaSource(mode: u32) {
    let c = match mode {
        ALPHASOURCE_CC_ALPHA => [FUNC_LOCAL, FACTOR_ZERO, LOCAL_CONSTANT, OTHER_CONSTANT, 0],
        ALPHASOURCE_ITERATED_ALPHA => [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
        ALPHASOURCE_TEXTURE_ALPHA => [FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE, 0],
        ALPHASOURCE_TEXTURE_ALPHA_TIMES_ITERATED_ALPHA => [FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE, 0],
        _ => return,
    };
    push(Cmd::AlphaCombine(c));
}

/// Only matters for `GR_COMBINE_LOCAL_ITERATED` alpha-controlled lighting; POD passes 0.
#[unsafe(no_mangle)]
pub extern "system" fn grAlphaControlsITRGBLighting(enable: u32) {
    if enable != 0 {
        crate::log::log_once!("grAlphaControlsITRGBLighting(TRUE) is not handled");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grConstantColorValue(c: u32) {
    push(Cmd::ConstantColor(c));
}

#[unsafe(no_mangle)]
pub extern "system" fn grAlphaBlendFunction(rgb_sf: u32, rgb_df: u32, alpha_sf: u32, alpha_df: u32) {
    push(Cmd::AlphaBlend([rgb_sf, rgb_df, alpha_sf, alpha_df]));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthBufferMode(mode: u32) {
    push(Cmd::DepthMode(mode));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthBufferFunction(f: u32) {
    push(Cmd::DepthFunc(f));
}

#[unsafe(no_mangle)]
pub extern "system" fn grDepthMask(enable: u32) {
    push(Cmd::DepthMask(enable != 0));
}

#[unsafe(no_mangle)]
pub extern "system" fn grChromakeyMode(mode: u32) {
    push(Cmd::ChromaMode(mode != 0));
}

#[unsafe(no_mangle)]
pub extern "system" fn grChromakeyValue(c: u32) {
    push(Cmd::ChromaValue(c));
}

#[unsafe(no_mangle)]
pub extern "system" fn grFogMode(mode: u32) {
    push(Cmd::FogMode(mode));
}

#[unsafe(no_mangle)]
pub extern "system" fn grFogColorValue(c: u32) {
    push(Cmd::FogColor(c));
}

/// `GR_FOG_TABLE_SIZE` (64) bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grFogTable(table: *const u8) {
    if table.is_null() {
        return;
    }
    let mut t = Box::new([0u8; 64]);
    unsafe { std::ptr::copy_nonoverlapping(table, t.as_mut_ptr(), 64) };
    push(Cmd::FogTable(t));
}
