//! The town's clock. Original: `FUN_1400c9460` (a module note, not a label).

/// Advances the clock by one step.
/// Original: `FUN_1400c9560`
pub fn tick(clock: &mut Clock) {
    clock.ticks += 1;
}

/// The clock.
/// Original: `FUN_1400c9d70`, `FUN_1400c9e00`
pub struct Clock {
    pub ticks: u64,
}

impl Clock {
    /// Seconds since the start.
    /// Original: `FUN_1400ca000`
    #[inline]
    pub fn seconds(&self) -> f64 {
        self.ticks as f64 / 60.0
    }

    /// Original: `FUN_1400ca100`
    pub fn reset(&mut self) {
        self.ticks = 0;
    }
}
