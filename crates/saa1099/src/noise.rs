/// A deterministic noise source. An external hardware-measured implementation reports this
/// 18-bit polynomial; the Philips data sheet does not give the power-up state, so seed one is a
/// repeatable choice rather than a confirmed silicon default.
#[derive(Clone, Copy, Debug)]
pub(super) struct NoiseGenerator {
    phase: u64,
    state: u32,
}

impl NoiseGenerator {
    const PHASE_ONE: u64 = 1_u64 << 32;
    const STATE_MASK: u32 = (1 << 18) - 1;
    const FEEDBACK_TAPS: u32 = (1 << 17) | (1 << 10);

    pub const fn new() -> Self {
        Self { phase: 0, state: 1 }
    }

    /// Advance the generator by a Q32 number of noise clocks per output sample.
    pub fn advance(&mut self, phase_step: u64) -> bool {
        let accumulated = self.phase + phase_step;
        if accumulated >= Self::PHASE_ONE {
            self.phase = accumulated - Self::PHASE_ONE;
            self.shift();
        } else {
            self.phase = accumulated;
        }
        self.state & 1 != 0
    }

    fn shift(&mut self) {
        let feedback = self.state & 1 != 0;
        self.state >>= 1;
        if feedback {
            self.state ^= Self::FEEDBACK_TAPS;
        }
        self.state &= Self::STATE_MASK;
        if self.state == 0 {
            self.state = 1;
        }
    }
}
