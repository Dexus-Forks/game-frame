//! The exports, and the game's wave outputs followed through its import table.

use crate::streams::{MAP_WORDS, Streams, stereo_volume};
use shim::{log, log_once};
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[link(name = "winmm")]
unsafe extern "system" {
    fn waveOutSetVolume(handle: usize, volume: u32) -> u32;
}

type WaveOutOpen = unsafe extern "system" fn(*mut usize, u32, *const c_void, usize, usize, u32) -> u32;
type WaveOutClose = unsafe extern "system" fn(usize) -> u32;
type WaveOutWrite = unsafe extern "system" fn(usize, *const u8, u32) -> u32;

static STREAMS: Mutex<Streams> = Mutex::new(Streams::new());
static REAL_OPEN: AtomicUsize = AtomicUsize::new(0);
static REAL_CLOSE: AtomicUsize = AtomicUsize::new(0);
static REAL_WRITE: AtomicUsize = AtomicUsize::new(0);
/// Streams whose first sound has been logged, one bit each.
static HEARD: AtomicUsize = AtomicUsize::new(0);

const WAVE_FORMAT_QUERY: u32 = 1;

fn streams() -> std::sync::MutexGuard<'static, Streams> {
    STREAMS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Hook the game's `waveOutOpen`, `waveOutClose` and `waveOutWrite`, and its mixer calls.
/// Called from `DllMain` and again before the map is read, in case the loader had not filled
/// the import table yet.
fn hook() {
    if let Some(old) = shim::iat::hook("winmm.dll", "waveOutOpen", open as *const () as usize) {
        REAL_OPEN.store(old, Ordering::Relaxed);
    }
    if let Some(old) = shim::iat::hook("winmm.dll", "waveOutClose", close as *const () as usize) {
        REAL_CLOSE.store(old, Ordering::Relaxed);
    }
    if let Some(old) = shim::iat::hook("winmm.dll", "waveOutWrite", write as *const () as usize) {
        REAL_WRITE.store(old, Ordering::Relaxed);
    }
    crate::mixer_dll::hook();
}

unsafe extern "system" fn open(out: *mut usize, device: u32, format: *const c_void, cb: usize, inst: usize, flags: u32) -> u32 {
    let real: WaveOutOpen = unsafe { std::mem::transmute(REAL_OPEN.load(Ordering::Relaxed)) };
    let r = unsafe { real(out, device, format, cb, inst, flags) };
    if r == 0 && !out.is_null() && flags & WAVE_FORMAT_QUERY == 0 {
        let h = unsafe { out.read() };
        match streams().open(h) {
            Some(n) => log!("waveOutOpen {h:#x}: stream {n}"),
            None => log!("waveOutOpen {h:#x}: every stream taken, not listed"),
        }
    }
    r
}

unsafe extern "system" fn close(h: usize) -> u32 {
    let real: WaveOutClose = unsafe { std::mem::transmute(REAL_CLOSE.load(Ordering::Relaxed)) };
    if let Some(n) = streams().close(h) {
        log!("waveOutClose {h:#x}: stream {n} free");
    }
    unsafe { real(h) }
}

/// Passed on unchanged; the first buffer of each stream that is not silence is logged, with
/// its loudest 16-bit sample, to show the game's sound reaches the output.
unsafe extern "system" fn write(h: usize, header: *const u8, size: u32) -> u32 {
    let real: WaveOutWrite = unsafe { std::mem::transmute(REAL_WRITE.load(Ordering::Relaxed)) };
    let n = streams().stream_of(h);
    if let Some(n) = n.filter(|&n| HEARD.load(Ordering::Relaxed) & 1 << n == 0) {
        let (data, len) = unsafe { ((header as *const *const i16).read(), (header.add(4) as *const u32).read()) };
        let peak = unsafe { std::slice::from_raw_parts(data, len as usize / 2) }.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        if peak > 0 && HEARD.fetch_or(1 << n, Ordering::Relaxed) & 1 << n == 0 {
            log!("stream {n}: first sound, {len} bytes, loudest sample {peak}");
        }
    }
    unsafe { real(h, header, size) }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        shim::log::init("maxi94", "MAXI94_LOG");
        shim::ports::install();
        shim::patch::apply();
        hook();
    }
    1
}

/// A command to the card's MPU-401 side, with `count` bytes at `data`.
#[unsafe(no_mangle)]
pub extern "system" fn mpuMessage(command: u32, _data: *const u8, count: u32) -> u32 {
    log_once!("mpuMessage {command:#x}, {count} bytes (not handled)");
    1
}

/// The same, acknowledged: non-zero means the card took it.
#[unsafe(no_mangle)]
pub extern "system" fn mpuMessageAc(command: u32, data: *const u8, count: u32, _flags: u32) -> u32 {
    let first = if data.is_null() || count == 0 { String::new() } else { format!(" {:#04x}", unsafe { data.read() }) };
    log!("mpuMessageAc {command:#x}{first}, {count} bytes (acknowledged)");
    1
}

/// `size` words of the card's memory at (`hi`, `lo`) into `buffer`: the memory map, which
/// lists the streams the open wave outputs hold.
#[unsafe(no_mangle)]
pub extern "system" fn UpLoad(buffer: *mut u16, hi: u32, lo: u32, size: u32) -> u32 {
    hook();
    let map = streams().map();
    let n = (size as usize).min(MAP_WORDS);
    if !buffer.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(map.as_ptr(), buffer, n) };
    }
    if size as usize > MAP_WORDS {
        log_once!("UpLoad {size:#x} words at {hi:#x}:{lo:#x}: only the map's {MAP_WORDS:#x} given");
    }
    1
}

/// Where the memory map is on the card: any address does, as `UpLoad` gives the map for all.
#[unsafe(no_mangle)]
pub extern "system" fn GetMemMapAddress() -> u32 {
    hook();
    0
}

fn set_volume(stream: u32, left: u32, right: u32, aux: bool) {
    let mut s = streams();
    let Some(st) = s.get_mut(stream as usize & 0xffff) else {
        log_once!("wave volume for stream {stream:#x}, which no wave output holds");
        return;
    };
    let v = (left as u16, right as u16);
    if aux { st.aux = v } else { st.main = v }
    let (h, vol) = (st.handle, stereo_volume(st));
    drop(s);
    let r = unsafe { waveOutSetVolume(h, vol) };
    let which = if aux { "aux" } else { "main" };
    log!("stream {stream} {which} {left:#x},{right:#x}: wave output {h:#x} at {vol:#010x} ({r})");
}

/// Stream `stream`'s aux (rear) volume, left and right, 0 to 0xffff.
#[unsafe(no_mangle)]
pub extern "system" fn waveSetAuxVolume(stream: u32, left: u32, right: u32) -> u32 {
    set_volume(stream, left, right, true);
    1
}

/// Stream `stream`'s main (front) volume, left and right, 0 to 0xffff.
#[unsafe(no_mangle)]
pub extern "system" fn waveSetMainVolume(stream: u32, left: u32, right: u32) -> u32 {
    set_volume(stream, left, right, false);
    1
}
