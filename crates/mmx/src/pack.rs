//! Packs (narrow with saturation) and unpacks (interleave the low or high halves).

use crate::lanes::Mm;

/// Signed words of `a` then `b` to signed bytes, clamped to -128..127.
pub fn packsswb(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.swords(), b.swords());
    let s = |v: i16| v.clamp(-128, 127) as i8 as u8;
    Mm::from_bytes([s(x[0]), s(x[1]), s(x[2]), s(x[3]), s(y[0]), s(y[1]), s(y[2]), s(y[3])])
}

/// Signed doublewords of `a` then `b` to signed words, clamped to -32768..32767.
pub fn packssdw(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.sdwords(), b.sdwords());
    let s = |v: i32| v.clamp(-32768, 32767) as i16;
    Mm::from_swords([s(x[0]), s(x[1]), s(y[0]), s(y[1])])
}

/// Signed words of `a` then `b` to unsigned bytes, clamped to 0..255.
pub fn packuswb(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.swords(), b.swords());
    let s = |v: i16| v.clamp(0, 255) as u8;
    Mm::from_bytes([s(x[0]), s(x[1]), s(x[2]), s(x[3]), s(y[0]), s(y[1]), s(y[2]), s(y[3])])
}

pub fn punpcklbw(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.bytes(), b.bytes());
    Mm::from_bytes([x[0], y[0], x[1], y[1], x[2], y[2], x[3], y[3]])
}

pub fn punpckhbw(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.bytes(), b.bytes());
    Mm::from_bytes([x[4], y[4], x[5], y[5], x[6], y[6], x[7], y[7]])
}

/// `a0 b0 a1 b1`: with `b` a second channel, this interleaves two mono streams into stereo.
pub fn punpcklwd(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.words(), b.words());
    Mm::from_words([x[0], y[0], x[1], y[1]])
}

pub fn punpckhwd(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.words(), b.words());
    Mm::from_words([x[2], y[2], x[3], y[3]])
}

pub fn punpckldq(a: Mm, b: Mm) -> Mm {
    Mm::from_dwords([a.dwords()[0], b.dwords()[0]])
}

pub fn punpckhdq(a: Mm, b: Mm) -> Mm {
    Mm::from_dwords([a.dwords()[1], b.dwords()[1]])
}
