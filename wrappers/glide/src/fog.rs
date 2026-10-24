//! The `gu` fog tables: the *w* of each of the 64 entries (`guFogTableIndexToW`) and the three
//! generators. Plain Rust, tested on the host.
//!
//! *Measured*: these give the same 64 bytes as Glide 2.43's own functions (the Win95
//! `glide2x.dll`, run under `tools/emu/pe32_emu.py`) for 15 densities and 7 linear ranges.
//! That needs the same rounding: the scale and each value are rounded to `float` before the
//! next step, as the original stores them, and the final value is truncated to a byte.

pub const TABLE_SIZE: usize = 64;

/// The *w* of entry `i`: 2^(3 + i/4) / (8 − i mod 4), so entry 0 is *w* = 1 and the entries
/// step four times per doubling of *w*.
pub fn index_to_w(i: usize) -> f64 {
    2f64.powi(3 + (i >> 2) as i32) / (8 - (i & 3)) as f64
}

fn f32r(v: f64) -> f64 {
    v as f32 as f64
}

/// A value 0–1 to a table byte: above 1 is 1, below 0 is 0, then ×255 truncated.
fn byte(v: f64) -> u8 {
    let v = if v > 1.0 { 1.0 } else { f32r(v) };
    (v.max(0.0) * 255.0) as u8
}

/// The exponential curves, scaled so that the last entry is full fog.
fn curve(density: f32, f: impl Fn(f64) -> f64) -> [u8; TABLE_SIZE] {
    let d = density as f64;
    let g = |i: usize| 1.0 - (-f(d * index_to_w(i))).exp();
    let scale = f32r(1.0 / g(TABLE_SIZE - 1));
    std::array::from_fn(|i| byte(g(i) * scale))
}

/// `guFogGenerateExp`: fog = 1 − e^(−density · w).
pub fn exp(density: f32) -> [u8; TABLE_SIZE] {
    curve(density, |x| x)
}

/// `guFogGenerateExp2`: fog = 1 − e^(−(density · w)²).
pub fn exp2(density: f32) -> [u8; TABLE_SIZE] {
    curve(density, |x| x * x)
}

/// `guFogGenerateLinear`: none nearer than `near`, full beyond `far`, linear between. A *w*
/// above 65535 counts as 65535.
pub fn linear(near: f32, far: f32) -> [u8; TABLE_SIZE] {
    let range = f32r(far as f64 - near as f64);
    std::array::from_fn(|i| {
        let w = index_to_w(i);
        let w = if w > 65535.0 { 65535.0 } else { f32r(w) };
        byte((w - near as f64) / range)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w_of_each_entry() {
        assert_eq!(index_to_w(0), 1.0);
        assert_eq!(index_to_w(1), 8.0 / 7.0);
        assert_eq!(index_to_w(4), 2.0);
        assert_eq!(index_to_w(63), 262144.0 / 5.0);
    }

    #[test]
    fn exponential_tables_end_full() {
        for d in [0.0001f32, 0.003, 0.2] {
            let t = exp(d);
            assert!(t.windows(2).all(|p| p[0] <= p[1]), "exp {d} rises");
            assert!(t[63] >= 254);
            let t2 = exp2(d);
            assert!(t2.windows(2).all(|p| p[0] <= p[1]), "exp2 {d} rises");
        }
        // The last entry's own value times its scale rounds below 1 here, as in the original.
        assert_eq!(exp(0.0001)[63], 254);
        assert_eq!(exp(0.01)[0], 2); // w = 1: 1 - e^-0.01 is 0.00995 of 255
        assert_eq!(exp(0.01)[44], 255);
    }

    #[test]
    fn linear_table() {
        let t = linear(1.0, 1000.0);
        assert_eq!(t[0], 0);
        assert_eq!(t[63], 255);
        // Entry 20: w = 2^8 / 8 = 32; (32 - 1) / 999 of 255 is 7.9.
        assert_eq!(t[20], 7);
        assert_eq!(linear(100.0, 200.0)[27], 6); // w = 2^9 / 5 = 102.4: 0.024 of 255
        assert_eq!(linear(100.0, 200.0)[26], 0); // w = 2^9 / 6 = 85.3: nearer than near
    }
}
