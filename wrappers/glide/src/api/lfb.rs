//! The linear frame buffer (the early Glide 2 calls: `grLfbBegin`, `grLfbGetWritePtr`, …).
//!
//! The game writes pixels through a pointer with a fixed stride of 1024 pixels (2048 bytes in
//! the 16-bit modes, 4096 in the 32-bit ones; `FUN_00492a9c`), and it may pick the write mode
//! after taking the pointer (`0x4a98f3`). So the pointer is a memory buffer filled with a
//! marker word; when the game is done (`grLfbEnd`, or the next draw or swap) every word that
//! is no longer the marker is a written pixel, read in the mode then current, and sent to the
//! render thread as an image. Reads of the depth or front buffer come back from the render
//! thread at the game's resolution (`lfbread`).
//!
//! Pixels are read in the write colour format (`grLfbWriteColorFormat`, by default the one the
//! game opened with) after the byte and word swaps of `grLfbWriteColorSwizzle`. With the lower
//! left origin, row 0 is the bottom one.

use crate::api::sst::screen;
use crate::cmd::{Cmd, LfbImage, push};
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

pub(super) struct Lfb {
    mode: u32,
    bypass: bool,
    constant_alpha: u8,
    /// `grLfbConstantDepth`. Deviation: kept but not used, as LFB writes are not depth-tested.
    constant_depth: u16,
    /// `GrColorFormat_t` of written pixels.
    format: u32,
    /// `grLfbWriteColorSwizzle`: bytes reversed in each 32-bit word, 16-bit halves swapped.
    swizzle: (bool, bool),
    /// Row 0 is the bottom one.
    pub(super) lower_left: bool,
    write: Vec<u32>,
    pub(super) depth: Vec<u16>,
    pub(super) front: Vec<u16>,
}

static LFB: Mutex<Option<Lfb>> = Mutex::new(None);
static OUTSTANDING: AtomicBool = AtomicBool::new(false);

pub(super) fn with<R>(f: impl FnOnce(&mut Lfb) -> R) -> R {
    let mut g = LFB.lock().unwrap_or_else(|e| e.into_inner());
    let l = g.get_or_insert_with(|| Lfb {
        mode: LFB_565,
        bypass: false,
        constant_alpha: 0xff,
        constant_depth: 0xffff,
        format: COLORFORMAT_ARGB,
        swizzle: (false, false),
        lower_left: false,
        write: vec![mark32(); STRIDE * 1024],
        depth: vec![0xffff; STRIDE * 1024],
        front: vec![0; STRIDE * 1024],
    });
    f(l)
}

/// At each open: the colour format and origin the game opened with.
pub fn opened(format: u32, origin: u32) {
    with(|l| {
        l.format = format;
        l.lower_left = origin == ORIGIN_LOWER_LEFT;
    });
}

/// A written word after `grLfbWriteColorSwizzle`.
fn swizzle(w: u32, (bytes, words): (bool, bool)) -> u32 {
    let w = if bytes { w.swap_bytes() } else { w };
    if words { w.rotate_left(16) } else { w }
}

/// One pixel in `mode` and colour `format` to RGBA; alpha is the pixel's own, or `constant`
/// if it has none. Deviation: in the 16-bit modes the formats only choose whether red or blue
/// is in the top bits; alpha stays where `GR_COLORFORMAT_ARGB` has it.
fn rgba(mode: u32, format: u32, v: u32, constant: u8) -> [u8; 4] {
    if wide(mode) {
        return rgba_argb(mode, argb(v, format), constant);
    }
    let mut c = rgba_argb(mode, v, constant);
    if matches!(format, COLORFORMAT_ABGR | COLORFORMAT_BGRA) {
        c.swap(0, 2);
    }
    c
}

fn rgba_argb(mode: u32, v: u32, constant: u8) -> [u8; 4] {
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
                    let mut c = rgba(l.mode, l.format, v, alpha);
                    c[3] = if l.bypass { 255 } else { c[3].max(1) };
                    rgba_out.extend_from_slice(&c);
                }
                None => rgba_out.extend_from_slice(&[0; 4]),
            }
        }
    }
    Some(LfbImage { x: x0 as u32, y: y0 as u32, w: (x1 - x0) as u32, h: (y1 - y0) as u32, rgba: rgba_out, pipeline: !l.bypass })
}

/// An image whose rows count from the bottom, turned to count from the top.
fn flip(mut img: LfbImage, lower_left: bool) -> LfbImage {
    if lower_left {
        let row = img.w as usize * 4;
        let rows: Vec<&[u8]> = img.rgba.chunks(row).rev().collect();
        img.rgba = rows.concat();
        img.y = screen().1.saturating_sub(img.y + img.h);
    }
    img
}

/// Send what the game wrote through the last write pointer.
pub fn finish_write() {
    if !OUTSTANDING.swap(false, Ordering::AcqRel) {
        return;
    }
    let (w, h) = screen();
    let img = with(|l| {
        let (mark16, mark32) = (mark16(), mark32());
        let img = if l.swizzle != (false, false) {
            // The marker as it reads after the same swaps, so unwritten pixels stay unwritten.
            let (sw, buf) = (l.swizzle, &l.write);
            let m = swizzle(mark32, sw);
            if wide(l.mode) {
                image(l, w, h, |x, y| Some(swizzle(buf[y * STRIDE + x], sw)).filter(|v| v & 0xff_ffff != m & 0xff_ffff))
            } else {
                image(l, w, h, |x, y| {
                    let v = swizzle(buf[(y * STRIDE + x) / 2], sw) >> (16 * (x & 1)) & 0xffff;
                    Some(v).filter(|v| *v != m & 0xffff)
                })
            }
        } else if wide(l.mode) {
            let buf = &l.write;
            image(l, w, h, |x, y| Some(buf[y * STRIDE + x]).filter(|v| v & 0xff_ffff != mark32 & 0xff_ffff))
        } else {
            let buf: &[u16] = unsafe { std::slice::from_raw_parts(l.write.as_ptr().cast(), STRIDE * 2 * 1024) };
            image(l, w, h, |x, y| Some(buf[y * STRIDE + x] as u32).filter(|v| *v != mark16 as u32))
        };
        l.write[..STRIDE * h as usize].fill(mark32);
        img.map(|i| flip(i, l.lower_left))
    });
    if let Some(img) = img {
        // The pixels the CPU wrote while the frame buffer was locked: not a Glide call, so it has
        // its own name, `lfb_pixels(x, y, w, h, [bytes hash])`. It is written when the library
        // takes the pixels: at the next call that draws, or at the swap.
        if shim::calls::on() {
            let mut args = format!("{:#x}, {:#x}, {:#x}, {:#x}, ", img.x, img.y, img.w, img.h);
            shim::calls::Arg::put(&shim::calls::Bytes(&img.rgba), &mut args);
            shim::calls::write("lfb_pixels", &args);
        }
        push(Cmd::Lfb(img));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbBegin() {
    shim::call!("grLfbBegin");
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbEnd() {
    shim::call!("grLfbEnd");
    finish_write();
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbGetWritePtr(buffer: u32) -> *mut u32 {
    shim::call!("grLfbGetWritePtr", buffer);
    finish_write();
    if buffer != BUFFER_BACK {
        shim::gap!("grLfbGetWritePtr buffer", "buffer {buffer}: only the back buffer is handled");
    }
    OUTSTANDING.store(true, Ordering::Release);
    with(|l| l.write.as_mut_ptr())
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbWriteMode(mode: u32) {
    shim::call!("grLfbWriteMode", mode);
    with(|l| l.mode = mode);
    if !matches!(mode, LFB_565 | LFB_555 | LFB_1555 | LFB_888 | LFB_8888) {
        shim::gap!("grLfbWriteMode", "mode {mode} is not handled");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbBypassMode(mode: u32) {
    shim::call!("grLfbBypassMode", mode);
    with(|l| l.bypass = mode == LFBBYPASS_ENABLE);
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbConstantAlpha(alpha: u32) {
    shim::call!("grLfbConstantAlpha", alpha);
    with(|l| l.constant_alpha = alpha as u8);
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbConstantDepth(depth: u32) {
    shim::call!("grLfbConstantDepth", depth);
    with(|l| l.constant_depth = depth as u16);
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbWriteColorFormat(format: u32) {
    shim::call!("grLfbWriteColorFormat", format);
    finish_write();
    with(|l| l.format = format & 3);
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbWriteColorSwizzle(bytes: u32, words: u32) {
    shim::call!("grLfbWriteColorSwizzle", bytes, words);
    finish_write();
    with(|l| l.swizzle = (bytes != 0, words != 0));
}

/// The early Glide 2 call that says where row 0 of the frame buffer is.
#[unsafe(no_mangle)]
pub extern "system" fn grLfbOrigin(origin: u32) {
    shim::call!("grLfbOrigin", origin);
    finish_write();
    with(|l| l.lower_left = origin == ORIGIN_LOWER_LEFT);
}

/// `guFbWriteRegion`: a `w × h` rectangle of pixels in the current write mode, `stride` bytes
/// per row, to `(x, y)` of the back buffer (POD: the loading picture in strips of 640 × 30).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFbWriteRegion(x: u32, y: u32, w: u32, h: u32, src: *const u8, stride: i32) {
    shim::call!("guFbWriteRegion", x, y, w, h, crate::callargs::Region(src, w, h, stride, with(|l| if wide(l.mode) { 4 } else { 2 })), stride);
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
        let lower_left = with(|l| l.lower_left);
        push(Cmd::Lfb(flip(img, lower_left)));
    }
}

// The later Glide 2.x LFB calls (POD Gold's 3dfx build): a lock that names the buffer, the
// write mode and the pixel pipeline at once, and fills a `GrLfbInfo_t` (size, pointer,
// stride in bytes, write mode, origin).

const LFB_READ_ONLY: u32 = 0;
const LFBWRITEMODE_ANY: u32 = 0xff;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grLfbLock(kind: u32, buffer: u32, mode: u32, origin: u32, pipeline: u32, info: *mut u32) -> u32 {
    shim::call!("grLfbLock", kind, buffer, mode, origin, pipeline, info);
    if info.is_null() {
        return 0;
    }
    finish_write();
    with(|l| l.lower_left = origin == ORIGIN_LOWER_LEFT);
    let (ptr, stride, mode) = if kind & 1 == LFB_READ_ONLY {
        (super::lfbread::grLfbGetReadPtr(buffer) as usize, 2048, LFB_565)
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
        info.add(4).write_unaligned(origin);
    }
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbUnlock(kind: u32, _buffer: u32) -> u32 {
    shim::call!("grLfbUnlock", kind, _buffer);
    if kind & 1 != LFB_READ_ONLY {
        finish_write();
    }
    1
}

/// A rectangle of pixels in `format` (the write-mode numbers), `stride` bytes per row, to
/// `(x, y)` of the buffer, bypassing the pixel pipeline.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grLfbWriteRegion(_buffer: u32, x: u32, y: u32, format: u32, w: u32, h: u32, stride: i32, data: *const u8) -> u32 {
    shim::call!("grLfbWriteRegion", _buffer, x, y, format, w, h, stride, crate::callargs::Region(data, w, h, stride, if wide(format) { 4 } else { 2 }));
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
