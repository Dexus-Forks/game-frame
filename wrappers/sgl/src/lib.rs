//! `sgl.dll`: a stand-in for PowerVR's SGL library, the scene-graph half. It was made for POD
//! Gold's PowerVR build (`WPODPVR.EXE`), which imports 46 SGL calls (`sgl.def`); it builds a scene graph of
//! display lists, meshes, materials, lights and cameras, and asks SGL to render it into the
//! DirectDraw surface its address callback hands over. This library keeps the graph and draws
//! it with OpenGL at any resolution. How the game uses each call is in
//! POD's `docs/hardware/3d-cards.md` ("The PowerVR build").
//!
//! - `scene`: the graph and its traversal (plain Rust, tested on the host).
//! - `api`: the exported calls; they read the game's pointers and edit the graph.
//! - `frame`: `sgl_render`: the address callback, the marker in the game's surface, and the
//!   frame handed to the render thread.
//! - `render`: the thread that owns the OpenGL context, and the window.
//! - `shim` (a crate shared with the other wrapper DLLs): the log and the in-memory patches.

#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

pub mod scene;

#[cfg(windows)]
mod api;
#[cfg(windows)]
mod frame;
#[cfg(windows)]
mod render;

#[cfg(windows)]
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut std::ffi::c_void, reason: u32, _reserved: *mut std::ffi::c_void) -> i32 {
    if reason == 1 {
        shim::log::init("sgl", "SGL_LOG");
        shim::ports::install();
        shim::patch::apply();
        api::install_ini();
    }
    1
}
