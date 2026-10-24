//! `ddraw.dll`: DirectDraw in memory for a 1990s full-screen game. It was made for POD's
//! DirectDraw builds (the software and MMX builds, and the ATI and S3 builds, which draw into
//! the same surfaces). The game asks for a full-screen mode such as 640 × 480 × 16; every
//! surface here is plain memory, any mode is granted, and the frame is shown in the game's
//! window at the output scale (`present`). So the game needs no EXE patch for its display, and
//! the picture can be any size. With `sgl.dll` loaded, each shown picture goes to
//! `sgl_wrapper_present` first.
//!
//! - `ddraw`: `IDirectDraw` and `IDirectDraw2`.
//! - `surface`, `palette`, `clipper`: the other interfaces.
//! - `desc`: `DDSURFACEDESC` and `DDPIXELFORMAT`.
//! - `com`: interface tables, and stubs for the methods POD does not call.

#![cfg(windows)]

mod clipper;
mod com;
mod ddraw;
mod desc;
mod palette;
mod present;
mod surface;
mod stubs;

use shim::log;
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        shim::log::init("ddraw", "DDRAW_LOG");
        shim::ports::install();
        shim::patch::apply();
    }
    if reason == 0 {
        shim::log::summary();
    }
    1
}

/// With `DDRAW_SYSTEM=1` this DLL only applies the in-memory patches (`shim::patch`) and
/// hands DirectDraw to the system's own `ddraw.dll`: the Direct3D build needs Direct3D, which
/// this DirectDraw does not have.
fn system_ddraw(name: &std::ffi::CStr) -> Option<*const c_void> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemDirectoryA(buf: *mut u8, size: u32) -> u32;
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(m: *mut c_void, name: *const u8) -> *const c_void;
    }
    if !std::env::var("DDRAW_SYSTEM").is_ok_and(|v| v == "1") {
        return None;
    }
    let mut dir = [0u8; 260];
    let n = unsafe { GetSystemDirectoryA(dir.as_mut_ptr(), 260) } as usize;
    let path = format!("{}\\ddraw.dll\0", String::from_utf8_lossy(&dir[..n]));
    let m = unsafe { LoadLibraryA(path.as_ptr()) };
    if m.is_null() {
        log!("the system's ddraw.dll ({}) did not load", path.trim_end_matches('\0'));
        return None;
    }
    let f = unsafe { GetProcAddress(m, name.as_ptr().cast()) };
    (!f.is_null()).then_some(f)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DirectDrawCreate(guid: *const u8, out: *mut *mut ddraw::DirectDraw, outer: *mut c_void) -> u32 {
    if let Some(f) = system_ddraw(c"DirectDrawCreate") {
        log!("DirectDrawCreate: the system's DirectDraw");
        let f: unsafe extern "system" fn(*const u8, *mut *mut ddraw::DirectDraw, *mut c_void) -> u32 = unsafe { std::mem::transmute(f) };
        return unsafe { f(guid, out, outer) };
    }
    log!("DirectDrawCreate (scale {})", present::scale());
    if out.is_null() {
        return com::DDERR_INVALIDPARAMS;
    }
    unsafe { out.write(ddraw::new(1)) };
    com::DD_OK
}

type EnumCallback = unsafe extern "system" fn(*const u8, *const u8, *const u8, *mut c_void) -> i32;

/// One device: the primary display.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DirectDrawEnumerateA(cb: EnumCallback, ctx: *mut c_void) -> u32 {
    if let Some(f) = system_ddraw(c"DirectDrawEnumerateA") {
        let f: unsafe extern "system" fn(EnumCallback, *mut c_void) -> u32 = unsafe { std::mem::transmute(f) };
        return unsafe { f(cb, ctx) };
    }
    unsafe { cb(std::ptr::null(), c"Display".as_ptr().cast(), c"display".as_ptr().cast(), ctx) };
    com::DD_OK
}
