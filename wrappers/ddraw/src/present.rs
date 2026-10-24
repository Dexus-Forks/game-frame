//! Showing the frame: the primary surface's pixels drawn into the game's window, scaled. This
//! is the frame scaler of the wrapper's first layer. The window is `DDRAW_SCALE` times the
//! mode (default 2); `DDRAW_FILTER` picks how the picture fills it:
//!
//! - `nearest` (the default): the largest whole multiple that fits, each pixel a sharp block;
//! - `scale2x`: the same size, with Scale2x (EPX) smoothing the stair steps of edges;
//! - `crt`: the same size, with scanlines and a column mask;
//! - `smooth`: any size, GDI's halftone stretch (soft).
//!
//! The first three are the `scalers` crate, done on the CPU and drawn 1:1, centred.

use crate::surface::Surface;
use shim::log;
use std::ffi::c_void;
use std::sync::Mutex;

type Hwnd = *mut c_void;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetDC(w: Hwnd) -> *mut c_void;
    fn ReleaseDC(w: Hwnd, dc: *mut c_void) -> i32;
    fn GetClientRect(w: Hwnd, r: *mut [i32; 4]) -> i32;
    fn SetWindowLongA(w: Hwnd, index: i32, v: i32) -> i32;
    fn SetWindowPos(w: Hwnd, after: Hwnd, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn PatBlt(dc: *mut c_void, x: i32, y: i32, w: i32, h: i32, rop: u32) -> i32;
    fn StretchDIBits(dc: *mut c_void, x: i32, y: i32, w: i32, h: i32, sx: i32, sy: i32, sw: i32, sh: i32, bits: *const c_void, info: *const c_void, usage: u32, rop: u32) -> i32;
    fn SetStretchBltMode(dc: *mut c_void, mode: i32) -> i32;
}

struct Display {
    hwnd: usize,
    mode: (u32, u32, u32),
    primary: usize,
    palette_dirty: bool,
}

static DISPLAY: Mutex<Display> = Mutex::new(Display { hwnd: 0, mode: (640, 480, 16), primary: 0, palette_dirty: true });

fn display() -> std::sync::MutexGuard<'static, Display> {
    DISPLAY.lock().unwrap_or_else(|e| e.into_inner())
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.is_empty())
}

pub fn scale() -> f32 {
    env("DDRAW_SCALE").and_then(|v| v.parse().ok()).filter(|s: &f32| *s > 0.0).unwrap_or(2.0)
}

pub fn mode() -> (u32, u32, u32) {
    display().mode
}

pub fn set_window(hwnd: *mut c_void) {
    display().hwnd = hwnd as usize;
}

pub fn set_primary(s: *mut Surface) {
    display().primary = s as usize;
}

pub fn palette_changed() {
    display().palette_dirty = true;
}

/// The mode the game asked for; the window becomes a borderless one of the output size.
pub fn set_mode(w: u32, h: u32, bpp: u32) {
    let hwnd = {
        let mut d = display();
        d.mode = (w, h, bpp);
        d.hwnd as Hwnd
    };
    if !hwnd.is_null() {
        let s = scale();
        let (ow, oh) = ((w as f32 * s) as i32, (h as f32 * s) as i32);
        unsafe {
            SetWindowLongA(hwnd, -16, 0x9000_0000u32 as i32); // WS_POPUP | WS_VISIBLE
            SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, ow, oh, 0x4 | 0x20); // NOZORDER, FRAMECHANGED
        }
        log!("window {hwnd:?}: {w}x{h}x{bpp} shown at {ow}x{oh}");
    }
}

/// The front buffer as `0x00RRGGBB` pixels.
fn rgb32(front: &Surface, pal: Option<[[u8; 4]; 256]>) -> Vec<u32> {
    let [rm, gm, bm] = crate::desc::masks16();
    let (w, h) = (front.w as usize, front.h as usize);
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        let row = &front.mem[y * front.pitch as usize..];
        for x in 0..w {
            out.push(if front.bpp == 8 {
                let [r, g, b, _] = pal.map(|p| p[row[x] as usize]).unwrap_or([row[x]; 4]);
                (r as u32) << 16 | (g as u32) << 8 | b as u32
            } else {
                let v = u16::from_le_bytes([row[2 * x], row[2 * x + 1]]) as u32;
                // Widen each field to 8 bits by repeating its top bits.
                let f = |m: u32| {
                    let (shift, bits) = (m.trailing_zeros(), m.count_ones());
                    let c = (v & m) >> shift;
                    (c << (8 - bits)) | (c >> (2 * bits - 8).min(bits))
                };
                f(rm) << 16 | f(gm) << 8 | f(bm)
            });
        }
    }
    out
}

/// `sgl_wrapper_present` of the PowerVR build's `sgl.dll` (`wrappers/sgl`), if loaded: it
/// puts the game's 2D picture over its own 3D frame and shows both.
type SglPresent = unsafe extern "C" fn(usize, *const u8, u32, u32, u32, u32) -> i32;

fn sgl_present() -> Option<SglPresent> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(m: *mut c_void, name: *const u8) -> *const c_void;
    }
    static F: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let f = *F.get_or_init(|| unsafe {
        let m = GetModuleHandleA(c"sgl.dll".as_ptr().cast());
        if m.is_null() { 0 } else { GetProcAddress(m, c"sgl_wrapper_present".as_ptr().cast()) as usize }
    });
    (f != 0).then(|| unsafe { std::mem::transmute::<usize, SglPresent>(f) })
}

/// Draw `front` into the window, filling its client area with the picture kept in shape.
pub fn show(front: &Surface) {
    if let Some(f) = sgl_present() {
        let hwnd = display().hwnd;
        if hwnd != 0 && unsafe { f(hwnd, front.mem.as_ptr(), front.w, front.h, front.pitch, front.bpp) } != 0 {
            return;
        }
    }
    let (hwnd, pal) = {
        let d = display();
        let pal = if front.bpp == 8 { palette(d.primary as *const Surface, front) } else { None };
        (d.hwnd as Hwnd, pal)
    };
    if hwnd.is_null() {
        return;
    }
    let filter = env("DDRAW_FILTER").unwrap_or_else(|| "nearest".into()).to_ascii_lowercase();
    unsafe {
        let mut r = [0i32; 4];
        GetClientRect(hwnd, &mut r);
        let (cw, ch) = (r[2] - r[0], r[3] - r[1]);
        let dc = GetDC(hwnd);
        let (w, h) = (front.w as i32, front.h as i32);
        let k = (cw / w.max(1)).min(ch / h.max(1)).max(1);
        if let (Some(scale), false) = (scalers::by_name(&filter), cw < w || ch < h) {
            // A whole multiple, on the CPU, drawn pixel for pixel.
            let img = scale(&rgb32(front, pal), w as usize, h as usize, k as usize);
            let (vw, vh) = (w * k, h * k);
            let mut info = [0u32; 10];
            info[0] = 40;
            info[1] = vw as u32;
            info[2] = (-vh) as u32;
            info[3] = 1 | 32 << 16;
            if (vw, vh) != (cw, ch) {
                PatBlt(dc, 0, 0, cw, ch, 0x0000_0042); // BLACKNESS
            }
            StretchDIBits(dc, (cw - vw) / 2, (ch - vh) / 2, vw, vh, 0, 0, vw, vh, img.as_ptr().cast(), info.as_ptr().cast(), 0, 0x00cc_0020);
        } else {
            // Any size, by GDI: halftone (`smooth`) or plain stretching.
            let img = rgb32(front, pal);
            let mut info = [0u32; 10];
            info[0] = 40;
            info[1] = w as u32;
            info[2] = (-h) as u32;
            info[3] = 1 | 32 << 16;
            let f = (cw as f32 / w as f32).min(ch as f32 / h as f32);
            let (vw, vh) = ((w as f32 * f) as i32, (h as f32 * f) as i32);
            SetStretchBltMode(dc, if filter == "smooth" { 4 } else { 3 }); // HALFTONE or COLORONCOLOR
            StretchDIBits(dc, (cw - vw) / 2, (ch - vh) / 2, vw, vh, 0, 0, w, h, img.as_ptr().cast(), info.as_ptr().cast(), 0, 0x00cc_0020);
        }
        ReleaseDC(hwnd, dc);
    }
}

/// The palette of the primary surface (or of `front` itself).
fn palette(primary: *const Surface, front: &Surface) -> Option<[[u8; 4]; 256]> {
    let p = if !front.palette.is_null() {
        front.palette
    } else if !primary.is_null() {
        unsafe { (*primary).palette }
    } else {
        std::ptr::null_mut()
    };
    (!p.is_null()).then(|| unsafe { (*p).entries })
}
