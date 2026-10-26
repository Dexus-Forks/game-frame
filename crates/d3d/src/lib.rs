#![doc = include_str!("../README.md")]
#![doc = ""]
//! Direct3D 3 immediate mode in software: [`exec`] for execute buffers, [`Device`] to run
//! them into an RGB565 target at any multiple of the game's resolution.

pub mod consts;
pub mod device;
pub mod exec;
pub mod pixel;
pub mod raster;
pub mod state;
pub mod texture;

pub use device::{Device, Skipped};
pub use exec::{Builder, ExecuteData, Instruction, TlVertex};
pub use state::States;
pub use texture::{Format, Texture};
