//! `IDirectDraw` and `IDirectDraw2` (POD asks for the second with `QueryInterface`). One
//! display state behind both: the window, and the mode the game set. Every mode it asks for
//! is granted: the frame stays in memory and is shown scaled (`present`).

use crate::com::*;
use crate::desc::{self, Desc};
use crate::surface::{self, Surface};
use shim::log;
use std::ffi::c_void;
use std::sync::OnceLock;

#[repr(C)]
pub struct DirectDraw {
    vtbl: *const Slot,
    refs: u32,
}

/// The modes offered, each at 8 and 16 bits: fewer than POD's mode table holds (56,
/// `docs/running.md`).
const MODES: [(u32, u32); 7] = [(320, 200), (320, 240), (512, 384), (640, 400), (640, 480), (800, 600), (1024, 768)];

static V1: OnceLock<Vec<usize>> = OnceLock::new();
static V2: OnceLock<Vec<usize>> = OnceLock::new();

fn common() -> Vec<(usize, Slot)> {
    vec![
        (0, query as Slot),
        (1, add_ref as Slot),
        (2, release as Slot),
        (4, create_clipper as Slot),
        (5, create_palette as Slot),
        (6, create_surface as Slot),
        (8, enum_modes as Slot),
        (10, ok0 as Slot),
        (11, get_caps as Slot),
        (12, get_display_mode as Slot),
        (19, ok0 as Slot),
        (20, set_cooperative_level as Slot),
        (22, ok2 as Slot),
    ]
}

fn finish(t: Vec<Slot>) -> Vec<usize> {
    t.into_iter().map(|p| p as usize).collect()
}

pub fn new(version: u32) -> *mut DirectDraw {
    let vtbl = if version == 1 {
        V1.get_or_init(|| {
            let mut m = common();
            m.push((21, set_display_mode1 as Slot));
            let mut stubs = direct_draw::stubs();
            stubs.truncate(23);
            finish(table(stubs, &m))
        })
    } else {
        V2.get_or_init(|| {
            let mut m = common();
            m.push((21, set_display_mode2 as Slot));
            m.push((23, get_available_vid_mem as Slot));
            finish(table(direct_draw::stubs(), &m))
        })
    };
    Box::into_raw(Box::new(DirectDraw { vtbl: vtbl.as_ptr() as *const Slot, refs: 1 }))
}

unsafe extern "system" fn query(_this: *mut DirectDraw, iid: *const u32, out: *mut *mut DirectDraw) -> Hr {
    let first = if iid.is_null() { 0 } else { unsafe { iid.read_unaligned() } };
    log!("IDirectDraw::QueryInterface {first:08x}: IDirectDraw2");
    unsafe { out.write(new(2)) };
    DD_OK
}

unsafe extern "system" fn add_ref(this: *mut DirectDraw) -> u32 {
    unsafe {
        (*this).refs += 1;
        (*this).refs
    }
}

unsafe extern "system" fn release(this: *mut DirectDraw) -> u32 {
    unsafe {
        (*this).refs = (*this).refs.saturating_sub(1);
        (*this).refs
    }
}

unsafe extern "system" fn ok0(_this: *mut DirectDraw) -> Hr {
    DD_OK
}

unsafe extern "system" fn ok2(_this: *mut DirectDraw, _a: u32, _b: u32) -> Hr {
    DD_OK
}

unsafe extern "system" fn create_clipper(_this: *mut DirectDraw, _flags: u32, out: *mut *mut c_void, _outer: u32) -> Hr {
    unsafe { out.write(crate::clipper::new().cast()) };
    DD_OK
}

unsafe extern "system" fn create_palette(_this: *mut DirectDraw, _flags: u32, entries: *const [u8; 4], out: *mut *mut c_void, _outer: u32) -> Hr {
    unsafe { out.write(crate::palette::Palette::new(entries).cast()) };
    DD_OK
}

/// The primary surface takes the display mode (and a back buffer when asked: the flip
/// chain); any other takes the size and depth it describes, or the mode's depth.
unsafe extern "system" fn create_surface(_this: *mut DirectDraw, d: *const u32, out: *mut *mut Surface, _outer: u32) -> Hr {
    let (flags, want) = unsafe { desc::read(d) };
    let (mw, mh, mbpp) = crate::present::mode();
    let s = if want.caps & surface::PRIMARYSURFACE != 0 {
        let chain = flags & desc::BACKBUFFERCOUNT != 0 && want.back_count > 0;
        let caps = want.caps | surface::VIDEOMEMORY | if chain { surface::FRONTBUFFER | surface::FLIP | surface::COMPLEX } else { 0 };
        let p = Surface::new(mw, mh, mbpp, caps);
        if chain {
            let back = Surface::new(mw, mh, mbpp, surface::BACKBUFFER | surface::FLIP | surface::COMPLEX | surface::VIDEOMEMORY);
            unsafe { (*p).back = back };
        }
        crate::present::set_primary(p);
        p
    } else {
        let bpp = if want.bpp != 0 { want.bpp } else { mbpp };
        let mem = if want.caps & surface::SYSTEMMEMORY != 0 { 0 } else { surface::VIDEOMEMORY };
        Surface::new(want.w, want.h, bpp, want.caps | mem)
    };
    let sr = unsafe { &*s };
    log!("CreateSurface flags {flags:#x} caps {:#x}: {}x{}x{} (back buffer {})", want.caps, sr.w, sr.h, sr.bpp, !sr.back.is_null());
    unsafe { out.write(s) };
    DD_OK
}

type EnumCallback = unsafe extern "system" fn(*mut u32, *mut c_void) -> u32;

unsafe extern "system" fn enum_modes(_this: *mut DirectDraw, _flags: u32, _filter: *const u32, ctx: *mut c_void, cb: EnumCallback) -> Hr {
    for (w, h) in MODES {
        for bpp in [8, 16] {
            let mut d = [0u32; 0x6c / 4];
            d[0] = 0x6c;
            let info = Desc { w, h, bpp, pitch: w * bpp / 8, ..Default::default() };
            unsafe { desc::write(d.as_mut_ptr(), &info) };
            if unsafe { cb(d.as_mut_ptr(), ctx) } == 0 {
                return DD_OK;
            }
        }
    }
    DD_OK
}

/// Zeroes the caps (keeping `dwSize`), with 8 MB of video memory, total and free.
unsafe extern "system" fn get_caps(_this: *mut DirectDraw, drv: *mut u32, hel: *mut u32) -> Hr {
    for c in [drv, hel] {
        if !c.is_null() {
            unsafe {
                let size = c.read_unaligned() as usize;
                std::ptr::write_bytes(c.add(1).cast::<u8>(), 0, size.saturating_sub(4));
                if size >= 0x44 {
                    c.add(0x3c / 4).write_unaligned(8 << 20);
                    c.add(0x40 / 4).write_unaligned(8 << 20);
                }
            }
        }
    }
    DD_OK
}

unsafe extern "system" fn get_display_mode(_this: *mut DirectDraw, d: *mut u32) -> Hr {
    let (w, h, bpp) = crate::present::mode();
    unsafe { desc::write(d, &Desc { w, h, bpp, pitch: w * bpp / 8, ..Default::default() }) };
    DD_OK
}

unsafe extern "system" fn set_cooperative_level(_this: *mut DirectDraw, hwnd: *mut c_void, flags: u32) -> Hr {
    log!("SetCooperativeLevel {hwnd:?} {flags:#x}");
    crate::present::set_window(hwnd);
    DD_OK
}

unsafe extern "system" fn set_display_mode1(_this: *mut DirectDraw, w: u32, h: u32, bpp: u32) -> Hr {
    log!("SetDisplayMode {w}x{h}x{bpp}");
    crate::present::set_mode(w, h, bpp);
    DD_OK
}

unsafe extern "system" fn set_display_mode2(this: *mut DirectDraw, w: u32, h: u32, bpp: u32, _refresh: u32, _flags: u32) -> Hr {
    unsafe { set_display_mode1(this, w, h, bpp) }
}

unsafe extern "system" fn get_available_vid_mem(_this: *mut DirectDraw, _caps: *const u32, total: *mut u32, free: *mut u32) -> Hr {
    for p in [total, free] {
        if !p.is_null() {
            unsafe { p.write_unaligned(8 << 20) };
        }
    }
    DD_OK
}
