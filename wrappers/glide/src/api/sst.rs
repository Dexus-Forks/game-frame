//! Library and board calls: start-up, the hardware query, opening a resolution, status,
//! buffers, clipping and gamma.

use crate::cmd::{Cmd, flush, push};
use crate::consts;
use crate::log::log;
use std::sync::atomic::{AtomicU32, Ordering};

static WIDTH: AtomicU32 = AtomicU32::new(640);
static HEIGHT: AtomicU32 = AtomicU32::new(480);
/// `GrColorFormat_t` from the open: how the game packs every `GrColor_t`.
static FORMAT: AtomicU32 = AtomicU32::new(consts::COLORFORMAT_ARGB);

/// A colour the game packed, as 0xAARRGGBB.
pub fn argb(c: u32) -> u32 {
    consts::argb(c, FORMAT.load(Ordering::Relaxed))
}

/// What every open does: the size, the colour format, the state reset to Glide's defaults and
/// the origin.
fn opened(w: u32, h: u32, format: u32, origin: u32) {
    WIDTH.store(w, Ordering::Relaxed);
    HEIGHT.store(h, Ordering::Relaxed);
    FORMAT.store(format & 3, Ordering::Relaxed);
    super::shadow::reset(w, h);
    super::shadow::origin(origin & 1);
    super::lfb::opened(format & 3, origin & 1);
}

pub fn screen() -> (u32, u32) {
    (WIDTH.load(Ordering::Relaxed), HEIGHT.load(Ordering::Relaxed))
}

#[unsafe(no_mangle)]
pub extern "system" fn grGlideInit() {
    log!("grGlideInit (output scale {}, size {:?})", crate::config::get().scale, crate::config::get().size);
}

#[unsafe(no_mangle)]
pub extern "system" fn grGlideShutdown() {
    flush();
}

/// Fill a `GrHwConfiguration`: one Voodoo Graphics board, 4 MB frame buffer, one TMU with
/// 4 MB. POD stops unless `num_sst` > 0 and the board type is 0 (`FUN_0049f3a0`).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grSstQueryHardware(hw: *mut u32) -> i32 {
    if hw.is_null() {
        return 0;
    }
    // num_sst, then per board (0x24 bytes): type, fbRam, fbiRev, nTexelfx, sliDetect,
    // tmuConfig[2] = {tmuRev, tmuRam}. Four boards in the SDK's struct.
    let words: [u32; 10] = [1, 0, 4, 2, 1, 0, 1, 4, 0, 0];
    unsafe {
        std::ptr::write_bytes(hw, 0, 1 + 4 * 9);
        for (i, w) in words.iter().enumerate() {
            hw.add(i).write_unaligned(*w);
        }
    }
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstSelect(_board: u32) {}

#[unsafe(no_mangle)]
pub extern "system" fn grSstOpen(res: u32, refresh: u32, format: u32, origin: u32, smoothing: u32, buffers: u32) -> i32 {
    let (w, h) = consts::resolution(res).unwrap_or((640, 480));
    log!("grSstOpen {w}x{h} (res {res}, refresh {refresh}, format {format}, origin {origin}, smoothing {smoothing}, {buffers} buffers)");
    let (ow, oh) = crate::config::get().output(w, h);
    let hwnd = crate::window::prepare(ow, oh) as usize;
    push(Cmd::Open { width: w, height: h, out: (ow, oh), hwnd });
    opened(w, h, format, origin);
    flush();
    1
}

/// The later Glide 2.x open (POD Gold's 3dfx build): the game names its window.
#[unsafe(no_mangle)]
pub extern "system" fn grSstWinOpen(hwnd: usize, res: u32, refresh: u32, format: u32, origin: u32, colour: u32, aux: u32) -> u32 {
    let (w, h) = consts::resolution(res).unwrap_or((640, 480));
    log!("grSstWinOpen {w}x{h} (window {hwnd:#x}, res {res}, refresh {refresh}, format {format}, origin {origin}, {colour} colour and {aux} aux buffers)");
    let (ow, oh) = crate::config::get().output(w, h);
    let hwnd = if hwnd == 0 { crate::window::prepare(ow, oh) as usize } else { crate::window::size(hwnd as _, ow, oh) as usize };
    push(Cmd::Open { width: w, height: h, out: (ow, oh), hwnd });
    opened(w, h, format, origin);
    flush();
    1 // a context handle the game only passes back
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstWinClose() -> u32 {
    flush();
    1
}

/// `GR_CONTROL_ACTIVATE` (1), `DEACTIVATE` (2), `RESIZE` (3), `MOVE` (4): the window is ours to
/// keep the size of, so nothing to do.
#[unsafe(no_mangle)]
pub extern "system" fn grSstControl(code: u32) -> u32 {
    log!("grSstControl {code}");
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstOrigin(origin: u32) {
    super::shadow::origin(origin & 1);
}

/// Sets one register of the FBI or a TMU (`which` 3, or 0–2), for the chip's pipeline. The
/// render state here is OpenGL's, so there is nothing to set.
///
/// Original: `grSstConfigPipeline` (glide2x.dll 2.43, 0x10009c40) writes `value` to register
/// `param` of that chip (*measured*).
#[unsafe(no_mangle)]
pub extern "system" fn grSstConfigPipeline(which: u32, param: u32, value: u32) {
    log!("grSstConfigPipeline {which} {param} {value:#x}: ignored");
}

/// A `GrVidTiming` for board `which`, for the next open. The window and its size are this
/// library's, so the timing is not used.
///
/// Original: `grSstVidMode` (glide2x.dll 2.43, 0x10004660) only stores the pointer in the
/// board's settings (*measured*).
#[unsafe(no_mangle)]
pub extern "system" fn grSstVidMode(which: u32, _timing: *const u8) {
    log!("grSstVidMode {which}: ignored");
}

/// The Voodoo hands the monitor to the 2D card when this is 1 (the videos, dialogs). Here the
/// window stays; nothing to do.
#[unsafe(no_mangle)]
pub extern "system" fn grSstPassthruMode(mode: u32) {
    log!("grSstPassthruMode {mode}");
}

#[unsafe(no_mangle)]
pub extern "system" fn grBufferSwap(_interval: u32) {
    super::lfb::finish_write();
    push(Cmd::Swap);
    flush();
    super::movie::frame();
}

#[unsafe(no_mangle)]
pub extern "system" fn grBufferClear(color: u32, alpha: u32, depth: u32) {
    push(Cmd::Clear { color: argb(color), alpha: alpha & 0xff, depth: depth & 0xffff });
}

#[unsafe(no_mangle)]
pub extern "system" fn grClipWindow(minx: u32, miny: u32, maxx: u32, maxy: u32) {
    super::shadow::clip([minx, miny, maxx, maxy]);
}

#[unsafe(no_mangle)]
pub extern "system" fn grGammaCorrectionValue(gamma: f32) {
    push(Cmd::Gamma(gamma));
}
