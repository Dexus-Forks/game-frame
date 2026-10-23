//! Shifts. The count is the whole 64-bit second operand (or an immediate): a count at or
//! above the lane width gives 0 for logical shifts and fills with the sign for arithmetic
//! ones. `psrlq` by 64 or more is how hand-written code makes "all ones above bit n" masks.

use crate::lanes::Mm;

fn lanes_w(a: Mm, f: impl Fn(u16) -> u16) -> Mm {
    Mm::from_words(a.words().map(f))
}

fn lanes_d(a: Mm, f: impl Fn(u32) -> u32) -> Mm {
    Mm::from_dwords(a.dwords().map(f))
}

pub fn psllw(a: Mm, n: u64) -> Mm {
    lanes_w(a, |v| if n > 15 { 0 } else { v << n })
}

pub fn pslld(a: Mm, n: u64) -> Mm {
    lanes_d(a, |v| if n > 31 { 0 } else { v << n })
}

pub fn psllq(a: Mm, n: u64) -> Mm {
    Mm(if n > 63 { 0 } else { a.0 << n })
}

pub fn psrlw(a: Mm, n: u64) -> Mm {
    lanes_w(a, |v| if n > 15 { 0 } else { v >> n })
}

pub fn psrld(a: Mm, n: u64) -> Mm {
    lanes_d(a, |v| if n > 31 { 0 } else { v >> n })
}

pub fn psrlq(a: Mm, n: u64) -> Mm {
    Mm(if n > 63 { 0 } else { a.0 >> n })
}

pub fn psraw(a: Mm, n: u64) -> Mm {
    lanes_w(a, |v| ((v as i16) >> n.min(15)) as u16)
}

pub fn psrad(a: Mm, n: u64) -> Mm {
    lanes_d(a, |v| ((v as i32) >> n.min(31)) as u32)
}
