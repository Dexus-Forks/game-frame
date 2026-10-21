//! Turbo Pascal's 6-byte `Real` and the arithmetic of its runtime library, bit for bit.
//!
//! Byte 0 is the exponent (bias 129; 0 means zero), bytes 1 to 5 the mantissa, low byte first,
//! with the sign in the top bit of byte 5 and the leading 1 implied. Programs compiled without
//! the 8087 option do all `Real` arithmetic in the System unit's routines. Their rounding is
//! not IEEE rounding, so a port must use these routines to get the same results.
//!
//! The addresses in the comments are the routines in the System unit as linked into Tyrian 2000
//! (segment 1040). In the Tyrian 2026 project every routine here matches the original on 40,000
//! random operands, and multiplication and `Round` on every 16-bit integer times the game's
//! constants (*measured* under the harness's DOS emulator). `div` is measured only for divisors
//! of 2.0; for other divisors its rounding is *inferred* from the multiply's.
//!
//! ```
//! use gameframe_turbo_pascal::Real48;
//!
//! let x = Real48::from_i32(3).mul(Real48::from_f64(0.75));
//! assert_eq!(x.round(), 2); // 2.25
//! assert_eq!(Real48::from_i32(5).div(Real48::from_i32(2)).round(), 3); // halves away from zero
//! ```

/// A Turbo Pascal 6-byte Real, kept as its bytes so values stay exact; `to_f64` converts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Real48(pub [u8; 6]);

impl Real48 {
    /// A Real from a double, rounded to the nearest 40-bit mantissa, halves up. Zero for values
    /// too small for a Real; the largest Real for values too large. For writing constants: the
    /// compiler's own conversion of source literals may differ in the last bit (*inferred*).
    pub fn from_f64(x: f64) -> Real48 {
        if x == 0.0 || x.is_nan() {
            return Real48([0; 6]);
        }
        let neg = x < 0.0;
        let bits = x.abs().to_bits();
        let mut exp = ((bits >> 52) & 0x7ff) as i32 - 1023;
        let mut mant = (bits & ((1u64 << 52) - 1)) | (1u64 << 52); // 53 bits, leading 1
        if (bits >> 52) & 0x7ff == 0 {
            return Real48([0; 6]); // subnormal doubles are far below the smallest Real
        }
        // Keep 40 bits (the leading 1 and 39 fraction bits), rounding halves up.
        mant = (mant + (1u64 << 12)) >> 13;
        if mant >> 40 != 0 {
            mant >>= 1;
            exp += 1;
        }
        let e = exp + 129;
        if e <= 0 {
            return Real48([0; 6]);
        }
        if e > 255 {
            let mut b = [0xff; 6];
            b[5] = if neg { 0xff } else { 0x7f };
            return Real48(b);
        }
        let frac = mant & ((1u64 << 39) - 1);
        let mut b = [0u8; 6];
        b[0] = e as u8;
        for (k, byte) in b[1..6].iter_mut().enumerate() {
            *byte = (frac >> (8 * k)) as u8;
        }
        if neg {
            b[5] |= 0x80;
        }
        Real48(b)
    }

    pub fn to_f64(self) -> f64 {
        let b = self.0;
        if b[0] == 0 {
            return 0.0;
        }
        let mut mant: u64 = 0;
        for i in (1..6).rev() {
            mant = (mant << 8) | b[i] as u64;
        }
        mant &= (1u64 << 39) - 1;
        let sign = if b[5] & 0x80 != 0 { -1.0 } else { 1.0 };
        sign * (1.0 + mant as f64 / (1u64 << 39) as f64) * 2f64.powi(b[0] as i32 - 129)
    }
}



/// The 40-bit significand (with the leading 1) and the exponent byte; None for zero.
fn parts(r: Real48) -> Option<(bool, u64, i32)> {
    let b = r.0;
    if b[0] == 0 {
        return None;
    }
    let mut m = 0u64;
    for i in (1..6).rev() {
        m = (m << 8) | b[i] as u64;
    }
    Some((b[5] & 0x80 != 0, (m & ((1 << 39) - 1)) | (1 << 39), b[0] as i32))
}

fn build(neg: bool, sig: u64, exp: i32) -> Real48 {
    if exp <= 0 {
        return Real48([0; 6]);
    }
    let m = (sig & ((1 << 39) - 1)) | if neg { 1 << 39 } else { 0 };
    let mut b = [exp as u8, 0, 0, 0, 0, 0];
    for (i, byte) in b[1..].iter_mut().enumerate() {
        *byte = (m >> (8 * i)) as u8;
    }
    Real48(b)
}

impl Real48 {
    /// Longint to Real (1040:14c0). Exact: a longint fits in the mantissa.
    pub fn from_i32(x: i32) -> Real48 {
        if x == 0 {
            return Real48([0; 6]);
        }
        let a = x.unsigned_abs() as u64;
        let bits = 64 - a.leading_zeros() as i32; // a is in [2^(bits-1), 2^bits)
        build(x < 0, a << (40 - bits), 128 + bits)
    }

    /// Real times Real (1040:14ac): the 80-bit product, rounded to 40 bits, halves up.
    pub fn mul(self, other: Real48) -> Real48 {
        let (Some((na, a, ea)), Some((nb, b, eb))) = (parts(self), parts(other)) else {
            return Real48([0; 6]);
        };
        let p = a as u128 * b as u128; // in [2^78, 2^80)
        let mut exp = ea + eb - 129;
        let shift = if p >> 79 != 0 { exp += 1; 40 } else { 39 };
        let mut sig = ((p + (1u128 << (shift - 1))) >> shift) as u64;
        if sig >> 40 != 0 {
            sig >>= 1;
            exp += 1;
        }
        build(na != nb, sig, exp)
    }

    /// Real divided by Real (1040:14b2): the quotient rounded to 40 bits, halves up.
    /// Measured only for divisions by 2.0 (exact); other divisors follow the multiply's
    /// rounding, which is inferred. Division by zero gives zero here (the original stops with
    /// a run-time error).
    pub fn div(self, other: Real48) -> Real48 {
        let (Some((na, a, ea)), Some((nb, b, eb))) = (parts(self), parts(other)) else {
            return Real48([0; 6]);
        };
        // a / b is in (1/2, 2): take 42 bits of it, then round to 40.
        let q = ((a as u128) << 41) / b as u128;
        let mut exp = ea - eb + 129;
        let (mut sig, shift) = if q >> 41 != 0 { (q, 2) } else { exp -= 1; (q, 1) };
        sig = (sig + (1 << (shift - 1))) >> shift;
        if sig >> 40 != 0 {
            sig >>= 1;
            exp += 1;
        }
        build(na != nb, sig as u64, exp)
    }

    /// Real plus Real (1040:149a, the near routine at 1040:1189). The operand with the smaller
    /// exponent is shifted right into 40 bits plus one guard byte, and the bits shifted past
    /// the guard byte are lost before the add or subtract; then halves of the guard byte
    /// round up. Not the exactly rounded sum: a subtraction can differ in the last bit.
    pub fn add(self, other: Real48) -> Real48 {
        // A zero exponent is zero, whatever the other bytes: the second zero gives the first
        // back as it is, the first zero gives the second.
        if other.0[0] == 0 {
            return self;
        }
        if self.0[0] == 0 {
            return other;
        }
        let (Some(a), Some(b)) = (parts(self), parts(other)) else { unreachable!() };
        // After the swap, `small` has the smaller (or equal) exponent.
        let (small, big) = if a.2 > b.2 { (b, a) } else { (a, b) };
        let d = big.2 - small.2;
        if d >= 41 {
            return if a.2 > b.2 { self } else { other };
        }
        let (sign_small, ms, _) = small;
        let (_, mb, eb) = big;
        let x = (ms << 8) >> d; // 48 bits: the mantissa and a guard byte, truncated
        let y = mb << 8;
        let mut exp = eb;
        let mut sign = sign_small;
        let mut s: u64;
        if small.0 == big.0 {
            s = x + y;
            if s >> 48 != 0 {
                s >>= 1;
                exp += 1;
                if exp > 255 {
                    return Real48([0; 6]); // overflow: the original stops with run-time error 205
                }
            }
        } else {
            let (diff, borrow) = x.overflowing_sub(y);
            s = diff & ((1 << 48) - 1);
            if borrow {
                s = s.wrapping_neg() & ((1 << 48) - 1);
                sign = !sign;
            }
            if s == 0 {
                return Real48([0; 6]);
            }
            while s >> 47 == 0 {
                s <<= 1;
                exp -= 1;
                if exp == 0 {
                    return Real48([0; 6]);
                }
            }
        }
        s += 0x80;
        if s >> 48 != 0 {
            s = 1 << 47;
            exp += 1;
        }
        build(sign, (s >> 8) & ((1 << 40) - 1), exp)
    }

    /// Real minus Real (1040:14a0): the second operand's sign flipped, then `add`.
    pub fn sub(self, other: Real48) -> Real48 {
        let mut b = other;
        if b.0[0] != 0 {
            b.0[5] ^= 0x80;
        }
        self.add(b)
    }

    pub fn neg(self) -> Real48 {
        let mut r = self;
        if r.0[0] != 0 {
            r.0[5] ^= 0x80;
        }
        r
    }

    fn negative(self) -> bool {
        self.0[0] != 0 && self.0[5] & 0x80 != 0
    }

    /// Compare (1040:13c6).
    pub fn cmp(self, other: Real48) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        let (na, nb) = (self.0[5] & 0x80 != 0, other.0[5] & 0x80 != 0);
        if na != nb {
            return if na { Ordering::Less } else { Ordering::Greater };
        }
        // The same sign bit: compare exponent, then mantissa from the top byte down.
        let key = |r: Real48| {
            let b = r.0;
            (b[0], if b[0] == 0 { 0 } else { u64::from_le_bytes([b[1], b[2], b[3], b[4], b[5], 0, 0, 0]) })
        };
        let o = key(self).cmp(&key(other));
        if na && o != Ordering::Equal { o.reverse() } else { o }
    }

    /// `Int` (1040:150e): the integer part, towards zero.
    pub fn int_part(self) -> Real48 {
        let e = self.0[0];
        if e >= 0xa8 {
            return self;
        }
        if e <= 0x80 {
            return Real48([0; 6]);
        }
        // Keep the sign and the top e - 0x81 bits of the stored mantissa.
        let keep = (e - 0x80) as u32; // bits counted from the sign bit
        let m = u64::from_le_bytes([self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], 0, 0, 0]);
        let mask = !((1u64 << (40 - keep)) - 1) & ((1 << 40) - 1);
        let k = (m & mask).to_le_bytes();
        Real48([e, k[0], k[1], k[2], k[3], k[4]])
    }

    /// `Frac` (1040:155f): x - Int(x).
    pub fn frac(self) -> Real48 {
        self.sub(self.int_part())
    }

    /// `Random` without a range (1040:1976): the next seed (`TurboRandom::next_rand`) as a
    /// Real in [0, 1): seed / 2^32.
    pub fn from_seed(seed: u32) -> Real48 {
        if seed == 0 {
            return Real48([0; 6]);
        }
        let shift = seed.leading_zeros();
        let m = ((seed << shift) as u64) << 8; // 40 bits: the seed's 32, then a zero byte
        build(false, m & ((1 << 39) - 1), (0x80 - shift) as i32)
    }

    /// `Sin` (1040:15e5): reduced to [0, π/2] by 2π (with `Frac`), π and π/2, then
    /// x * (1 + z * P(z)) with z = x², the coefficients from 1040:1646.
    pub fn sin(self) -> Real48 {
        let mut x = self;
        if x.0[0] < 0x6c {
            return x;
        }
        let two_pi = Real48([0x83, 0x21, 0xa2, 0xda, 0x0f, 0x49]);
        if x.abs().cmp(two_pi) != core::cmp::Ordering::Less {
            x = x.div(two_pi).frac().mul(two_pi);
        }
        if x.negative() {
            x = x.add(two_pi);
        }
        let pi = Real48([0x82, 0x21, 0xa2, 0xda, 0x0f, 0x49]);
        let below_pi = x.cmp(pi) == core::cmp::Ordering::Less;
        if !below_pi {
            x = x.sub(pi);
        }
        let half_pi = Real48([0x81, 0x21, 0xa2, 0xda, 0x0f, 0x49]);
        if x.cmp(half_pi) != core::cmp::Ordering::Less {
            x = pi.add(x.neg());
        }
        if x.0[0] >= 0x6c {
            x = poly(x);
        }
        if !below_pi && x.0[0] != 0 {
            x = x.neg();
        }
        x
    }

    /// `Cos` (1040:15d2): Sin(π/2 - x), as x + (-π/2) negated.
    pub fn cos(self) -> Real48 {
        let minus_half_pi = Real48([0x81, 0x21, 0xa2, 0xda, 0x0f, 0xc9]);
        let x = self.add(minus_half_pi);
        (if x.0[0] != 0 { x.neg() } else { x }).sin()
    }

    fn abs(self) -> Real48 {
        let mut r = self;
        r.0[5] &= 0x7f;
        r
    }

    /// Trunc (1040:14c4): towards zero.
    pub fn trunc(self) -> i32 {
        let Some((neg, sig, exp)) = parts(self) else { return 0 };
        let shift = 168 - exp;
        let n = if shift <= 0 { (sig << -shift) as i64 } else if shift > 40 { 0 } else { (sig >> shift) as i64 };
        (if neg { -n } else { n }) as i32
    }

    /// Round (1040:14cc): to the nearest integer, halves away from zero.
    pub fn round(self) -> i32 {
        let Some((neg, sig, exp)) = parts(self) else { return 0 };
        let shift = 168 - exp; // value = sig * 2^(exp - 168)
        let n = if shift <= 0 {
            (sig << -shift) as i64
        } else if shift > 41 {
            0
        } else {
            ((sig as u128 + (1u128 << (shift - 1))) >> shift) as i64
        };
        (if neg { -n } else { n }) as i32
    }
}

/// The coefficients of `Sin`'s polynomial (1040:1646), highest power first.
const SIN_COEFFS: [Real48; 7] = [
    Real48([0x58, 0x9d, 0x39, 0x9f, 0x3f, 0xd7]),
    Real48([0x60, 0x43, 0x9d, 0x30, 0x92, 0x30]),
    Real48([0x67, 0xaa, 0x3f, 0x28, 0x32, 0xd7]),
    Real48([0x6e, 0xb6, 0x2a, 0x1d, 0xef, 0x38]),
    Real48([0x74, 0x0d, 0xd0, 0x00, 0x0d, 0xd0]),
    Real48([0x7a, 0x88, 0x88, 0x88, 0x88, 0x08]),
    Real48([0x7e, 0xab, 0xaa, 0xaa, 0xaa, 0xaa]),
];

/// 1040:18f1 with the table above: x * (((c0 z + c1) z + ... + c6) z + 1), z = x * x.
fn poly(x: Real48) -> Real48 {
    let z = x.mul(x);
    let mut acc = SIN_COEFFS[0].mul(z);
    for c in &SIN_COEFFS[1..] {
        acc = acc.add(*c).mul(z);
    }
    acc.add(Real48([0x81, 0, 0, 0, 0, 0])).mul(x)
}
