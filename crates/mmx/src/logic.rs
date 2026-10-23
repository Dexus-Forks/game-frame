//! Bitwise logic on all 64 bits.

use crate::lanes::Mm;

pub fn pand(a: Mm, b: Mm) -> Mm {
    Mm(a.0 & b.0)
}

/// `!a & b`: note the first operand is the one inverted.
pub fn pandn(a: Mm, b: Mm) -> Mm {
    Mm(!a.0 & b.0)
}

pub fn por(a: Mm, b: Mm) -> Mm {
    Mm(a.0 | b.0)
}

pub fn pxor(a: Mm, b: Mm) -> Mm {
    Mm(a.0 ^ b.0)
}
