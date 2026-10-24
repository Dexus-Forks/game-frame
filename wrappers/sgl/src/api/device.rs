//! Devices, viewports, the address callback and `sgl_render`. POD makes one screen device of
//! its resolution rounded down to SGL's 32 × 32 regions (`FUN_0049db00`, the region size read
//! from `sglhw.ini`), a viewport over it with a square camera rectangle (`FUN_0049dbe0`), and
//! in split screen two more, subtracted from the first (`FUN_0049dca0`). It renders with
//! `sgl_render(viewport, camera, swap)` (`FUN_00489000`).

use super::{sgl, trace};

#[derive(Clone, Debug, PartialEq)]
pub struct Viewport {
    /// Left, top, right, bottom in device pixels, inclusive.
    pub rect: [i32; 4],
    /// The camera rectangle: where the camera's square window lies on the device.
    pub cam: [f32; 4],
    pub subtract: Vec<i32>,
}

/// The ini file SGL's driver installer writes into the Windows folder (`MIDAS.INF`,
/// `[Midas4_UpdateInis]`). POD reads `SW_XRegionSize` and `SW_YRegionSize` from it before its
/// first SGL call (`FUN_004a8480`) and divides by them (`FUN_0049db00`): without the file it
/// divides by zero. Written here if it is missing, with the installer's `[default]` keys and
/// values (`MIDAS.INF`; *observed*, `sgl2-defaults.md`).
pub fn install_ini() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetWindowsDirectoryA(buf: *mut u8, size: u32) -> u32;
    }
    let mut dir = [0u8; 260];
    let n = unsafe { GetWindowsDirectoryA(dir.as_mut_ptr(), 260) } as usize;
    if n == 0 || n >= dir.len() {
        return;
    }
    let path = format!("{}\\sglhw.ini", String::from_utf8_lossy(&dir[..n]));
    if std::path::Path::new(&path).exists() {
        return;
    }
    let ini = "[mode]\r\nmode=ddraw\r\n[default]\r\nSW_XRegionSize=32\r\nSW_YRegionSize=32\r\nStrictLocks=0\r\nTSPParamSize=256\r\nResetTSPVSync=1\r\nRenderTimeout=5\r\n";
    match std::fs::write(&path, ini) {
        Ok(()) => shim::log!("wrote {path} (the PowerVR installer's settings)"),
        Err(e) => shim::log!("cannot write {path}: {e}"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_screen_device(device_number: i32, x: i32, y: i32, mode: i32, double_buffer: i32) -> i32 {
    let mut s = sgl();
    s.device = Some((x.clamp(1, 4096), y.clamp(1, 4096)));
    let h = s.handle();
    shim::log!("sgl_create_screen_device {device_number} {x}x{y} mode {mode} double {double_buffer} -> {h}");
    h
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_get_device(name: i32, device_number: *mut i32, x: *mut i32, y: *mut i32, mode: *mut i32, double_buffer: *mut i32) -> i32 {
    let (w, h) = sgl().device.unwrap_or((640, 480));
    trace!("sgl_get_device", "{name} -> {w}x{h}");
    unsafe {
        for (p, v) in [(device_number, 0), (x, w), (y, h), (mode, 0), (double_buffer, 1)] {
            if !p.is_null() {
                p.write_unaligned(v);
            }
        }
    }
    name
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_delete_device(name: i32) {
    trace!("sgl_delete_device", "{name}");
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_viewport(parent_device: i32, left: i32, top: i32, right: i32, bottom: i32, cl: f32, ct: f32, cr: f32, cb: f32) -> i32 {
    let mut s = sgl();
    let h = s.handle();
    s.viewports.insert(h, Viewport { rect: [left, top, right, bottom], cam: [cl, ct, cr, cb], subtract: Vec::new() });
    shim::log!("sgl_create_viewport {parent_device} ({left},{top})-({right},{bottom}) camera ({cl},{ct})-({cr},{cb}) -> {h}");
    h
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_set_viewport(viewport: i32, left: i32, top: i32, right: i32, bottom: i32, cl: f32, ct: f32, cr: f32, cb: f32) -> i32 {
    trace!("sgl_set_viewport", "{viewport} ({left},{top})-({right},{bottom}) camera ({cl},{ct})-({cr},{cb})");
    match sgl().viewports.get_mut(&viewport) {
        Some(v) => {
            v.rect = [left, top, right, bottom];
            v.cam = [cl, ct, cr, cb];
            0
        }
        None => crate::scene::ERR_BAD_NAME,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_subtract_viewport(viewport: i32, removed_viewport: i32) {
    trace!("sgl_subtract_viewport", "{viewport} {removed_viewport}");
    if let Some(v) = sgl().viewports.get_mut(&viewport) {
        if !v.subtract.contains(&removed_viewport) {
            v.subtract.push(removed_viewport);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_delete_viewport(viewport: i32) {
    trace!("sgl_delete_viewport", "{viewport}");
    let mut s = sgl();
    s.viewports.remove(&viewport);
    for v in s.viewports.values_mut() {
        v.subtract.retain(|&r| r != viewport);
    }
}

/// `sglwin32.h`: SGL calls `next_address` for the memory to render each frame into; the word
/// it points `status` at says whether a render is busy (never, here).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_use_address_mode(next_address: usize, status: *mut *const u32) -> i32 {
    shim::log!("sgl_use_address_mode callback {next_address:#x} status {status:?}");
    crate::frame::set_callback(next_address);
    if !status.is_null() {
        unsafe { status.write_unaligned(crate::frame::status()) };
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_render(viewport_or_device: i32, camera_or_list: i32, swap_buffers: i32) {
    trace!("sgl_render", "{viewport_or_device} {camera_or_list} {swap_buffers}");
    crate::frame::render(viewport_or_device, camera_or_list, swap_buffers != 0);
}
