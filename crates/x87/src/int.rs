//! `frndint` and `fist`: a value rounded to an integer by the rounding control.

use crate::round::round_shift;
use crate::{Control, Ext, Rounding};

impl Ext {
    /// The magnitude rounded to an integer by `rc` (the precision does not apply).
    fn integer(self, rc: Rounding) -> u128 {
        if self.is_zero() {
            return 0;
        }
        if self.exp >= 0 {
            // At least 2^63: wider than any integer `fist` stores, past it saturate.
            return if self.exp > 63 {
                u128::MAX
            } else {
                u128::from(self.mant) << self.exp
            };
        }
        let s = (-self.exp).min(200) as u32;
        round_shift(u128::from(self.mant), s, false, self.neg, rc)
    }

    /// `frndint` under `cw`'s rounding: an integer value, keeping the sign of a zero result.
    pub fn frndint(self, cw: Control) -> Ext {
        if self.is_zero() || self.exp >= 0 {
            return self;
        }
        let k = self.integer(cw.rounding);
        if k == 0 {
            return Ext::zero(self.neg);
        }
        Ext::round_from(self.neg, 0, k, false, Control::DEFAULT)
    }

    /// The signed integer, or `None` when it does not fit in `bits` bits.
    fn fist(self, cw: Control, bits: u32) -> Option<i64> {
        let k = self.integer(cw.rounding);
        let limit = 1u128 << (bits - 1);
        if self.neg {
            (k <= limit).then(|| (k as i64).wrapping_neg())
        } else {
            (k < limit).then_some(k as i64)
        }
    }

    /// `fist word` under `cw`'s rounding. Out of range, the x87 (with the invalid-operation
    /// exception masked) stores the "integer indefinite", −32768.
    pub fn fist_i16(self, cw: Control) -> i16 {
        self.fist(cw, 16).map_or(i16::MIN, |v| v as i16)
    }

    /// `fist dword` under `cw`'s rounding; `i32::MIN` (the integer indefinite) out of range.
    pub fn fist_i32(self, cw: Control) -> i32 {
        self.fist(cw, 32).map_or(i32::MIN, |v| v as i32)
    }

    /// `fist qword` under `cw`'s rounding; `i64::MIN` (the integer indefinite) out of range.
    pub fn fist_i64(self, cw: Control) -> i64 {
        self.fist(cw, 64).unwrap_or(i64::MIN)
    }

    /// `fistp dword` to nearest, ties to even: what Pascal's and Delphi's `Round` compiles to
    /// under `{$N+}` when the control word is the default. Check your runtime's `Round`: it
    /// may test the range first and stop with a run-time error.
    pub fn round_i32(self) -> i32 {
        self.fist_i32(Control::DEFAULT)
    }

    /// `fistp dword` toward zero: what `Trunc` does after it loads the chop control word.
    pub fn trunc_i32(self) -> i32 {
        self.fist_i32(Control::DEFAULT.with_rounding(Rounding::Zero))
    }
}
