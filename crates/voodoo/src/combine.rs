//! The colour and alpha combine units: `local`, `other`, a factor and a function, as the
//! Glide 2.x reference manual describes `grColorCombine` and `grAlphaCombine`. Values are
//! 0–1. The same rules as the fragment shader of GameFrame's `wrappers/glide`.
//!
//! Deviation: the Voodoo computes these in 8-bit fixed point (and its factor has a ninth bit
//! so 1.0 is exact); floats are used here. The chip's rounding is *unknown*.

use crate::consts::*;
use crate::state::Combine;

/// The inputs a pixel has before combining: the iterated colour, the texel and the constant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inputs {
    pub iterated: [f32; 4],
    pub texel: [f32; 4],
    pub constant: [f32; 4],
}

fn pick_local(local: u32, i: &Inputs) -> [f32; 4] {
    if local == LOCAL_CONSTANT { i.constant } else { i.iterated }
}

fn pick_other(other: u32, i: &Inputs) -> [f32; 4] {
    match other {
        OTHER_TEXTURE => i.texel,
        OTHER_CONSTANT => i.constant,
        _ => i.iterated,
    }
}

fn colour_factor(f: u32, cl: [f32; 3], al: f32, ao: f32, at: f32) -> [f32; 3] {
    match f {
        FACTOR_LOCAL => cl,
        FACTOR_OTHER_ALPHA => [ao; 3],
        FACTOR_LOCAL_ALPHA => [al; 3],
        FACTOR_TEXTURE_ALPHA => [at; 3],
        FACTOR_ONE => [1.0; 3],
        FACTOR_ONE_MINUS_LOCAL => cl.map(|c| 1.0 - c),
        FACTOR_ONE_MINUS_OTHER_ALPHA => [1.0 - ao; 3],
        FACTOR_ONE_MINUS_LOCAL_ALPHA => [1.0 - al; 3],
        FACTOR_ONE_MINUS_TEXTURE_ALPHA => [1.0 - at; 3],
        _ => [0.0; 3],
    }
}

fn alpha_factor(f: u32, al: f32, ao: f32, at: f32) -> f32 {
    match f {
        FACTOR_LOCAL | FACTOR_LOCAL_ALPHA => al,
        FACTOR_OTHER_ALPHA => ao,
        FACTOR_TEXTURE_ALPHA => at,
        FACTOR_ONE => 1.0,
        FACTOR_ONE_MINUS_LOCAL | FACTOR_ONE_MINUS_LOCAL_ALPHA => 1.0 - al,
        FACTOR_ONE_MINUS_OTHER_ALPHA => 1.0 - ao,
        FACTOR_ONE_MINUS_TEXTURE_ALPHA => 1.0 - at,
        _ => 0.0,
    }
}

fn function(func: u32, f: f32, l: f32, o: f32, al: f32) -> f32 {
    match func {
        FUNC_LOCAL => l,
        FUNC_LOCAL_ALPHA => al,
        FUNC_SCALE_OTHER => f * o,
        FUNC_SCALE_OTHER_ADD_LOCAL => f * o + l,
        FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA => f * o + al,
        FUNC_SCALE_OTHER_MINUS_LOCAL => f * (o - l),
        FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL => f * (o - l) + l,
        FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA => f * (o - l) + al,
        FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL => l - f * l,
        FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA => al - f * l,
        _ => 0.0,
    }
}

/// The pixel's colour (r, g, b) and alpha after both units, clamped to 0–1, and the `other`
/// colour the chroma key compares (the Voodoo keys on the colour before combining).
pub fn combine(colour: &Combine, alpha: &Combine, i: &Inputs) -> ([f32; 4], [f32; 3]) {
    let al = pick_local(alpha.local, i)[3];
    let ao = pick_other(alpha.other, i)[3];
    let at = i.texel[3];
    let cl4 = pick_local(colour.local, i);
    let co4 = pick_other(colour.other, i);
    let (cl, co) = ([cl4[0], cl4[1], cl4[2]], [co4[0], co4[1], co4[2]]);
    let f = colour_factor(colour.factor, cl, al, ao, at);
    let mut out = [0.0; 4];
    for k in 0..3 {
        let c = function(colour.function, f[k], cl[k], co[k], al).clamp(0.0, 1.0);
        out[k] = if colour.invert { 1.0 - c } else { c };
    }
    let fa = alpha_factor(alpha.factor, al, ao, at);
    let a = function(alpha.function, fa, al, ao, al).clamp(0.0, 1.0);
    out[3] = if alpha.invert { 1.0 - a } else { a };
    (out, co)
}

/// `0xAARRGGBB` as four values 0–1, in the order r, g, b, a.
pub fn argb(c: u32) -> [f32; 4] {
    let b = |s: u32| ((c >> s) & 0xff) as f32 / 255.0;
    [b(16), b(8), b(0), b(24)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> Inputs {
        Inputs {
            iterated: [0.25, 0.5, 0.0, 0.5],
            texel: [0.8, 0.4, 0.2, 1.0],
            constant: [1.0, 0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn texture_times_alpha_add_iterated() {
        // POD's lit faces: (SCALE_OTHER_ADD_LOCAL, LOCAL_ALPHA, ITERATED, TEXTURE).
        let c = Combine::new(FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, false);
        let a = Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, false);
        let (out, other) = combine(&c, &a, &inputs());
        assert_eq!(out, [0.65, 0.7, 0.1, 0.5]);
        assert_eq!(other, [0.8, 0.4, 0.2]);
    }

    #[test]
    fn the_gu_functions() {
        let a = Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, false);
        let decal = Combine::colour_function(CC_DECAL_TEXTURE).unwrap();
        assert_eq!(combine(&decal, &a, &inputs()).0[..3], [0.8, 0.4, 0.2]);
        let ccrgb = Combine::colour_function(CC_CCRGB).unwrap();
        assert_eq!(combine(&ccrgb, &a, &inputs()).0[..3], [1.0, 0.0, 0.0]);
        let one = Combine::colour_function(CC_ONE).unwrap();
        assert_eq!(combine(&one, &a, &inputs()).0[..3], [1.0; 3]);
        assert_eq!(Combine::colour_function(0x40), None);
    }

    #[test]
    fn argb_order() {
        assert_eq!(argb(0x80ff0000), [1.0, 0.0, 0.0, 128.0 / 255.0]);
    }
}
