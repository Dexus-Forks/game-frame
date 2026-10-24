//! `sgl_render`, and how the 3D frame meets the game's 2D one.
//!
//! The PowerVR chip rendered into memory the game named through its address callback
//! (`sgl_use_address_mode`; POD's is `0x4a82f0`). That callback also finishes the frame
//! before: it draws the dashboard over the last render (`FUN_004a7ff0`), shows it (a `Blt` to
//! the window, or a `Flip`), and locks the back buffer for the next one (`FUN_004a73c0`). So
//! here, at the first `sgl_render` of a frame:
//!
//! 1. the callback runs, which shows the frame before (through `wrappers/ddraw`, below);
//! 2. the viewport's pixels in the memory it returns are filled with a marker colour, and the
//!    scene goes to the render thread, which draws it at the output size;
//! 3. the game then draws its dashboard into that memory; when it shows it, `wrappers/ddraw`
//!    hands the picture to `sgl_wrapper_present`, and the render thread puts every pixel
//!    that is not the marker over the 3D frame.
//!
//! With another DirectDraw (no call to `sgl_wrapper_present`), the 3D frame is shown alone.

use crate::api::{Viewport, sgl};
use crate::render::{self, Cmd, Present, ViewDraw};
use crate::scene::walk;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

/// The marker: `WRAPPER_MARKER`, by default RGB565 0x0821, (8, 4, 8), almost black. A 2D
/// pixel of exactly this value counts as not drawn by the game.
fn marker() -> u16 {
    shim::config::marker()
}

static CALLBACK: AtomicUsize = AtomicUsize::new(0);
/// The word `sgl_use_address_mode` points the game at: 0, never busy.
static STATUS: AtomicU32 = AtomicU32::new(0);
static HOOKED: AtomicBool = AtomicBool::new(false);

struct Pending {
    need_address: bool,
    views: Vec<ViewDraw>,
    frames: u32,
}

static PENDING: Mutex<Pending> = Mutex::new(Pending { need_address: true, views: Vec::new(), frames: 0 });

pub fn set_callback(f: usize) {
    CALLBACK.store(f, Ordering::Relaxed);
}

pub fn status() -> *const u32 {
    STATUS.as_ptr()
}

/// `CALLBACK_ADDRESS_PARAMS` (sglwin32.h, packed): the memory, its stride, bits per pixel.
#[repr(C, packed)]
#[derive(Default)]
struct AddressParams {
    mem: usize,
    stride: u16,
    bpp: u8,
}

/// Ask the game where to draw.
fn address() -> Option<AddressParams> {
    let f = CALLBACK.load(Ordering::Relaxed);
    if f == 0 {
        return None;
    }
    let f: unsafe extern "C" fn(*mut AddressParams) -> i32 = unsafe { std::mem::transmute(f) };
    let mut p = AddressParams::default();
    let r = unsafe { f(&mut p) };
    let (mem, stride, bpp) = (p.mem, p.stride, p.bpp);
    shim::log_once!("address callback: {mem:#x}, stride {stride}, {bpp} bits (returned {r})");
    (mem != 0).then_some(p)
}

/// The viewport's rectangle less the subtracted ones, as rectangles (inclusive corners)
/// clipped to the device.
fn visible(vp: &Viewport, all: &std::collections::HashMap<i32, Viewport>, dev: (i32, i32)) -> Vec<[i32; 4]> {
    let clip = |r: [i32; 4]| [r[0].max(0), r[1].max(0), r[2].min(dev.0 - 1), r[3].min(dev.1 - 1)];
    let mut out = vec![clip(vp.rect)];
    for s in vp.subtract.iter().filter_map(|n| all.get(n)) {
        let c = s.rect;
        out = out
            .into_iter()
            .flat_map(|r| {
                if c[0] > r[2] || c[2] < r[0] || c[1] > r[3] || c[3] < r[1] {
                    return vec![r];
                }
                let mut parts = Vec::new();
                if c[1] > r[1] {
                    parts.push([r[0], r[1], r[2], c[1] - 1]);
                }
                if c[3] < r[3] {
                    parts.push([r[0], c[3] + 1, r[2], r[3]]);
                }
                let (t, b) = (c[1].max(r[1]), c[3].min(r[3]));
                if c[0] > r[0] {
                    parts.push([r[0], t, c[0] - 1, b]);
                }
                if c[2] < r[2] {
                    parts.push([c[2] + 1, t, r[2], b]);
                }
                parts
            })
            .collect();
    }
    out.retain(|r| r[0] <= r[2] && r[1] <= r[3]);
    out
}

fn fill_marker(p: &AddressParams, rects: &[[i32; 4]]) {
    if p.bpp != 16 {
        shim::log_once!("{} bits per pixel: no marker drawn", { p.bpp });
        return;
    }
    let (mem, stride, mark) = (p.mem, p.stride as usize, marker());
    for r in rects {
        for y in r[1]..=r[3] {
            let row = (mem + y as usize * stride) as *mut u16;
            for x in r[0]..=r[2] {
                unsafe { row.add(x as usize).write_unaligned(mark) };
            }
        }
    }
}

pub fn render(viewport: i32, camera: i32, swap: bool) {
    let (draw, rects) = {
        let s = sgl();
        let dev = s.device.unwrap_or((640, 480));
        let Some(vp) = s.viewports.get(&viewport) else {
            shim::log_once!("sgl_render: viewport {viewport} unknown");
            return;
        };
        let rects = visible(vp, &s.viewports, dev);
        let view = walk::view(&s.scene, camera as u32);
        if view.is_none() {
            shim::log_once!("sgl_render: camera {camera} not in the scene");
        }
        (view.map(|v| ViewDraw { device: dev, cam: vp.cam, visible: rects.clone(), view: v }), rects)
    };
    let need = PENDING.lock().unwrap_or_else(|e| e.into_inner()).need_address;
    if need {
        // The game's callback runs its own drawing and DirectDraw calls: no lock is held.
        if let Some(p) = address() {
            fill_marker(&p, &rects);
        }
    }
    let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    pending.need_address = false;
    if let Some(d) = draw {
        pending.views.push(d);
    }
    if swap {
        let views = std::mem::take(&mut pending.views);
        pending.need_address = true;
        pending.frames += 1;
        let first = pending.frames == 1;
        drop(pending);
        if first {
            shim::log!("first frame: {} view(s), {} batches", views.len(), views.iter().map(|v| v.view.batches.len()).sum::<usize>());
        }
        render::send(Cmd::Frame(views));
        if !HOOKED.load(Ordering::Relaxed) {
            render::send(Cmd::Present(Present { hwnd: 0, w: 0, h: 0, rgba: None }));
        }
    }
}

/// Called by `wrappers/ddraw` each time the game shows its picture: the window and the
/// primary surface's pixels. Returns 1 when this library showed it (16-bit pictures), 0 to
/// leave it to DirectDraw.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_wrapper_present(hwnd: usize, pixels: *const u8, w: u32, h: u32, pitch: u32, bpp: u32) -> i32 {
    if bpp != 16 || pixels.is_null() || w == 0 || h == 0 || std::env::var("SGL_PRESENT").is_ok_and(|v| v == "0") {
        return 0;
    }
    if !HOOKED.swap(true, Ordering::Relaxed) {
        shim::log!("presenting through wrappers/ddraw: {w}x{h}");
    }
    let mark = marker();
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        let row = unsafe { pixels.add(y * pitch as usize) } as *const u16;
        for x in 0..w as usize {
            let v = unsafe { row.add(x).read_unaligned() };
            if v == mark {
                continue;
            }
            let (r, g, b) = ((v >> 11) & 31, (v >> 5) & 63, v & 31);
            let o = (y * w as usize + x) * 4;
            rgba[o] = ((r << 3) | (r >> 2)) as u8;
            rgba[o + 1] = ((g << 2) | (g >> 4)) as u8;
            rgba[o + 2] = ((b << 3) | (b >> 2)) as u8;
            rgba[o + 3] = 255;
        }
    }
    render::send(Cmd::Present(Present { hwnd, w, h, rgba: Some(rgba) }));
    1
}
