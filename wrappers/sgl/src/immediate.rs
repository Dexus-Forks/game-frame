//! `sgltri_render`'s frame. This module sends an immediate-mode frame to the window in each of
//! SGL's two output modes, as PowerVR's driver orders it (SGLMID4.dll `FUN_10004dc0`, called
//! at the start of each render; *observed*).
//!
//! - Address mode (`sgl_use_address_mode`). The game's callback finishes the previous frame
//!   and names the memory for the next, as for the scene graph (`frame`). The device's pixels
//!   in that memory are filled with the marker. The game's 2D goes on top through
//!   `wrappers/ddraw`.
//! - DirectDraw mode (`sgl_use_ddraw_mode`). SGL owns the screen. At each render it first
//!   calls the game's 2D callback with the back buffer that holds the frame the chip has just
//!   drawn (the one before this render), before the flip. The surface is not locked during
//!   the call: SGL locked it after the last flip only to read its address and pitch, then
//!   unlocked it (with sglhw.ini's `StrictLocks` 0). Then SGL flips, and the chip draws the
//!   new frame into the next back buffer. The order here is the same. The surface is a 16-bit
//!   picture of the device's size, kept by this library. The callback draws into it. The
//!   picture goes over the previous frame and is shown (the flip). Then the library fills it
//!   with the marker again, and the new frame goes to the render thread.
//!
//! Then the frame goes to the render thread. The end-of-render callback
//! (`sgl_use_eor_callback`) runs after that. If it returns a value other than 0, the next
//! `sgltri_render` reports that value as its error (`FUN_100479a0`).

use crate::render::{self, Cmd, Present, ViewDraw};
use crate::scene::walk::View;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

static PROC_2D: AtomicUsize = AtomicUsize::new(0);
static HWND: AtomicUsize = AtomicUsize::new(0);
static EOR: AtomicUsize = AtomicUsize::new(0);
/// What the end-of-render callback returned, for the next render.
static EOR_ERROR: AtomicI32 = AtomicI32::new(0);

/// The DirectDraw mode's surface: RGB565, the device's size.
static SURFACE: Mutex<(Vec<u16>, i32, i32)> = Mutex::new((Vec::new(), 0, 0));

/// `CALLBACK_SURFACE_PARAMS` (sglwin32.h, packed).
#[repr(C, packed)]
struct SurfaceParams {
    ddraw: usize,
    surface: usize,
    mem: usize,
    bpp: u16,
    stride: u32,
    w: u16,
    h: u16,
}

/// `sgl_use_ddraw_mode`; false when it was called before.
pub fn set_ddraw(hwnd: usize, proc_2d: usize) -> bool {
    if HWND.load(Ordering::Relaxed) != 0 {
        return false;
    }
    HWND.store(hwnd.max(1), Ordering::Relaxed);
    PROC_2D.store(proc_2d, Ordering::Relaxed);
    true
}

pub fn ddraw_mode() -> bool {
    HWND.load(Ordering::Relaxed) != 0
}

pub fn set_eor(f: usize) {
    EOR.store(f, Ordering::Relaxed);
}

/// Hand the game's 2D callback the surface, show what it drew over the frame before, and fill
/// the surface with the marker for the next.
fn ddraw_picture(dev: (i32, i32)) {
    let (w, h) = (dev.0.max(1), dev.1.max(1));
    let mut guard = SURFACE.lock().unwrap_or_else(|e| e.into_inner());
    let (buf, sw, sh) = &mut *guard;
    if (*sw, *sh) != (w, h) {
        *buf = vec![shim::config::marker(); (w * h) as usize];
        (*sw, *sh) = (w, h);
    }
    let f = PROC_2D.load(Ordering::Relaxed);
    if f != 0 {
        // Deviation: the game gets no DirectDraw objects (both are null), only the memory; the
        // driver passes its DirectDraw object and the back buffer's surface. The size is the
        // device's, where the driver snaps it to 320 x 240, 640 x 400 or 480, or 800 x 600.
        let mut p = SurfaceParams { ddraw: 0, surface: 0, mem: buf.as_mut_ptr() as usize, bpp: 16, stride: (w * 2) as u32, w: w as u16, h: h as u16 };
        let f: unsafe extern "C" fn(*mut SurfaceParams) -> i32 = unsafe { std::mem::transmute(f) };
        let r = unsafe { f(&mut p) };
        shim::log_once!("2D callback: {w}x{h} surface (returned {r})");
    }
    let rgba = unsafe { crate::frame::picture(buf.as_ptr().cast(), w as u32, h as u32, (w * 2) as u32) };
    buf.fill(shim::config::marker());
    let hwnd = HWND.load(Ordering::Relaxed);
    render::send(Cmd::Present(Present { hwnd: if hwnd > 1 { hwnd } else { 0 }, w: w as u32, h: h as u32, rgba: Some(rgba) }));
}

/// Show an immediate-mode frame. Returns the end-of-render callback's error from the render
/// before.
pub fn render(view: View, dev: (i32, i32)) -> i32 {
    let before = EOR_ERROR.swap(0, Ordering::Relaxed);
    let whole = vec![[0, 0, dev.0 - 1, dev.1 - 1]];
    if ddraw_mode() {
        ddraw_picture(dev);
    } else if let Some(p) = crate::frame::address() {
        crate::frame::fill_marker(&p, &whole);
    }
    render::send(Cmd::Frame(vec![ViewDraw { device: dev, cam: [-1.0, -1.0, 1.0, 1.0], visible: whole, view }]));
    if !ddraw_mode() && !crate::frame::hooked() {
        render::send(Cmd::Present(Present { hwnd: 0, w: 0, h: 0, rgba: None }));
    }
    let f = EOR.load(Ordering::Relaxed);
    if f != 0 {
        let f: unsafe extern "C" fn() -> i32 = unsafe { std::mem::transmute(f) };
        let r = unsafe { f() };
        if r != 0 {
            EOR_ERROR.store(r, Ordering::Relaxed);
        }
    }
    before
}
