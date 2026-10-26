//! The Glide render state the pixel pipeline reads, set by the calls of the same names.

use crate::consts::*;

/// One combine unit's setting (`grColorCombine` or `grAlphaCombine`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combine {
    pub function: u32,
    pub factor: u32,
    pub local: u32,
    pub other: u32,
    pub invert: bool,
}

impl Combine {
    pub const fn new(function: u32, factor: u32, local: u32, other: u32, invert: bool) -> Combine {
        Combine {
            function,
            factor,
            local,
            other,
            invert,
        }
    }

    /// `guColorCombineFunction`: each `GR_COLORCOMBINE_*` as the `grColorCombine` call it
    /// stands for (the Glide 2.x reference manual's table). `None` for an unknown one.
    pub fn colour_function(f: u32) -> Option<Combine> {
        let c = |a, b, l, o| Some(Combine::new(a, b, l, o, false));
        match f {
            CC_ZERO => c(FUNC_ZERO, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED),
            CC_CCRGB => c(FUNC_LOCAL, FACTOR_ZERO, LOCAL_CONSTANT, OTHER_CONSTANT),
            CC_ITRGB | CC_ITRGB_DELTA0 => c(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT),
            CC_DECAL_TEXTURE => c(FUNC_SCALE_OTHER, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE),
            CC_TEXTURE_TIMES_CCRGB => c(FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_CONSTANT, OTHER_TEXTURE),
            CC_TEXTURE_TIMES_ITRGB | CC_TEXTURE_TIMES_ITRGB_DELTA0 => {
                c(FUNC_SCALE_OTHER, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE)
            }
            CC_TEXTURE_TIMES_ITRGB_ADD_ALPHA => {
                c(FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA, FACTOR_LOCAL, LOCAL_ITERATED, OTHER_TEXTURE)
            }
            CC_TEXTURE_TIMES_ALPHA => c(FUNC_SCALE_OTHER, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE),
            CC_TEXTURE_TIMES_ALPHA_ADD_ITRGB => {
                c(FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE)
            }
            CC_TEXTURE_ADD_ITRGB => c(FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE),
            CC_TEXTURE_SUB_ITRGB => c(FUNC_SCALE_OTHER_MINUS_LOCAL, FACTOR_ONE, LOCAL_ITERATED, OTHER_TEXTURE),
            CC_ONE => Some(Combine::new(FUNC_ZERO, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, true)),
            _ => None,
        }
    }
}

/// The state, with the values Glide sets in `grSstOpen` (the Glide 2.x reference manual:
/// colour = iterated, alpha = iterated, no blending, no chroma key, no fog, depth off, 4 × 4
/// dither).
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub colour: Combine,
    pub alpha: Combine,
    /// `guTexCombineFunction` on the one TMU modelled: decal, zero or one.
    pub texture: u32,
    /// `grConstantColorValue`: `0xAARRGGBB` (`GR_COLORFORMAT_ARGB`).
    pub constant: u32,
    /// `grAlphaBlendFunction`: colour source and destination, alpha source and destination.
    pub blend: [u32; 4],
    /// `grChromakeyMode`, `grChromakeyValue` (`0x00RRGGBB`).
    pub chroma: bool,
    pub key: u32,
    pub depth_mode: u32,
    pub depth_function: u32,
    pub depth_mask: bool,
    /// `grAlphaTestFunction` and its reference.
    pub alpha_test: (u32, u8),
    pub fog_mode: u32,
    pub fog_colour: u32,
    pub fog_table: [u8; 64],
    pub filter: u32,
    /// `grTexClampMode`: s and t.
    pub clamp: [u32; 2],
    /// `grDitherMode`: false for `GR_DITHER_DISABLE`.
    pub dither: bool,
    /// `grClipWindow`: left, top, right, bottom (right and bottom not drawn), in pixels.
    pub clip: [i32; 4],
}

impl Default for State {
    fn default() -> State {
        State {
            colour: Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, false),
            alpha: Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_ITERATED, false),
            texture: TEXTURECOMBINE_DECAL,
            constant: 0,
            blend: [BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO],
            chroma: false,
            key: 0,
            depth_mode: DEPTHBUFFER_DISABLE,
            depth_function: CMP_LESS,
            depth_mask: false,
            alpha_test: (CMP_ALWAYS, 0),
            fog_mode: FOG_DISABLE,
            fog_colour: 0,
            fog_table: [0; 64],
            filter: TEXTUREFILTER_POINT_SAMPLED,
            clamp: [TEXTURECLAMP_WRAP; 2],
            dither: true,
            clip: [0, 0, i32::MAX, i32::MAX],
        }
    }
}

/// Whether `new` passes `old` under `GrCmpFnc_t` `f`.
pub fn compare<T: PartialOrd>(f: u32, new: T, old: T) -> bool {
    match f {
        CMP_NEVER => false,
        CMP_LESS => new < old,
        CMP_EQUAL => new == old,
        CMP_LEQUAL => new <= old,
        CMP_GREATER => new > old,
        CMP_NOTEQUAL => new != old,
        CMP_GEQUAL => new >= old,
        _ => true,
    }
}
