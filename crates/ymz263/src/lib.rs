#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Yamaha YMZ263B multimedia audio and game interface controller.

mod adpcm;
mod channel;
mod chip;
mod fifo;
mod midi;
mod timers;

pub use channel::{ChannelOutput, PcmDataFormat, SamplingRate};
pub use chip::{
    MmaAccessError, MmaOutputEvent, MmaPortResult, MmaSampleOutput, Ymz263, Ymz263PortError,
    YMZ263_MASTER_CLOCK_HZ, YMZ263_PORT_ACCESS_CLOCKS,
};
pub use midi::{MidiLineEvent, MIDI_BAUD_RATE};

/// Size of each PCM/ADPCM channel FIFO, in bytes.
pub const PCM_FIFO_CAPACITY: usize = 128;
/// Size of each MIDI FIFO, in bytes.
pub const MIDI_FIFO_CAPACITY: usize = 16;
/// Number of sample channels.
pub const PCM_CHANNELS: usize = 2;
/// Number of programmable MMA timers.
pub const MMA_TIMERS: usize = 3;
