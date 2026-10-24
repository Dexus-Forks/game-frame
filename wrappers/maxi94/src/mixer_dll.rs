//! The game's mixer calls, hooked in its import table: the card's two source lines (`mixer`)
//! answer for themselves, and the system's mixer answers the rest.
//!
//! The structures are the 32-bit ANSI ones: MIXERLINEA (`0xa8` bytes), MIXERLINECONTROLSA
//! (`0x18`), MIXERCONTROLA (`0x94`), MIXERCONTROLDETAILS (`0x18`).

use crate::mixer::{CONTROL_VOLUME, Line, Mixer, VOLUME_MAX, name_field};
use shim::log;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

type MixerCall = unsafe extern "system" fn(usize, *mut u8, u32) -> u32;

static MIXER: Mutex<Mixer> = Mutex::new(Mixer::new());
static REAL: [AtomicUsize; 4] = [const { AtomicUsize::new(0) }; 4];
const NAMES: [&str; 4] = ["mixerGetLineInfoA", "mixerGetLineControlsA", "mixerGetControlDetailsA", "mixerSetControlDetails"];

const MMSYSERR_INVALPARAM: u32 = 11;
const QUERY_MASK: u32 = 0xf;
const LINEINFO_LINEID: u32 = 2;
const LINEINFO_COMPONENTTYPE: u32 = 3;
const LINECONTROLS_ALL: u32 = 0;
const LINECONTROLS_ONEBYID: u32 = 1;
const LINECONTROLS_ONEBYTYPE: u32 = 2;
const MIXERLINE_LINEF_ACTIVE: u32 = 1;
const MIXERLINE_LINEF_SOURCE: u32 = 0x8000_0000;

pub fn hook() {
    let new: [usize; 4] = [
        line_info as *const () as usize,
        line_controls as *const () as usize,
        get_details as *const () as usize,
        set_details as *const () as usize,
    ];
    for i in 0..4 {
        if let Some(old) = shim::iat::hook("winmm.dll", NAMES[i], new[i]) {
            REAL[i].store(old, Ordering::Relaxed);
        }
    }
}

fn mixer() -> std::sync::MutexGuard<'static, Mixer> {
    MIXER.lock().unwrap_or_else(|e| e.into_inner())
}

unsafe fn real(i: usize, h: usize, p: *mut u8, flags: u32) -> u32 {
    let f: MixerCall = unsafe { std::mem::transmute(REAL[i].load(Ordering::Relaxed)) };
    unsafe { f(h, p, flags) }
}

unsafe fn rd(p: *const u8, at: usize) -> u32 {
    unsafe { (p.add(at) as *const u32).read_unaligned() }
}

unsafe fn wr(p: *mut u8, at: usize, v: u32) {
    unsafe { (p.add(at) as *mut u32).write_unaligned(v) }
}

unsafe fn write_bytes(p: *mut u8, at: usize, b: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), p.add(at), b.len()) }
}

/// A MIXERLINEA for `line`, a source of destination 0 with one volume control.
unsafe fn fill_line(p: *mut u8, line: &Line) {
    unsafe {
        std::ptr::write_bytes(p.add(4), 0, 0xa8 - 4);
        wr(p, 0x0c, line.line_id());
        wr(p, 0x10, MIXERLINE_LINEF_ACTIVE | MIXERLINE_LINEF_SOURCE);
        wr(p, 0x18, line.component);
        wr(p, 0x1c, 1); // cChannels
        wr(p, 0x24, 1); // cControls
        write_bytes(p, 0x28, &name_field::<16>(line.name));
        write_bytes(p, 0x38, &name_field::<64>(line.name));
    }
}

/// A MIXERCONTROLA for `line`'s volume, 0 to `VOLUME_MAX`.
unsafe fn fill_control(p: *mut u8, line: &Line) {
    unsafe {
        std::ptr::write_bytes(p, 0, 0x94);
        wr(p, 0x00, 0x94);
        wr(p, 0x04, line.control_id());
        wr(p, 0x08, CONTROL_VOLUME);
        write_bytes(p, 0x14, &name_field::<16>("Volume"));
        write_bytes(p, 0x24, &name_field::<64>(line.name));
        wr(p, 0x68, VOLUME_MAX); // lMaximum
        wr(p, 0x7c, 0x40); // cSteps
    }
}

unsafe extern "system" fn line_info(h: usize, p: *mut u8, flags: u32) -> u32 {
    if p.is_null() {
        return unsafe { real(0, h, p, flags) };
    }
    let m = mixer();
    let line = match flags & QUERY_MASK {
        LINEINFO_COMPONENTTYPE => m.by_component(unsafe { rd(p, 0x18) }),
        LINEINFO_LINEID => m.by_line(unsafe { rd(p, 0x0c) }),
        _ => None,
    };
    let Some(line) = line.copied() else {
        drop(m);
        return unsafe { real(0, h, p, flags) };
    };
    unsafe { fill_line(p, &line) };
    log!("mixerGetLineInfoA: the card's {:?} line", line.name);
    0
}

unsafe extern "system" fn line_controls(h: usize, p: *mut u8, flags: u32) -> u32 {
    if p.is_null() {
        return unsafe { real(1, h, p, flags) };
    }
    let (id, which, ctrl) = unsafe { (rd(p, 4), rd(p, 8), rd(p, 0x14) as *mut u8) };
    let m = mixer();
    let line = match flags & QUERY_MASK {
        LINECONTROLS_ALL => m.by_line(id),
        LINECONTROLS_ONEBYTYPE if which == CONTROL_VOLUME => m.by_line(id),
        LINECONTROLS_ONEBYID => m.by_control_id(which),
        _ => None,
    };
    let Some(line) = line.copied() else {
        drop(m);
        return unsafe { real(1, h, p, flags) };
    };
    if ctrl.is_null() || unsafe { rd(p, 0x10) } < 0x94 {
        return MMSYSERR_INVALPARAM;
    }
    unsafe {
        wr(p, 4, line.line_id());
        wr(p, 0x0c, 1);
        fill_control(ctrl, &line);
    }
    0
}

/// `None` when the control is not one of the card's lines.
unsafe fn details(p: *mut u8, set: bool) -> Option<u32> {
    if p.is_null() {
        return None;
    }
    let (id, channels, size, values) = unsafe { (rd(p, 4), rd(p, 8).max(1), rd(p, 0x10), rd(p, 0x14) as *mut u8) };
    let mut m = mixer();
    let line = m.by_control(id)?;
    if size < 4 || values.is_null() {
        return Some(MMSYSERR_INVALPARAM);
    }
    for c in 0..channels as usize {
        if set {
            if c == 0 {
                line.volume = unsafe { rd(values, 0) }.min(VOLUME_MAX);
                log!("{} volume {:#x}", line.name, line.volume);
            }
        } else {
            unsafe { wr(values, c * size as usize, line.volume) };
        }
    }
    Some(0)
}

unsafe extern "system" fn get_details(h: usize, p: *mut u8, flags: u32) -> u32 {
    match unsafe { details(p, false) } {
        Some(r) => r,
        None => unsafe { real(2, h, p, flags) },
    }
}

unsafe extern "system" fn set_details(h: usize, p: *mut u8, flags: u32) -> u32 {
    match unsafe { details(p, true) } {
        Some(r) => r,
        None => unsafe { real(3, h, p, flags) },
    }
}
