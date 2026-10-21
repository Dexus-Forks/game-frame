//! The Microsoft C runtime's `rand` and `srand`: a linear congruential generator.
//!
//! The state is a 32-bit seed. Each call sets `seed = seed * 214013 + 2531011` and returns bits
//! 16 to 30 of the new seed, a value from 0 to 32,767 (`RAND_MAX`). `srand` sets the seed; a
//! program that never calls it starts from 1. The runtime keeps one seed per thread.
//!
//! ```
//! use gameframe_msvc_crt::rand::Rand;
//!
//! let mut r = Rand::new(1);
//! assert_eq!([r.rand(), r.rand(), r.rand()], [41, 18467, 6334]);
//! ```
//!
//! Original: MSVC `rand` and `srand` (documented behaviour; the constants are widely published).

/// The largest value [`Rand::rand`] returns.
pub const RAND_MAX: i32 = 0x7fff;

/// One thread's `rand` state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rand {
    /// The seed, as `srand` sets it and `rand` advances it.
    pub seed: u32,
}

impl Default for Rand {
    /// The state of a program that never called `srand`.
    fn default() -> Self {
        Rand { seed: 1 }
    }
}

impl Rand {
    /// The state after `srand(seed)`.
    pub fn new(seed: u32) -> Self {
        Rand { seed }
    }

    /// `srand(seed)`.
    pub fn srand(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// `rand()`: the next value, from 0 to [`RAND_MAX`].
    pub fn rand(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(214_013).wrapping_add(2_531_011);
        ((self.seed >> 16) & 0x7fff) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_values_from_seed_1() {
        let mut r = Rand::default();
        let v: Vec<i32> = (0..10).map(|_| r.rand()).collect();
        assert_eq!(v, [41, 18467, 6334, 26500, 19169, 15724, 11478, 29358, 26962, 24464]);
    }

    #[test]
    fn srand_restarts_the_sequence() {
        let mut r = Rand::new(12345);
        let a: Vec<i32> = (0..5).map(|_| r.rand()).collect();
        r.srand(12345);
        let b: Vec<i32> = (0..5).map(|_| r.rand()).collect();
        assert_eq!(a, b);
        assert!(a.iter().all(|&x| (0..=RAND_MAX).contains(&x)));
    }
}
