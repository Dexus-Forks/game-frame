//! The Glide 2.x constants this library interprets, with the values of the Glide 2.x SDK's
//! `glide.h`. Texture formats and sizes are `gpu_1997::glide`.

#![allow(dead_code)]

// GrCombineFunction_t
pub const FUNC_ZERO: u32 = 0x0;
pub const FUNC_LOCAL: u32 = 0x1;
pub const FUNC_LOCAL_ALPHA: u32 = 0x2;
pub const FUNC_SCALE_OTHER: u32 = 0x3;
pub const FUNC_SCALE_OTHER_ADD_LOCAL: u32 = 0x4;
pub const FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA: u32 = 0x5;
pub const FUNC_SCALE_OTHER_MINUS_LOCAL: u32 = 0x6;
pub const FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL: u32 = 0x7;
pub const FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA: u32 = 0x8;
pub const FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL: u32 = 0x9;
pub const FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA: u32 = 0x10;

// GrCombineFactor_t
pub const FACTOR_ZERO: u32 = 0x0;
pub const FACTOR_LOCAL: u32 = 0x1;
pub const FACTOR_LOCAL_ALPHA: u32 = 0x3;
pub const FACTOR_TEXTURE_ALPHA: u32 = 0x4;
pub const FACTOR_ONE: u32 = 0x8;

// GrCombineLocal_t, GrCombineOther_t
pub const LOCAL_ITERATED: u32 = 0x0;
pub const LOCAL_CONSTANT: u32 = 0x1;
pub const OTHER_ITERATED: u32 = 0x0;
pub const OTHER_TEXTURE: u32 = 0x1;
pub const OTHER_CONSTANT: u32 = 0x2;

// GrColorCombineFnc_t (guColorCombineFunction)
pub const CC_ZERO: u32 = 0x0;
pub const CC_CCRGB: u32 = 0x1;
pub const CC_ITRGB: u32 = 0x2;
pub const CC_ITRGB_DELTA0: u32 = 0x3;
pub const CC_DECAL_TEXTURE: u32 = 0x4;
pub const CC_TEXTURE_TIMES_CCRGB: u32 = 0x5;
pub const CC_TEXTURE_TIMES_ITRGB: u32 = 0x6;
pub const CC_TEXTURE_TIMES_ITRGB_DELTA0: u32 = 0x7;
pub const CC_TEXTURE_TIMES_ITRGB_ADD_ALPHA: u32 = 0x8;
pub const CC_TEXTURE_TIMES_ALPHA: u32 = 0x9;
pub const CC_TEXTURE_TIMES_ALPHA_ADD_ITRGB: u32 = 0xa;
pub const CC_TEXTURE_ADD_ITRGB: u32 = 0xb;
pub const CC_TEXTURE_SUB_ITRGB: u32 = 0xc;
pub const CC_CCRGB_BLEND_ITRGB_ON_TEXALPHA: u32 = 0xd;
pub const CC_DIFF_SPEC_A: u32 = 0xe;
pub const CC_DIFF_SPEC_B: u32 = 0xf;
pub const CC_ONE: u32 = 0x10;

// GrAlphaSource_t (guAlphaSource)
pub const ALPHASOURCE_CC_ALPHA: u32 = 0x0;
pub const ALPHASOURCE_ITERATED_ALPHA: u32 = 0x1;
pub const ALPHASOURCE_TEXTURE_ALPHA: u32 = 0x2;
pub const ALPHASOURCE_TEXTURE_ALPHA_TIMES_ITERATED_ALPHA: u32 = 0x3;

// GrAlphaBlendFnc_t
pub const BLEND_ZERO: u32 = 0x0;
pub const BLEND_SRC_ALPHA: u32 = 0x1;
pub const BLEND_SRC_COLOR: u32 = 0x2; // DST_COLOR as a source factor
pub const BLEND_DST_ALPHA: u32 = 0x3;
pub const BLEND_ONE: u32 = 0x4;
pub const BLEND_ONE_MINUS_SRC_ALPHA: u32 = 0x5;
pub const BLEND_ONE_MINUS_SRC_COLOR: u32 = 0x6; // ONE_MINUS_DST_COLOR as a source factor
pub const BLEND_ONE_MINUS_DST_ALPHA: u32 = 0x7;
pub const BLEND_ALPHA_SATURATE: u32 = 0xf; // PREFOG_COLOR as a destination factor

// GrDepthBufferMode_t
pub const DEPTHBUFFER_DISABLE: u32 = 0x0;
pub const DEPTHBUFFER_ZBUFFER: u32 = 0x1;
pub const DEPTHBUFFER_WBUFFER: u32 = 0x2;

// GrFogMode_t
pub const FOG_WITH_TABLE: u32 = 0x2;

// GrTextureFilterMode_t, GrTextureClampMode_t
pub const FILTER_BILINEAR: u32 = 0x1;
pub const CLAMP_CLAMP: u32 = 0x1;

// GrBuffer_t
pub const BUFFER_FRONT: u32 = 0;
pub const BUFFER_BACK: u32 = 1;
pub const BUFFER_AUX: u32 = 2;
pub const BUFFER_DEPTH: u32 = 3;

// GrLfbWriteMode_t
pub const LFB_565: u32 = 0x0;
pub const LFB_555: u32 = 0x1;
pub const LFB_1555: u32 = 0x2;
pub const LFB_888: u32 = 0x4;
pub const LFB_8888: u32 = 0x5;

// GrLfbBypassMode_t
pub const LFBBYPASS_ENABLE: u32 = 1;

/// `GrScreenResolution_t` to pixels; `None` for values this library does not know.
pub fn resolution(res: u32) -> Option<(u32, u32)> {
    Some(match res {
        0x0 => (320, 200),
        0x1 => (320, 240),
        0x2 => (400, 256),
        0x3 => (512, 384),
        0x4 => (640, 200),
        0x5 => (640, 350),
        0x6 => (640, 400),
        0x7 => (640, 480),
        0x8 => (800, 600),
        0x9 => (960, 720),
        0xa => (856, 480),
        0xb => (512, 256),
        0xc => (1024, 768),
        0xd => (1280, 1024),
        0xe => (1600, 1200),
        0xf => (400, 300),
        _ => return None,
    })
}
