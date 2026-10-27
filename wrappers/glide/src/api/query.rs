//! Queries and the calls that have nothing to draw: status, retrace, statistics, the version,
//! the splash screen, and the `gu` fog tables (`fog`).

use super::draw::TRIS;
use super::sst::screen;
use crate::fog;
use std::sync::atomic::{AtomicU32, Ordering};

/// Idle, with the memory FIFO (bits 12–27) and the PCI FIFO (bits 0–5) empty, and in vertical
/// retrace (bit 6 clear: Glide 2.43's `grSstVRetraceOn` tests it so, *measured*). POD waits
/// for bits 12–27 to read 0xffff before a large write.
#[unsafe(no_mangle)]
pub extern "system" fn grSstStatus() -> u32 {
    shim::call!("grSstStatus");
    0x0fff_f03f
}

/// The commands go to the render thread at each swap; there is nothing to wait for here.
#[unsafe(no_mangle)]
pub extern "system" fn grSstIdle() {
    shim::call!("grSstIdle");
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstIsBusy() -> u32 {
    shim::call!("grSstIsBusy");
    0
}

/// Always in retrace, so a game that waits for it goes on at once. Deviation: the window is
/// shown at each swap with no wait for the monitor.
#[unsafe(no_mangle)]
pub extern "system" fn grSstVRetraceOn() -> u32 {
    shim::call!("grSstVRetraceOn");
    1
}

/// The scan line, counted up at each call so that a game polling for a line reaches it.
#[unsafe(no_mangle)]
pub extern "system" fn grSstVideoLine() -> u32 {
    shim::call!("grSstVideoLine");
    static LINE: AtomicU32 = AtomicU32::new(0);
    LINE.fetch_add(1, Ordering::Relaxed) % screen().1.max(1)
}

#[unsafe(no_mangle)]
pub extern "system" fn grBufferNumPending() -> i32 {
    shim::call!("grBufferNumPending");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstScreenWidth() -> u32 {
    shim::call!("grSstScreenWidth");
    screen().0
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstScreenHeight() -> u32 {
    shim::call!("grSstScreenHeight");
    screen().1
}

/// Only the number of boards: one.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grSstQueryBoards(hw: *mut u32) -> u32 {
    shim::call!("grSstQueryBoards", hw);
    if hw.is_null() {
        return 0;
    }
    unsafe { hw.write_unaligned(1) };
    1
}

/// `GrSstPerfStats_t`: pixels in, failing the chroma key, the depth test and the alpha test,
/// and pixels out. Deviation: the pixels are counted by OpenGL, not here, so all are 0.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grSstPerfStats(stats: *mut u32) {
    shim::call!("grSstPerfStats", stats);
    if !stats.is_null() {
        unsafe { std::ptr::write_bytes(stats, 0, 5) };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grSstResetPerfStats() {
    shim::call!("grSstResetPerfStats");
}

/// Triangles given to the drawing calls, and those drawn after culling.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTriStats(processed: *mut u32, drawn: *mut u32) {
    shim::call!("grTriStats", processed, drawn);
    unsafe {
        if !processed.is_null() {
            processed.write_unaligned(TRIS[0].load(Ordering::Relaxed));
        }
        if !drawn.is_null() {
            drawn.write_unaligned(TRIS[1].load(Ordering::Relaxed));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grResetTriStats() {
    shim::call!("grResetTriStats");
    TRIS.iter().for_each(|t| t.store(0, Ordering::Relaxed));
}

/// The version string Glide 2.43 copies out: "2.43" (*measured*, the Win95 `glide2x.dll`).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grGlideGetVersion(version: *mut u8) {
    shim::call!("grGlideGetVersion", version);
    if !version.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(c"2.43".as_ptr().cast::<u8>(), version, 5) };
    }
}

/// The callback for Glide's errors. This library raises none, so it is never called.
#[unsafe(no_mangle)]
pub extern "system" fn grErrorSetCallback(_callback: usize) {
    shim::call!("grErrorSetCallback", _callback);
}

/// Deviation: no 3Dfx logo. The splash screen and the "shameless plug" were the Voodoo's own.
#[unsafe(no_mangle)]
pub extern "system" fn grGlideShamelessPlug(_on: u32) {
    shim::call!("grGlideShamelessPlug", _on);
}

#[unsafe(no_mangle)]
pub extern "system" fn grSplash(_x: f32, _y: f32, _w: f32, _h: f32, _frame: u32) {
    shim::call!("grSplash", _x, _y, _w, _h, _frame);
}

#[unsafe(no_mangle)]
pub extern "system" fn grSplash21() {
    shim::call!("grSplash21");
}

/// Room in the command FIFO: there always is.
#[unsafe(no_mangle)]
pub extern "system" fn grCheckForRoom(_bytes: i32) {
    shim::call!("grCheckForRoom", _bytes);
}

unsafe fn put(table: *mut u8, t: [u8; fog::TABLE_SIZE]) {
    if !table.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(t.as_ptr(), table, fog::TABLE_SIZE) };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn guFogTableIndexToW(i: i32) -> f32 {
    shim::call!("guFogTableIndexToW", i);
    fog::index_to_w(i.clamp(0, 63) as usize) as f32
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFogGenerateExp(table: *mut u8, density: f32) {
    shim::call!("guFogGenerateExp", table, density);
    unsafe { put(table, fog::exp(density)) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFogGenerateExp2(table: *mut u8, density: f32) {
    shim::call!("guFogGenerateExp2", table, density);
    unsafe { put(table, fog::exp2(density)) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFogGenerateLinear(table: *mut u8, near: f32, far: f32) {
    shim::call!("guFogGenerateLinear", table, near, far);
    unsafe { put(table, fog::linear(near, far)) };
}

/// The two bytes of a 16-bit value swapped (*measured*: Glide 2.43 swaps the low word only).
#[unsafe(no_mangle)]
pub extern "system" fn guEndianSwapBytes(v: u32) -> u32 {
    shim::call!("guEndianSwapBytes", v);
    (v as u16).swap_bytes() as u32
}

/// The two 16-bit halves of a word swapped.
#[unsafe(no_mangle)]
pub extern "system" fn guEndianSwapWords(v: u32) -> u32 {
    shim::call!("guEndianSwapWords", v);
    v.rotate_left(16)
}
