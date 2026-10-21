#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod data;
pub mod random;
pub mod real48;

pub use data::{PStr, PascalReader, PascalWriter, ShortRead};
pub use random::TurboRandom;
pub use real48::Real48;
