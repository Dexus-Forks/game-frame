//! What every wrapper DLL shares (`glide`, `ati3dcif`, `s3dtkw`, `ddraw`, `sgl`, `maxi94`).
//!
//! - `calls`: the call log, every call the game makes, one line each, to the file
//!   `WRAPPER_CALLS` names (`call!`), for comparing the original's calls with a port's.
//! - `config`: the settings they share (the game's window class, the marker colour, the
//!   patch file), and their parsers. Plain Rust, tested on the host.
//! - `ports`: Windows 95 port I/O (a Cyrix CPU check's, for example), which Wine and Windows
//!   NT stop, emulated by an exception handler; each DLL installs it from its `DllMain`.
//! - `patch`: the game's own code patched in memory at load, from the file `WRAPPER_PATCHES`
//!   names, so the EXE on disk stays unchanged.
//! - `iat`: one of the EXE's imports replaced by the wrapper's own function, to see the calls
//!   the game makes to another library.
//! - `log`: a log to the file a DLL names by an environment variable, or to standard error,
//!   and the counted log of what is not implemented yet (`stub!`, `gap!`). Plain Rust, so
//!   the scene code tested on the host can use it.

pub mod calls;
pub mod config;
#[cfg(windows)]
pub mod iat;
pub mod log;
#[cfg(windows)]
pub mod patch;
#[cfg(windows)]
pub mod ports;
