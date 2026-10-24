//! `ati3dcif.dll`: ATI's 3D Rage interface (ATI3DCIF), made for POD's ATI build
//! (`WPODATI.EXE`). That game imports six calls (POD's `docs/hardware/3d-cards.md`); it uses the chip only for
//! untextured faces, drawn into the DirectDraw back buffer its software renderer also draws
//! into. This library draws those faces in software into the same surface.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::Mutex;

mod api;
mod raster;
mod state;
mod stubs;

/// `C3D_EC_OK`.
const OK: u32 = 0;

static CONTEXT: Mutex<state::Context> = Mutex::new(state::Context::new());

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        shim::log::init("ati3dcif", "ATI3DCIF_LOG");
        shim::ports::install();
        shim::patch::apply();
    }
    if reason == 0 {
        shim::log::summary();
    }
    1
}

pub(crate) fn context() -> std::sync::MutexGuard<'static, state::Context> {
    CONTEXT.lock().unwrap_or_else(|e| e.into_inner())
}

