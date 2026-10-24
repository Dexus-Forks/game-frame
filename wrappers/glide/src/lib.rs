//! `glide.dll`: the Glide 2.x library, drawn with OpenGL at any output size. It was made for
//! POD's 3dfx builds (`WPOD3DFX.EXE`). It implements the 55 functions they call and
//! exports all 156 of Glide 2.43 and 2.1.1, under their decorated `__stdcall` names
//! (`glide2x.def`, written with `stubs` from `api.tsv`); how the game uses each is in POD's
//! `docs/hardware/3d-cards.md`.
//!
//! - `api`: the exported calls. They record commands (`cmd`) and answer queries.
//! - `render`: the thread that owns the OpenGL context and replays the commands.
//! - `window`: the game's window, made the drawing surface.
//! - `shim` (a crate shared with the other replacement DLLs): settings, a CPU check's port
//!   I/O, which Windows 95 allowed, emulated; patches from a file; the log.

#![cfg(windows)]
#![allow(clippy::missing_safety_doc)]

mod api;
mod cmd;
mod config;
mod consts;
mod gl;
mod log;
mod render;
mod win32;
mod window;
mod stubs;

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
