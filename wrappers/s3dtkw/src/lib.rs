//! `S3DTKW.dll`: S3's S3D Toolkit for the ViRGE, made for POD's S3 build (`WPODS3D.EXE`).
//! That game imports four calls and draws through the function table `S3DTK_CreateRenderer`
//! fills (POD's `docs/hardware/3d-cards.md`). This library draws the triangles in software into the
//! game's DirectDraw surfaces.

#![cfg(windows)]

use std::ffi::c_void;

mod api;
mod draw;
mod table;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        shim::log::init("s3dtk", "S3DTK_LOG");
        shim::ports::install();
        shim::patch::apply();
    }
    1
}
