//! `glide.dll`: the Glide 2.x library, drawn with OpenGL at any output size. It was made for
//! POD's 3dfx builds (`WPOD3DFX.EXE`). It implements and exports all 156 functions of
//! Glide 2.43 and 2.1.1, under their decorated `__stdcall` names
//! (`glide2x.def`, written with `stubs` from `api.tsv`); how the game uses each is in POD's
//! `docs/hardware/3d-cards.md`.
//!
//! - `vertex`, `layout`, `fog`, `tdf`, `rle`: vertices (culling, fans, clipping), where mip
//!   levels lie in texture memory, the fog tables, 3DF files and `guEncodeRLE16`'s code;
//!   plain Rust, tested on the host.
//! - `api`: the exported calls. They record commands (`cmd`) and answer queries. Each first
//!   writes itself to the call log when `WRAPPER_CALLS` is set (`callargs`: the values its
//!   pointers point to).
//! - `render`: the thread that owns the OpenGL context and replays the commands.
//! - `window`: the game's window, made the drawing surface.
//! - `shim` (a crate shared with the other replacement DLLs): settings, a CPU check's port
//!   I/O, which Windows 95 allowed, emulated; patches from a file; the log.

#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

// Plain Rust, tested on the host.
pub mod fog;
pub mod layout;
pub mod rle;
pub mod tdf;
pub mod vertex;

#[cfg(windows)]
mod api;
#[cfg(windows)]
mod callargs;
#[cfg(windows)]
mod cmd;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod consts;
#[cfg(windows)]
mod gl;
#[cfg(windows)]
mod log;
#[cfg(windows)]
mod render;
#[cfg(windows)]
mod win32;
#[cfg(windows)]
mod window;
#[cfg(windows)]
mod stubs;

#[cfg(windows)]
/// Called by the C runtime's DLL entry point: the port I/O handler goes in before the game's
/// own code runs.
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut std::ffi::c_void, reason: u32, _reserved: *mut std::ffi::c_void) -> i32 {
    if reason == 1 {
        shim::log::init("glide", "GLIDE_LOG");
        shim::ports::install();
        shim::patch::apply();
    }
    if reason == 0 {
        shim::log::summary();
    }
    1
}
