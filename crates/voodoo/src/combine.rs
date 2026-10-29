//! The colour and alpha combine units, in the chip's integer arithmetic.
//!
//! *Observed* in the SST-1 (Voodoo Graphics) specification, revision 1.61, section 5.16
//! (`fbzColorPath`, the two combine-unit diagrams): `other` (or 0 with `zero_other`) minus
//! `local` (with `sub_clocal`) is a 9-bit signed value; the blend factor is an 8-bit select
//! (0, `c_local`, `a_other`, `a_local`, texture alpha) XORed with the inverse of
//! `reverse_blend` and incremented, a 9-bit 0–256; their product drops its 8 low bits with no
//! rounding; `c_local` or `a_local` (or 0) is added, the sum is clamped to 0–255 and XORed
//! with `invert_output`.
//!
//! *Inferred*: how Glide's `grColorCombine` arguments set those bits. The factor's bit 3
//! ("one minus") clears `reverse_blend` and its low bits are the select, so `FACTOR_ONE` is
//! 256 exactly and `FACTOR_LOCAL` is `c_local + 1`. The functions:
//!
//! | Function | `zero_other` | `sub_clocal` | add |
//! |---|---|---|---|
//! | `ZERO` | 1 | 0 | none |
//! | `LOCAL`, `LOCAL_ALPHA` | 1 | 0 | `c_local`, `a_local` |
//! | `SCALE_OTHER` | 0 | 0 | none |
//! | `SCALE_OTHER_ADD_LOCAL(_ALPHA)` | 0 | 0 | `c_local` (`a_local`) |
//! | `SCALE_OTHER_MINUS_LOCAL` | 0 | 1 | none |
//! | `SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL(_ALPHA)` | 0 | 1 | `c_local` (`a_local`) |
//! | `SCALE_MINUS_LOCAL_ADD_LOCAL(_ALPHA)` | 1 | 1 | `c_local` (`a_local`) |

use crate::consts::*;
use crate::state::Combine;

/// The inputs a pixel has before combining, 8 bits a channel, r, g, b, a: the iterated colour
/// (the iterators' integer parts, clamped), the texel and `grConstantColorValue`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inputs {
    pub iterated: [u8; 4],
    pub texel: [u8; 4],
    pub constant: [u8; 4],
}

/// The unit's control bits, from a Glide setting (table above).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bits {
    zero_other: bool,
    sub_clocal: bool,
    mselect: u32,
    reverse_blend: bool,
    add_clocal: bool,
    add_alocal: bool,
}

fn bits(c: &Combine) -> Bits {
    let (zero_other, sub_clocal, add) = match c.function {
        FUNC_ZERO => (true, false, 0),
        FUNC_LOCAL => (true, false, 1),
        FUNC_LOCAL_ALPHA => (true, false, 2),
        FUNC_SCALE_OTHER => (false, false, 0),
        FUNC_SCALE_OTHER_ADD_LOCAL => (false, false, 1),
        FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA => (false, false, 2),
        FUNC_SCALE_OTHER_MINUS_LOCAL => (false, true, 0),
        FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL => (false, true, 1),
        FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA => (false, true, 2),
        FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL => (true, true, 1),
        FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA => (true, true, 2),
        _ => (true, false, 0),
    };
    Bits {
        zero_other,
        sub_clocal,
        mselect: c.factor & 7,
        reverse_blend: c.factor & 8 == 0,
        add_clocal: add == 1,
        add_alocal: add == 2,
    }
}

/// One channel through a unit. `m` is the factor select's value: 0, `c_local` (this
/// channel's), `a_other`, `a_local`, texture alpha.
fn channel(b: &Bits, other: u8, local: u8, a_local: u8, m: [u8; 5], invert: bool) -> u8 {
    let o = if b.zero_other { 0 } else { other as i32 };
    let l = if b.sub_clocal { local as i32 } else { 0 };
    let diff = o - l;
    let sel = m.get(b.mselect as usize).copied().unwrap_or(0) as i32;
    let factor = if b.reverse_blend { sel } else { sel ^ 0xff } + 1;
    let product = (diff * factor) >> 8;
    let add = if b.add_clocal {
        local as i32
    } else if b.add_alocal {
        a_local as i32
    } else {
        0
    };
    let v = (product + add).clamp(0, 255) as u8;
    if invert { !v } else { v }
}

fn pick_local(local: u32, i: &Inputs) -> [u8; 4] {
    if local == LOCAL_CONSTANT { i.constant } else { i.iterated }
}

fn pick_other(other: u32, i: &Inputs) -> [u8; 4] {
    match other {
        OTHER_TEXTURE => i.texel,
        OTHER_CONSTANT => i.constant,
        _ => i.iterated,
    }
}

/// The pixel's colour and alpha after both units, and the colour unit's `other` input,
/// which the chroma key compares (section 5.19: "immediately after texture mapping lookup,
/// but before the color combine unit").
pub fn combine(colour: &Combine, alpha: &Combine, i: &Inputs) -> ([u8; 4], [u8; 3]) {
    let a_local = pick_local(alpha.local, i)[3];
    let a_other = pick_other(alpha.other, i)[3];
    let t_alpha = i.texel[3];
    let cl = pick_local(colour.local, i);
    let co = pick_other(colour.other, i);
    let cb = bits(colour);
    let mut out = [0u8; 4];
    for k in 0..3 {
        let m = [0, cl[k], a_other, a_local, t_alpha];
        out[k] = channel(&cb, co[k], cl[k], a_local, m, colour.invert);
    }
    let ab = bits(alpha);
    let m = [0, a_local, a_other, a_local, t_alpha];
    out[3] = channel(&ab, a_other, a_local, a_local, m, alpha.invert);
    (out, [co[0], co[1], co[2]])
}

/// `0xAARRGGBB` as r, g, b, a.
pub fn argb(c: u32) -> [u8; 4] {
    let b = |s: u32| ((c >> s) & 0xff) as u8;
    [b(16), b(8), b(0), b(24)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> Inputs {
        Inputs {
            iterated: [64, 128, 0, 128],
            texel: [200, 100, 50, 255],
            constant: [255, 0, 0, 255],
        }
    }

    const ITERATED_ALPHA: Combine = Combine::new(FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, false);

    #[test]
    fn texture_times_alpha_add_iterated() {
        // POD's lit faces: (SCALE_OTHER_ADD_LOCAL, LOCAL_ALPHA, ITERATED, TEXTURE). The
        // factor is a_local + 1 = 129: 200 × 129 >> 8 = 100, + 64.
        let c = Combine::new(FUNC_SCALE_OTHER_ADD_LOCAL, FACTOR_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, false);
        let (out, other) = combine(&c, &ITERATED_ALPHA, &inputs());
        assert_eq!(out, [164, 178, 25, 128]);
        assert_eq!(other, [200, 100, 50]);
    }

    #[test]
    fn factor_one_is_exact_and_a_full_alpha_nearly() {
        let decal = Combine::colour_function(CC_DECAL_TEXTURE).unwrap();
        assert_eq!(combine(&decal, &ITERATED_ALPHA, &inputs()).0[..3], [200, 100, 50]);
        // Alpha 255 is a factor of 256: the texel through unchanged.
        let c = Combine::new(FUNC_SCALE_OTHER, FACTOR_TEXTURE_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, false);
        assert_eq!(combine(&c, &ITERATED_ALPHA, &inputs()).0[..3], [200, 100, 50]);
        // One minus local alpha (128): 256 - 128 = 128.
        let c = Combine::new(FUNC_SCALE_OTHER, FACTOR_ONE_MINUS_LOCAL_ALPHA, LOCAL_ITERATED, OTHER_TEXTURE, false);
        assert_eq!(combine(&c, &ITERATED_ALPHA, &inputs()).0[..3], [100, 50, 25]);
    }

    #[test]
    fn subtraction_truncates_towards_minus_infinity() {
        // (iterated - texel) × 256 >> 8, then + iterated: the texel in the end.
        let c = Combine::new(FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL, FACTOR_ONE, LOCAL_CONSTANT, OTHER_TEXTURE, false);
        assert_eq!(combine(&c, &ITERATED_ALPHA, &inputs()).0[..3], [200, 100, 50]);
        // SCALE_MINUS_LOCAL_ADD_LOCAL with factor zero (1/256): local − local/256.
        let c = Combine::new(FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_TEXTURE, false);
        assert_eq!(combine(&c, &ITERATED_ALPHA, &inputs()).0[..3], [63, 127, 0]);
    }

    #[test]
    fn the_gu_functions() {
        let ccrgb = Combine::colour_function(CC_CCRGB).unwrap();
        assert_eq!(combine(&ccrgb, &ITERATED_ALPHA, &inputs()).0[..3], [255, 0, 0]);
        let one = Combine::colour_function(CC_ONE).unwrap();
        assert_eq!(combine(&one, &ITERATED_ALPHA, &inputs()).0[..3], [255; 3]);
        assert_eq!(combine(&ITERATED_ALPHA, &ITERATED_ALPHA, &inputs()).0[3], 128);
        assert_eq!(Combine::colour_function(0x40), None);
    }

    #[test]
    fn argb_order() {
        assert_eq!(argb(0x80ff0000), [255, 0, 0, 128]);
    }
}
