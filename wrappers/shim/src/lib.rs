//! What every wrapper DLL shares (`glide`, `ati3dcif`, `s3dtkw`, `ddraw`, `sgl`).
//!
//! - `config`: the settings they share (the game's window class, the marker colour, the
//!   patch file), and their parsers. Plain Rust, tested on the host.
//! - `ports`: Windows 95 port I/O (a Cyrix CPU check's, for example), which Wine and Windows
//!   NT stop, emulated by an exception handler; each DLL installs it from its `DllMain`.
//! - `patch`: the game's own code patched in memory at load, from the file `WRAPPER_PATCHES`
//!   names, so the EXE on disk stays unchanged.
//! - `log`: a log to the file a DLL names by an environment variable, or to standard error.

pub mod config;
#[cfg(windows)]
pub mod log;
#[cfg(windows)]
pub mod patch;
#[cfg(windows)]
pub mod ports;
