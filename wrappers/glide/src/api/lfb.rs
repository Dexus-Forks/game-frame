//! The linear frame buffer (the early Glide 2 calls: `grLfbBegin`, `grLfbGetWritePtr`, …).
//!
//! The game writes pixels through a pointer with a fixed stride of 1024 pixels (2048 bytes in
//! the 16-bit modes, 4096 in the 32-bit ones; `FUN_00492a9c`), and it may pick the write mode
//! after taking the pointer (`0x4a98f3`). So the pointer is a memory buffer filled with a
//! marker word; when the game is done (`grLfbEnd`, or the next draw or swap) every word that
//! is no longer the marker is a written pixel, read in the mode then current, and sent to the
//! render thread as an image. Reads of the depth or front buffer come back from the render
//! thread at the game's resolution.

use crate::api::sst::screen;
use crate::cmd::{Cmd, LfbImage, push, read};
use crate::consts::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const STRIDE: usize = 1024;
/// Deviation: the marker. A pixel the game writes with exactly this value
/// (`WRAPPER_MARKER`, by default RGB565 0x0821, almost black) is taken as not written; the
/// Voodoo has no such case.
fn mark16() -> u16 {
    shim::config::marker()
}

/// The marker twice in a word: two 16-bit pixels, or one 32-bit pixel's low 24 bits.
fn mark32() -> u32 {
    (mark16() as u32) << 16 | mark16() as u32
}

struct Lfb {
    mode: u32,
    bypass: bool,
    constant_alpha: u8,
    write: Vec<u32>,
    depth: Vec<u16>,
    front: Vec<u16>,
}

static LFB: Mutex<Option<Lfb>> = Mutex::new(None);
static OUTSTANDING: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut Lfb) -> R) -> R {
    let mut g = LFB.lock().unwrap_or_else(|e| e.into_inner());
    let l = g.get_or_insert_with(|| Lfb {
        mode: LFB_565,
        bypass: false,
        constant_alpha: 0xff,
        write: vec![mark32(); STRIDE * 1024],
        depth: vec![0xffff; STRIDE * 1024],
        front: vec![0; STRIDE * 1024],
    });
    f(l)
}

/// One pixel in `mode` to RGBA; alpha is the pixel's own, or `constant` if it has none.
fn rgba(mode: u32, v: u32, constant: u8) -> [u8; 4] {
    match mode {
        LFB_565 => {
            let [r, g, b] = pixels::rgb565(v as u16);
            [r, g, b, constant]
        }
        LFB_555 => {
            let [r, g, b] = pixels::rgb555(v as u16 & 0x7fff);
            [r, g, b, constant]
        }
        LFB_1555 => {
            let [r, g, b] = pixels::rgb555(v as u16 & 0x7fff);
            [r, g, b, if v & 0x8000 != 0 { 255 } else { 0 }]
        }
        LFB_8888 => [(v >> 16) as u8, (v >> 8) as u8, v as u8, (v >> 24) as u8],
        _ => [(v >> 16) as u8, (v >> 8) as u8, v as u8, constant],
    }
}

fn wide(mode: u32) -> bool {
    matches!(mode, LFB_888 | LFB_8888)
}

/// An image of the rectangle `x0..x1, y0..y1` from pixel words `px(x, y)` (None: not
/// written), or nothing if no pixel was written.
fn image(l: &Lfb, w: u32, h: u32, px: impl Fn(usize, usize) -> Option<u32>) -> Option<LfbImage> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..h as usize {
        for x in 0..w as usize {
            if px(x, y).is_some() {
                x0 = x0.min(x);
                x1 = x1.max(x + 1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x0 == usize::MAX {
        return None;
    }
    let alpha = if l.bypass { 0xff } else { l.constant_alpha };
    let mut rgba_out = Vec::with_capacity((x1 - x0) * (y1 - y0) * 4);
    for y in y0..y1 {
        for x in x0..x1 {
            match px(x, y) {
                // Written pixels keep at least alpha 1: 0 marks "not written".
                Some(v) => {
                    let mut c = rgba(l.mode, v, alpha);
                    c[3] = if l.bypass { 255 } else { c[3].max(1) };
                    rgba_out.extend_from_slice(&c);
                }
                None => rgba_out.extend_from_slice(&[0; 4]),
            }
        }
    }
    Some(LfbImage { x: x0 as u32, y: y0 as u32, w: (x1 - x0) as u32, h: (y1 - y0) as u32, rgba: rgba_out, pipeline: !l.bypass })
}

/// Send what the game wrote through the last write pointer.
pub fn finish_write() {
    if !OUTSTANDING.swap(false, Ordering::AcqRel) {
        return;
    }
    let (w, h) = screen();
    let img = with(|l| {
        let (mark16, mark32) = (mark16(), mark32());
        let img = if wide(l.mode) {
            let buf = &l.write;
            image(l, w, h, |x, y| Some(buf[y * STRIDE + x]).filter(|v| v & 0xff_ffff != mark32 & 0xff_ffff))
        } else {
            let buf: &[u16] = unsafe { std::slice::from_raw_parts(l.write.as_ptr().cast(), STRIDE * 2 * 1024) };
            image(l, w, h, |x, y| Some(buf[y * STRIDE + x] as u32).filter(|v| *v != mark16 as u32))
        };
        l.write[..STRIDE * h as usize].fill(mark32);
        img
    });
    if let Some(img) = img {
        push(Cmd::Lfb(img));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbBegin() {}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbEnd() {
    finish_write();
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbGetWritePtr(buffer: u32) -> *mut u32 {
    finish_write();
    if buffer != BUFFER_BACK {
        shim::gap!("grLfbGetWritePtr buffer", "buffer {buffer}: only the back buffer is handled");
    }
    OUTSTANDING.store(true, Ordering::Release);
    with(|l| l.write.as_mut_ptr())
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbGetReadPtr(buffer: u32) -> *const u16 {
    finish_write();
    let data = read(buffer);
    with(|l| {
        let dst = if buffer == BUFFER_DEPTH || buffer == BUFFER_AUX { &mut l.depth } else { &mut l.front };
        let n = data.len().min(dst.len());
        dst[..n].copy_from_slice(&data[..n]);
        dst.as_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbWriteMode(mode: u32) {
    with(|l| l.mode = mode);
    if !matches!(mode, LFB_565 | LFB_555 | LFB_1555 | LFB_888 | LFB_8888) {
        shim::gap!("grLfbWriteMode", "mode {mode} is not handled");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbBypassMode(mode: u32) {
    with(|l| l.bypass = mode == LFBBYPASS_ENABLE);
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbConstantAlpha(alpha: u32) {
    with(|l| l.constant_alpha = alpha as u8);
}

/// `guFbWriteRegion`: a `w × h` rectangle of pixels in the current write mode, `stride` bytes
/// per row, to `(x, y)` of the back buffer (POD: the loading picture in strips of 640 × 30).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFbWriteRegion(x: u32, y: u32, w: u32, h: u32, src: *const u8, stride: i32) {
    finish_write();
    if src.is_null() || w == 0 || h == 0 {
        return;
    }
    let img = with(|l| {
        let bpp = if wide(l.mode) { 4 } else { 2 };
        let px = |px: usize, py: usize| -> Option<u32> {
            let p = unsafe { src.offset(py as isize * stride as isize + (px * bpp) as isize) };
            Some(unsafe { if bpp == 4 { (p as *const u32).read_unaligned() } else { (p as *const u16).read_unaligned() as u32 } })
        };
        image(l, w, h, px)
    });
    if let Some(mut img) = img {
        img.x += x;
        img.y += y;
        push(Cmd::Lfb(img));
    }
}

// The later Glide 2.x LFB calls (POD Gold's 3dfx build): a lock that names the buffer, the
// write mode and the pixel pipeline at once, and fills a `GrLfbInfo_t` (size, pointer,
// stride in bytes, write mode, origin).

const LFB_READ_ONLY: u32 = 0;
const LFBWRITEMODE_ANY: u32 = 0xff;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grLfbLock(kind: u32, buffer: u32, mode: u32, _origin: u32, pipeline: u32, info: *mut u32) -> u32 {
    if info.is_null() {
        return 0;
    }
    let (ptr, stride, mode) = if kind & 1 == LFB_READ_ONLY {
        (grLfbGetReadPtr(buffer) as usize, 2048, LFB_565)
    } else {
        if mode != LFBWRITEMODE_ANY {
            grLfbWriteMode(mode);
        }
        grLfbBypassMode(if pipeline != 0 { 0 } else { LFBBYPASS_ENABLE });
        let m = with(|l| l.mode);
        (grLfbGetWritePtr(buffer) as usize, if wide(m) { 4096 } else { 2048 }, m)
    };
    unsafe {
        info.add(1).write_unaligned(ptr as u32);
        info.add(2).write_unaligned(stride);
        info.add(3).write_unaligned(mode);
        info.add(4).write_unaligned(0);
    }
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbUnlock(kind: u32, _buffer: u32) -> u32 {
    if kind & 1 != LFB_READ_ONLY {
        finish_write();
    }
    1
}

/// A rectangle of pixels in `format` (the write-mode numbers), `stride` bytes per row, to
/// `(x, y)` of the buffer, bypassing the pixel pipeline.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grLfbWriteRegion(_buffer: u32, x: u32, y: u32, format: u32, w: u32, h: u32, stride: i32, data: *const u8) -> u32 {
    let (mode, bypass) = with(|l| (l.mode, l.bypass));
    with(|l| {
        l.mode = format;
        l.bypass = true;
    });
    unsafe { guFbWriteRegion(x, y, w, h, data, stride) };
    with(|l| {
        l.mode = mode;
        l.bypass = bypass;
    });
    1
}
