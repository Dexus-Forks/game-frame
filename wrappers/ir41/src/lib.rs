//! `ir41_32.dll`: a Video for Windows codec for Intel Indeo Video Interactive 4.1 (`IV41`), the
//! file name and the one export (`DriverProc`) of Intel's own codec, so that a game that checks
//! for Intel's file finds it and Video for Windows plays its videos through it. The pictures come
//! from GameFrame's clean-room decoder (`crates/indeo`), equal to Intel's own codec's IF09 output
//! picture for picture (measured on POD Gold's six videos).
//!
//! - `formats`: the formats it takes (`IV41`) and gives (24- and 32-bit RGB, 16-bit 555 and
//!   565, `IF09` and `YVU9`), and the planes written in each.
//! - `tables`: where it finds Intel's tables (they are Intel's data, read from the player's
//!   own copy of Intel's DLL).
//! - `dll`: `DriverProc`, the driver and `ICM_DECOMPRESS*` messages.

pub mod formats;
pub mod tables;

#[cfg(windows)]
mod dll;
