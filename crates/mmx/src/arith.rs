//! Packed add, subtract and multiply: wrapping, signed saturating and unsigned saturating.

use crate::lanes::{zip_b, zip_d, zip_w, Mm};

pub fn paddb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, u8::wrapping_add)
}

pub fn paddw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, u16::wrapping_add)
}

pub fn paddd(a: Mm, b: Mm) -> Mm {
    zip_d(a, b, u32::wrapping_add)
}

pub fn psubb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, u8::wrapping_sub)
}

pub fn psubw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, u16::wrapping_sub)
}

pub fn psubd(a: Mm, b: Mm) -> Mm {
    zip_d(a, b, u32::wrapping_sub)
}

/// Signed bytes, clamped to -128..127.
pub fn paddsb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, |x, y| (x as i8).saturating_add(y as i8) as u8)
}

/// Signed words, clamped to -32768..32767: the usual sample mixer.
pub fn paddsw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| (x as i16).saturating_add(y as i16) as u16)
}

pub fn psubsb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, |x, y| (x as i8).saturating_sub(y as i8) as u8)
}

pub fn psubsw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| (x as i16).saturating_sub(y as i16) as u16)
}

/// Unsigned bytes, clamped to 0..255.
pub fn paddusb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, u8::saturating_add)
}

pub fn paddusw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, u16::saturating_add)
}

pub fn psubusb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, u8::saturating_sub)
}

pub fn psubusw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, u16::saturating_sub)
}

/// Low 16 bits of each signed 16 × 16 product.
pub fn pmullw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| ((x as i16 as i32) * (y as i16 as i32)) as u16)
}

/// High 16 bits of each signed 16 × 16 product: `(a * b) >> 16`, rounding toward minus
/// infinity. The usual fixed-point volume or blend multiply.
pub fn pmulhw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| (((x as i16 as i32) * (y as i16 as i32)) >> 16) as u16)
}

/// Multiply signed words and add adjacent pairs into two doublewords. The one case that
/// overflows, all four words `0x8000`, gives `0x80000000` in each lane, as the hardware does.
pub fn pmaddwd(a: Mm, b: Mm) -> Mm {
    let (x, y) = (a.swords(), b.swords());
    let p = |i: usize| (x[i] as i32).wrapping_mul(y[i] as i32);
    Mm::from_sdwords([p(0).wrapping_add(p(1)), p(2).wrapping_add(p(3))])
}
