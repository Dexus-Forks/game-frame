//! The Glide 2.x constants the pipeline reads, with the values of the Glide 2.x SDK's
//! `glide.h` (the same as GameFrame's `wrappers/glide/src/consts.rs`). Texture formats and
//! sizes are `gameframe_gpu_1997::glide`.

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
pub const FACTOR_OTHER_ALPHA: u32 = 0x2;
pub const FACTOR_LOCAL_ALPHA: u32 = 0x3;
pub const FACTOR_TEXTURE_ALPHA: u32 = 0x4;
pub const FACTOR_ONE: u32 = 0x8;
pub const FACTOR_ONE_MINUS_LOCAL: u32 = 0x9;
pub const FACTOR_ONE_MINUS_OTHER_ALPHA: u32 = 0xa;
pub const FACTOR_ONE_MINUS_LOCAL_ALPHA: u32 = 0xb;
pub const FACTOR_ONE_MINUS_TEXTURE_ALPHA: u32 = 0xc;

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
pub const CC_ONE: u32 = 0xf;

// GrAlphaBlendFnc_t
pub const BLEND_ZERO: u32 = 0x0;
pub const BLEND_SRC_ALPHA: u32 = 0x1;
pub const BLEND_SRC_COLOR: u32 = 0x2;
pub const BLEND_DST_ALPHA: u32 = 0x3;
pub const BLEND_ONE: u32 = 0x4;
pub const BLEND_ONE_MINUS_SRC_ALPHA: u32 = 0x5;
pub const BLEND_ONE_MINUS_SRC_COLOR: u32 = 0x6;
pub const BLEND_ONE_MINUS_DST_ALPHA: u32 = 0x7;

// GrCmpFnc_t
pub const CMP_NEVER: u32 = 0x0;
pub const CMP_LESS: u32 = 0x1;
pub const CMP_EQUAL: u32 = 0x2;
pub const CMP_LEQUAL: u32 = 0x3;
pub const CMP_GREATER: u32 = 0x4;
pub const CMP_NOTEQUAL: u32 = 0x5;
pub const CMP_GEQUAL: u32 = 0x6;
pub const CMP_ALWAYS: u32 = 0x7;

// GrDepthBufferMode_t
pub const DEPTHBUFFER_DISABLE: u32 = 0x0;
pub const DEPTHBUFFER_ZBUFFER: u32 = 0x1;
pub const DEPTHBUFFER_WBUFFER: u32 = 0x2;

// GrFogMode_t
pub const FOG_DISABLE: u32 = 0x0;
pub const FOG_WITH_ITERATED_ALPHA: u32 = 0x1;
pub const FOG_WITH_TABLE: u32 = 0x2;

// GrTextureCombineFnc_t (guTexCombineFunction)
pub const TEXTURECOMBINE_ZERO: u32 = 0x0;
pub const TEXTURECOMBINE_DECAL: u32 = 0x1;
pub const TEXTURECOMBINE_ONE: u32 = 0xa;

// GrTextureClampMode_t, GrTextureFilterMode_t
pub const TEXTURECLAMP_WRAP: u32 = 0x0;
pub const TEXTURECLAMP_CLAMP: u32 = 0x1;
pub const TEXTUREFILTER_POINT_SAMPLED: u32 = 0x0;
pub const TEXTUREFILTER_BILINEAR: u32 = 0x1;
