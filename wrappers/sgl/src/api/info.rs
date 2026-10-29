//! Errors, versions, texture memory and the output modes. What each returns was read from
//! PowerVR's own SGLMID4.dll of driver 3.1.0 (*observed*). The version strings are that
//! driver's.

use super::{error, sgl, trace};
use std::sync::Mutex;

/// The earliest and the most recent error since the last `sgl_get_errors`, as SGL keeps
/// them (`FUN_10005110`): each call sets the most recent, and the earliest is set while it is 0.
static ERRORS: Mutex<(i32, i32)> = Mutex::new((0, 0));

pub fn record(e: i32) {
    let mut g = ERRORS.lock().unwrap_or_else(|p| p.into_inner());
    if g.0 == 0 {
        g.0 = e;
    }
    g.1 = e;
}

/// Both errors, then cleared. Deviation: only the immediate-mode calls and the calls in this
/// module record their errors.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_get_errors(earliest: *mut i32, most_recent: *mut i32) {
    shim::call!("sgl_get_errors", earliest, most_recent);
    let (e, r) = std::mem::take(&mut *ERRORS.lock().unwrap_or_else(|p| p.into_inner()));
    trace!("sgl_get_errors", "-> {e} {r}");
    unsafe {
        if !earliest.is_null() {
            earliest.write_unaligned(e);
        }
        if !most_recent.is_null() {
            most_recent.write_unaligned(r);
        }
    }
}

/// A table of C string pointers handed to the game.
#[repr(C)]
pub struct Strings<const N: usize>([*const u8; N]);
unsafe impl<const N: usize> Sync for Strings<N> {}

/// `sgl_versions`: the library and the sgl.h it needs.
static VERSIONS: Strings<2> = Strings([c"3.1.0c3".as_ptr().cast(), c"1.27".as_ptr().cast()]);

#[unsafe(no_mangle)]
pub extern "C" fn sgl_get_versions() -> *const Strings<2> {
    shim::call!("sgl_get_versions");
    trace!("sgl_get_versions");
    &VERSIONS
}

/// `sgl_win_versions` (sglwin32.h): 13 strings, one table for each output mode, which the
/// `mode` field names. The PCI fields name a PCX2 behind NEC's bridge (sglwin32.h's IDs, in
/// SGL's `%04Xh` form).
macro_rules! win_versions {
    ($mode:literal) => {
        Strings([
            c"1.7".as_ptr().cast(),
            c"".as_ptr().cast(),
            c"1033h".as_ptr().cast(),
            c"001Fh".as_ptr().cast(),
            c"0".as_ptr().cast(),
            c"0".as_ptr().cast(),
            c"00000000h".as_ptr().cast(),
            c"0046h".as_ptr().cast(),
            c"4".as_ptr().cast(),
            c"0046h".as_ptr().cast(),
            $mode.as_ptr().cast(),
            c"".as_ptr().cast(),
            c"".as_ptr().cast(),
        ])
    };
}

static WIN_ADDRESS: Strings<13> = win_versions!(c"address");
static WIN_DDRAW: Strings<13> = win_versions!(c"direct draw");

#[unsafe(no_mangle)]
pub extern "C" fn sgl_get_win_versions() -> *const Strings<13> {
    shim::call!("sgl_get_win_versions");
    trace!("sgl_get_win_versions");
    if crate::immediate::ddraw_mode() { &WIN_DDRAW } else { &WIN_ADDRESS }
}

/// The driver's counter: a 4 MB board's memory less the charge of each texture that exists
/// (`scene::texmem`, with its deviations). Original: `sgl_get_free_texture_mem` at `10015830`.
#[unsafe(no_mangle)]
pub extern "C" fn sgl_get_free_texture_mem() -> u32 {
    shim::call!("sgl_get_free_texture_mem");
    let free = sgl().texmem.free();
    trace!("sgl_get_free_texture_mem", "-> {free}");
    error(0);
    free
}

/// Not in driver 3.1.0: Hardwar imports it from a later SGL. It takes no arguments here
/// (*inferred* from its name).
#[unsafe(no_mangle)]
pub extern "C" fn sgl_delete_all_textures() {
    shim::call!("sgl_delete_all_textures");
    let names: Vec<u32> = {
        let mut s = sgl();
        let names: Vec<u32> = s.scene.textures.keys().copied().collect();
        s.scene.textures.clear();
        s.texmem.give_back_all();
        names
    };
    trace!("sgl_delete_all_textures", "{} textures", names.len());
    for n in names {
        crate::render::send(crate::render::Cmd::DeleteTexture(n));
    }
}

/// `sgl_err_mode_already_defined` (sglwin32.h).
const ERR_MODE_ALREADY_DEFINED: i32 = -1023;

/// SGL draws into the window itself and calls `proc_2d` with each finished frame's surface
/// (`immediate`).
#[unsafe(no_mangle)]
pub extern "C" fn sgl_use_ddraw_mode(hwnd: usize, proc_2d: usize) -> i32 {
    shim::call!("sgl_use_ddraw_mode", hwnd, proc_2d);
    shim::log!("sgl_use_ddraw_mode window {hwnd:#x} 2D callback {proc_2d:#x}");
    if !crate::immediate::set_ddraw(hwnd, proc_2d) {
        error(ERR_MODE_ALREADY_DEFINED);
        return ERR_MODE_ALREADY_DEFINED;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_use_eor_callback(eor: usize) -> i32 {
    shim::call!("sgl_use_eor_callback", eor);
    shim::log!("sgl_use_eor_callback {eor:#x}");
    crate::immediate::set_eor(eor);
    0
}
