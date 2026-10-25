#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Clean Rust models of the Creative Sound Blaster DSP, mixer and SB16 MPU-401 UART ports.
//! The host supplies ISA DMA transfers and the machine's programmable interrupt controller.

mod card;
mod dsp;
mod mixer;
mod mpu;

pub use card::{SoundBlaster, SoundBlasterConfig};
pub use dsp::{
    DmaDirection, DmaRequest, Dsp, DspEvent, DspVersion, InterruptSource, PcmChannel, PcmSample,
    TimeError, TransferWidth,
};
pub use mixer::{Mixer, MixerKind};
pub use mpu::{Mpu401, MpuEvent};
