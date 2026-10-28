//! The extended value, its loads (`fld`, `fild`) and stores (`fst`, `fstp`).

use crate::round::{bit_len, round_shift, round_to};
use crate::{Control, Rounding};
use core::cmp::Ordering;

/// A finite value of the x87's 80-bit extended format: (−1)^sign · `mant` · 2^`exp`, with
/// `mant`'s top bit set, or a signed zero.
///
/// `PartialEq` compares representations: +0 and −0 differ. Use [`Ext::fcom`] for the x87's
/// comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ext {
    pub(crate) neg: bool,
    pub(crate) exp: i32,
    pub(crate) mant: u64,
}

/// The format of a `Double` or a `Single` in memory, for the stores.
struct Format {
    frac: u32,
    min_quantum: i32,
    bias_offset: i32,
    max_field: i32,
}

const DOUBLE: Format = Format {
    frac: 52,
    min_quantum: -1074,
    bias_offset: 1075,
    max_field: 2046,
};
const SINGLE: Format = Format {
    frac: 23,
    min_quantum: -149,
    bias_offset: 150,
    max_field: 254,
};

impl Ext {
    pub const ZERO: Ext = Ext {
        neg: false,
        exp: 0,
        mant: 0,
    };
    pub const NEG_ZERO: Ext = Ext {
        neg: true,
        exp: 0,
        mant: 0,
    };
    pub const ONE: Ext = Ext {
        neg: false,
        exp: -63,
        mant: 1 << 63,
    };

    pub(crate) fn zero(neg: bool) -> Ext {
        if neg {
            Ext::NEG_ZERO
        } else {
            Ext::ZERO
        }
    }

    /// The exact value m · 2^e (with `sticky` set when nonzero bits lie below m), rounded by
    /// `cw`'s precision and rounding. m must not be 0.
    pub(crate) fn round_from(neg: bool, e: i32, m: u128, sticky: bool, cw: Control) -> Ext {
        let bits = cw.precision.bits();
        let (r, shift) = round_to(m, sticky, bits, neg, cw.rounding);
        let pad = 64 - bits;
        Ext {
            neg,
            exp: e + shift - pad as i32,
            mant: (r << pad) as u64,
        }
    }

    /// m · 2^e, exactly (m fits in 64 bits).
    fn exact(neg: bool, e: i32, m: u128) -> Ext {
        if m == 0 {
            return Ext::zero(neg);
        }
        let n = bit_len(m);
        debug_assert!(n <= 64);
        Ext {
            neg,
            exp: e - (64 - n as i32),
            mant: (m << (64 - n)) as u64,
        }
    }

    /// `fild word`: exact.
    pub fn from_i16(v: i16) -> Ext {
        Ext::exact(v < 0, 0, u128::from(v.unsigned_abs()))
    }

    /// `fild dword`: exact.
    pub fn from_i32(v: i32) -> Ext {
        Ext::exact(v < 0, 0, u128::from(v.unsigned_abs()))
    }

    /// `fild qword` (Delphi's `Comp`): exact.
    pub fn from_i64(v: i64) -> Ext {
        Ext::exact(v < 0, 0, u128::from(v.unsigned_abs()))
    }

    /// `fld qword`: exact, denormals included.
    ///
    /// # Panics
    /// On an infinity or a NaN, which this crate does not model.
    pub fn from_f64(v: f64) -> Ext {
        assert!(
            v.is_finite(),
            "gameframe-x87 does not model infinities or NaNs"
        );
        let bits = v.to_bits();
        let field = ((bits >> 52) & 0x7ff) as i32;
        let frac = bits & ((1 << 52) - 1);
        let (m, e) = if field == 0 {
            (frac, -1074)
        } else {
            (frac | 1 << 52, field - 1075)
        };
        Ext::exact(bits >> 63 != 0, e, u128::from(m))
    }

    /// `fld dword`: exact, denormals included.
    ///
    /// # Panics
    /// On an infinity or a NaN.
    pub fn from_f32(v: f32) -> Ext {
        assert!(
            v.is_finite(),
            "gameframe-x87 does not model infinities or NaNs"
        );
        let bits = v.to_bits();
        let field = ((bits >> 23) & 0xff) as i32;
        let frac = bits & ((1 << 23) - 1);
        let (m, e) = if field == 0 {
            (frac, -149)
        } else {
            (frac | 1 << 23, field - 150)
        };
        Ext::exact(bits >> 31 != 0, e, u128::from(m))
    }

    /// `fld tbyte`: the ten bytes of an extended value in memory (Delphi's `Extended`), little
    /// endian. `None` for an infinity, a NaN or an unnormal, which this crate does not model.
    /// Denormals load exactly.
    pub fn from_bytes(b: [u8; 10]) -> Option<Ext> {
        let mut m = [0u8; 8];
        m.copy_from_slice(&b[..8]);
        let mant = u64::from_le_bytes(m);
        let se = u16::from_le_bytes([b[8], b[9]]);
        let neg = se >> 15 != 0;
        let field = i32::from(se & 0x7fff);
        match field {
            0x7fff => None,
            0 => Some(Ext::exact(neg, 1 - 16446, u128::from(mant))),
            _ if mant >> 63 == 0 => None,
            _ => Some(Ext {
                neg,
                exp: field - 16446,
                mant,
            }),
        }
    }

    /// `fstp tbyte`. `None` when the exponent lies outside the extended format's normal range,
    /// where the x87 would give a denormal, an infinity or 0 (this crate does not limit it).
    pub fn to_bytes(self) -> Option<[u8; 10]> {
        let sign = u16::from(self.neg) << 15;
        let field = if self.is_zero() { 0 } else { self.exp + 16446 };
        if !self.is_zero() && !(1..=0x7ffe).contains(&field) {
            return None;
        }
        let mut b = [0u8; 10];
        b[..8].copy_from_slice(&self.mant.to_le_bytes());
        b[8..].copy_from_slice(&(sign | field as u16).to_le_bytes());
        Some(b)
    }

    /// `fst qword` at the default rounding (to nearest), as every `Double` variable is stored.
    pub fn to_f64(self) -> f64 {
        self.to_f64_with(Control::DEFAULT)
    }

    /// `fst qword` under `cw`'s rounding (stores ignore the precision): a denormal or an
    /// infinity where the x87 gives one.
    pub fn to_f64_with(self, cw: Control) -> f64 {
        f64::from_bits(self.store(&DOUBLE, cw.rounding) | u64::from(self.neg) << 63)
    }

    /// `fst dword` at the default rounding, as every `Single` variable is stored.
    pub fn to_f32(self) -> f32 {
        self.to_f32_with(Control::DEFAULT)
    }

    /// `fst dword` under `cw`'s rounding.
    pub fn to_f32_with(self, cw: Control) -> f32 {
        f32::from_bits(self.store(&SINGLE, cw.rounding) as u32 | u32::from(self.neg) << 31)
    }

    /// The bits of the stored value without its sign.
    fn store(self, f: &Format, rc: Rounding) -> u64 {
        if self.is_zero() {
            return 0;
        }
        // The quantum: the value of the stored last bit, fixed at the bottom of the denormals.
        let top = self.exp + 63;
        let mut q = (top - f.frac as i32).max(f.min_quantum);
        let shift = (q - self.exp).min(200) as u32;
        let mut k = round_shift(u128::from(self.mant), shift, false, self.neg, rc);
        if k == 0 {
            return 0;
        }
        if bit_len(k) == f.frac + 2 {
            k >>= 1;
            q += 1;
        }
        if bit_len(k) <= f.frac {
            return k as u64; // a denormal
        }
        let field = q + f.bias_offset;
        if field > f.max_field {
            let to_infinity = match rc {
                Rounding::Nearest => true,
                Rounding::Zero => false,
                Rounding::Up => !self.neg,
                Rounding::Down => self.neg,
            };
            let largest = (f.max_field as u64) << f.frac | ((1 << f.frac) - 1);
            return if to_infinity { largest + 1 } else { largest };
        }
        (field as u64) << f.frac | (k as u64 & ((1 << f.frac) - 1))
    }

    pub fn is_zero(self) -> bool {
        self.mant == 0
    }

    /// The sign bit (set for −0 too).
    pub fn is_sign_negative(self) -> bool {
        self.neg
    }

    /// `fchs`.
    pub fn fchs(self) -> Ext {
        Ext {
            neg: !self.neg,
            ..self
        }
    }

    /// `fabs`.
    pub fn fabs(self) -> Ext {
        Ext { neg: false, ..self }
    }

    /// `fcom`: the order of two values; +0 and −0 are equal.
    pub fn fcom(self, o: Ext) -> Ordering {
        match (self.is_zero(), o.is_zero()) {
            (true, true) => return Ordering::Equal,
            (true, false) => {
                return if o.neg {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (false, true) => {
                return if self.neg {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            _ => {}
        }
        if self.neg != o.neg {
            return if self.neg {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        let magnitude = (self.exp, self.mant).cmp(&(o.exp, o.mant));
        if self.neg {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}
