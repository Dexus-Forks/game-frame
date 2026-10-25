#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! AdLib Gold audio-card ports and Control Chip behavior around Yamaha YMF262 and YMZ263B.

mod card;
mod control;
mod mixer;

pub use card::{AdlibGold, AdlibGoldConfig, AdlibGoldPortError};
pub use control::{
    BoardIdentity, BusModel, ControlChipError, ControlTiming, InterruptSource,
    CONTROL_REGISTER_BUSY_CLOCKS, CONTROL_REGISTER_COUNT, MIXER_REGISTER_BUSY_CLOCKS,
    OPL3_MASTER_CLOCK_HZ,
};
pub use gameframe_opl3::{Opl3, Opl3Output, StereoSample, OPL3_SAMPLE_RATE};
pub use gameframe_ymz263::{MmaOutputEvent, MmaSampleOutput, Ymz263};
pub use mixer::{AdlibGoldAudioInputs, AdlibGoldMixFrame, OutputMode, StereoSignal, ToneSettings};
