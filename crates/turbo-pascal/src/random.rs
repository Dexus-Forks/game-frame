//! Turbo Pascal's random number generator: `RandSeed`, `Random`, `Random(range)` and
//! `Randomize`.
//!
//! `RandSeed` is a 32-bit value. Each draw sets `RandSeed := RandSeed * $08088405 + 1`.
//! `Random(range)` returns the high 32 bits of the new seed times `range`. `Random` without a
//! range is [`crate::Real48::from_seed`].
//!
//! ```
//! use gameframe_turbo_pascal::TurboRandom;
//!
//! let mut r = TurboRandom::new(0);
//! assert_eq!(r.next_rand(), 1);
//! assert!(r.random(100) < 100);
//! ```
//!
//! Original: `NextRand`, `Random(range: Word): Word` and `Randomize` in the System unit (in
//! Tyrian 2000 at `1040:19b6`, `1040:1959` and `1040:19ee`). In Tyrian 2026, 90,000 draws
//! (10 seeds × 9 ranges × 1,000) match the original bit for bit (*measured*).

/// The generator's whole state: `System.RandSeed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TurboRandom {
    pub seed: u32,
}

impl TurboRandom {
    pub const MULTIPLIER: u32 = 0x0808_8405; // 134775813; the low word sits at 1040:19ec

    pub fn new(seed: u32) -> Self {
        TurboRandom { seed }
    }

    /// `NextRand` (1040:19b6): RandSeed := RandSeed * $08088405 + 1, modulo 2^32.
    pub fn next_rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(1);
        self.seed
    }

    /// `Random(range)` (1040:1959): the new 32-bit seed times `range`, high 32 bits.
    /// A range of 0 returns 0, as the original's multiply does.
    pub fn random(&mut self, range: u16) -> u16 {
        let s = self.next_rand() as u64;
        ((s * range as u64) >> 32) as u16
    }

    /// `Randomize` (1040:19ee): seed from DOS's time of day, INT 21h AH=2Ch.
    /// RandSeed low word := CX (hours, minutes), high word := DX (seconds, hundredths).
    pub fn randomize(&mut self, cx: u16, dx: u16) {
        self.seed = (dx as u32) << 16 | cx as u32;
    }
}
