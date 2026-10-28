//! `fadd`, `fsub`, `fmul`, `fdiv` and `fsqrt`: the exact result, rounded once by the control word.

use crate::round::bit_len;
use crate::{Control, Ext, Rounding};
use core::cmp::Ordering;

impl Ext {
    /// `fadd` at the default control word (64 bits, to nearest).
    pub fn fadd(self, o: Ext) -> Ext {
        self.fadd_with(o, Control::DEFAULT)
    }

    /// `fsub` (self − o) at the default control word.
    pub fn fsub(self, o: Ext) -> Ext {
        self.fadd_with(o.fchs(), Control::DEFAULT)
    }

    /// `fmul` at the default control word.
    pub fn fmul(self, o: Ext) -> Ext {
        self.fmul_with(o, Control::DEFAULT)
    }

    /// `fdiv` (self / o) at the default control word.
    ///
    /// # Panics
    /// When `o` is 0: the x87 gives an infinity or a NaN there, which this crate does not
    /// model (see [`Ext::checked_fdiv_with`]).
    pub fn fdiv(self, o: Ext) -> Ext {
        self.checked_fdiv_with(o, Control::DEFAULT)
            .expect("gameframe-x87: division by zero")
    }

    /// `fsqrt` at the default control word.
    ///
    /// # Panics
    /// For a negative value (below −0).
    pub fn fsqrt(self) -> Ext {
        self.checked_fsqrt_with(Control::DEFAULT)
            .expect("gameframe-x87: square root of a negative value")
    }

    /// `fsub` (self − o) under `cw`.
    pub fn fsub_with(self, o: Ext, cw: Control) -> Ext {
        self.fadd_with(o.fchs(), cw)
    }

    /// `fadd` under `cw`.
    pub fn fadd_with(self, o: Ext, cw: Control) -> Ext {
        // An exact zero sum is +0, or −0 when rounding down; −0 + −0 is −0 in every mode.
        let zero_sum =
            Ext::zero(self.neg && o.neg || cw.rounding == Rounding::Down && self.neg != o.neg);
        match (self.is_zero(), o.is_zero()) {
            (true, true) => return zero_sum,
            (true, false) => return Ext::round_from(o.neg, o.exp, u128::from(o.mant), false, cw),
            (false, true) => {
                return Ext::round_from(self.neg, self.exp, u128::from(self.mant), false, cw)
            }
            _ => {}
        }
        let (big, small) = if self.exp >= o.exp {
            (self, o)
        } else {
            (o, self)
        };
        // Both at 2^(big.exp − 62): 62 guard bits below the larger's significand; the smaller's
        // bits past them only make a sticky bit.
        let d = (big.exp - small.exp) as u32;
        let a = u128::from(big.mant) << 62;
        let (b, sticky) = if d >= 126 {
            (0, true)
        } else {
            let full = u128::from(small.mant) << 62;
            (full >> d, full & ((1u128 << d) - 1) != 0)
        };
        let e = big.exp - 62;
        if big.neg == small.neg {
            return Ext::round_from(big.neg, e, a + b, sticky, cw);
        }
        // A difference: the sticky part borrows one unit, and leaves a fraction behind.
        let b = b + u128::from(sticky);
        match a.cmp(&b) {
            Ordering::Equal => zero_sum,
            Ordering::Greater => Ext::round_from(big.neg, e, a - b, sticky, cw),
            Ordering::Less => Ext::round_from(small.neg, e, b - a, sticky, cw),
        }
    }

    /// `fmul` under `cw`.
    pub fn fmul_with(self, o: Ext, cw: Control) -> Ext {
        let neg = self.neg != o.neg;
        if self.is_zero() || o.is_zero() {
            return Ext::zero(neg);
        }
        let m = u128::from(self.mant) * u128::from(o.mant);
        Ext::round_from(neg, self.exp + o.exp, m, false, cw)
    }

    /// `fdiv` (self / o) under `cw`; `None` when `o` is 0.
    pub fn checked_fdiv_with(self, o: Ext, cw: Control) -> Option<Ext> {
        if o.is_zero() {
            return None;
        }
        let neg = self.neg != o.neg;
        if self.is_zero() {
            return Some(Ext::zero(neg));
        }
        // 64 or 65 quotient bits, then two more by long division: enough below the 64th for
        // the rounding bit, and the remainder for the sticky bit.
        let den = u128::from(o.mant);
        let num = u128::from(self.mant) << 64;
        let mut q = num / den;
        let mut rem = num % den;
        for _ in 0..2 {
            rem <<= 1;
            let bit = rem >= den;
            if bit {
                rem -= den;
            }
            q = q << 1 | u128::from(bit);
        }
        Some(Ext::round_from(neg, self.exp - o.exp - 66, q, rem != 0, cw))
    }

    /// `fsqrt` under `cw`; `None` for a negative value. The square root of −0 is −0.
    pub fn checked_fsqrt_with(self, cw: Control) -> Option<Ext> {
        if self.is_zero() {
            return Some(self);
        }
        if self.neg {
            return None;
        }
        // value = n · 2^(exp − shift) with an even power: its root is √n · 2^((exp − shift)/2),
        // and √n has 64 bits.
        let shift = if self.exp & 1 == 0 { 64 } else { 63 };
        let n = u128::from(self.mant) << shift;
        let r = isqrt(n);
        let rem = n - r * r;
        // One more bit: the root's fraction is at least one half exactly when rem > r (it is
        // never exactly one half); the rest is the sticky bit.
        let q = r << 1 | u128::from(rem > r);
        Some(Ext::round_from(
            false,
            (self.exp - shift) / 2 - 1,
            q,
            rem != 0,
            cw,
        ))
    }
}

/// The integer square root (floor) of `n`.
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    // Newton's method from a power of two above the root: it falls monotonically to the floor.
    let mut x = 1u128 << bit_len(n).div_ceil(2);
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

#[cfg(test)]
mod tests {
    use super::isqrt;

    #[test]
    fn isqrt_is_the_floor() {
        for n in [
            0u128,
            1,
            2,
            3,
            4,
            15,
            16,
            17,
            u128::from(u64::MAX),
            u128::MAX,
        ] {
            let r = isqrt(n);
            assert!(r * r <= n, "{n}");
            assert!((r + 1).checked_mul(r + 1).map_or(true, |s| s > n), "{n}");
        }
    }
}
