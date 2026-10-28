//! Rounding an exact binary value, the one step every x87 result goes through.

use crate::Rounding;
use core::cmp::Ordering;

/// The number of significant bits of `m`.
pub(crate) fn bit_len(m: u128) -> u32 {
    128 - m.leading_zeros()
}

/// `m / 2^s` rounded to an integer by `rc`, where `sticky` says that the exact value has more
/// nonzero bits below `m`'s last one. `neg` is the sign, which the directed modes need.
pub(crate) fn round_shift(m: u128, s: u32, sticky: bool, neg: bool, rc: Rounding) -> u128 {
    let (int, rest_vs_half, inexact) = if s == 0 {
        (m, Ordering::Less, sticky)
    } else if s > 128 {
        (0, Ordering::Less, m != 0 || sticky)
    } else {
        let int = if s == 128 { 0 } else { m >> s };
        let rest = if s == 128 { m } else { m & ((1u128 << s) - 1) };
        let half = 1u128 << (s - 1);
        let cmp = match rest.cmp(&half) {
            Ordering::Equal if sticky => Ordering::Greater,
            c => c,
        };
        (int, cmp, rest != 0 || sticky)
    };
    let up = match rc {
        Rounding::Nearest => {
            rest_vs_half == Ordering::Greater || (rest_vs_half == Ordering::Equal && int & 1 == 1)
        }
        Rounding::Zero => false,
        Rounding::Up => !neg && inexact,
        Rounding::Down => neg && inexact,
    };
    int + u128::from(up)
}

/// `m` (with `sticky` below it) rounded to `bits` significant bits: the significand and the
/// power of two it was shifted right by (negative when it was shifted left).
pub(crate) fn round_to(m: u128, sticky: bool, bits: u32, neg: bool, rc: Rounding) -> (u128, i32) {
    let mut m = m;
    let mut pad = 0;
    let mut n = bit_len(m);
    if n <= bits {
        // Room below for the rounding position: shifting left is exact.
        pad = (bits - n + 1) as i32;
        m <<= pad;
        n = bits + 1;
    }
    let s = n - bits;
    let int = round_shift(m, s, sticky, neg, rc);
    if int >> bits != 0 {
        (int >> 1, s as i32 + 1 - pad)
    } else {
        (int, s as i32 - pad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_go_to_even_only_when_nearest() {
        assert_eq!(round_shift(0b101, 1, false, false, Rounding::Nearest), 0b10);
        assert_eq!(
            round_shift(0b111, 1, false, false, Rounding::Nearest),
            0b100
        );
        assert_eq!(round_shift(0b101, 1, true, false, Rounding::Nearest), 0b11);
        assert_eq!(round_shift(0b101, 1, false, false, Rounding::Up), 0b11);
        assert_eq!(round_shift(0b101, 1, false, true, Rounding::Up), 0b10);
        assert_eq!(round_shift(0b101, 1, false, true, Rounding::Down), 0b11);
        assert_eq!(round_shift(0b101, 1, false, true, Rounding::Zero), 0b10);
        assert_eq!(round_shift(5, 200, false, false, Rounding::Up), 1);
        assert_eq!(round_shift(5, 200, false, false, Rounding::Nearest), 0);
    }

    #[test]
    fn a_carry_adds_a_bit() {
        assert_eq!(
            round_to(0xff, false, 4, false, Rounding::Nearest),
            (0b1000, 5)
        );
        assert_eq!(
            round_to(0b11, false, 4, false, Rounding::Nearest),
            (0b1100, -2)
        );
    }
}
