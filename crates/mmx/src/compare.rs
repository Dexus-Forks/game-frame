//! Packed compares: all ones in a lane where the test holds, zero where it does not.

use crate::lanes::{zip_b, zip_d, zip_w, Mm};

fn mask<T: Default + std::ops::Not<Output = T>>(t: bool) -> T {
    if t { !T::default() } else { T::default() }
}

pub fn pcmpeqb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, |x, y| mask(x == y))
}

pub fn pcmpeqw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| mask(x == y))
}

pub fn pcmpeqd(a: Mm, b: Mm) -> Mm {
    zip_d(a, b, |x, y| mask(x == y))
}

/// Signed greater-than.
pub fn pcmpgtb(a: Mm, b: Mm) -> Mm {
    zip_b(a, b, |x, y| mask((x as i8) > (y as i8)))
}

pub fn pcmpgtw(a: Mm, b: Mm) -> Mm {
    zip_w(a, b, |x, y| mask((x as i16) > (y as i16)))
}

pub fn pcmpgtd(a: Mm, b: Mm) -> Mm {
    zip_d(a, b, |x, y| mask((x as i32) > (y as i32)))
}
