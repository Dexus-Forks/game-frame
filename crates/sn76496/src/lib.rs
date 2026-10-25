#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Texas Instruments SN76496 programmable tone/noise generator with PCjr and Tandy port mappings.
//! Register behavior follows TI documentation and the IBM PCjr and Tandy 1000 technical manuals.

mod chip;
mod tandy;

pub use chip::{Sn76496, Sn76496Sample, TANDY_PSG_CLOCK_HZ, TONE_CLOCK_DIVIDER};
pub use tandy::{PcjrPsgCard, TandyPsgCard, TANDY_PSG_WRITE_WAIT_STATES};

/// Whether a PSG machine port drives the data bus during an input read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortReadBehavior {
    /// The address is outside the PSG's decoded port range.
    Unmapped,
    /// The PSG port is write-only, so the chip does not drive a value onto the bus.
    Undriven,
}
