//! `IDirectDrawSurface` (with the slots of versions 2 and 3, so a surface answers any of
//! them): plain memory, 8 or 16 bits a pixel, locked by pointer, copied by `Blt` and
//! `BltFast`, and shown by `Flip` on the primary surface.

use crate::com::*;
use crate::desc::{self, Desc};
use shim::log;
use std::ffi::c_void;
use std::sync::OnceLock;

// DDSCAPS_*
pub const BACKBUFFER: u32 = 0x4;
pub const COMPLEX: u32 = 0x8;
pub const FLIP: u32 = 0x10;
pub const FRONTBUFFER: u32 = 0x20;
pub const PRIMARYSURFACE: u32 = 0x200;
pub const SYSTEMMEMORY: u32 = 0x800;
pub const VIDEOMEMORY: u32 = 0x4000;

#[repr(C)]
pub struct Surface {
    vtbl: *const Slot,
    refs: u32,
    pub w: u32,
    pub h: u32,
    /// Bits per pixel: 8 or 16.
    pub bpp: u32,
    pub pitch: u32,
    pub caps: u32,
    pub mem: Vec<u8>,
    /// The back buffer of a flip chain (on the primary surface).
    pub back: *mut Surface,
    pub palette: *mut crate::palette::Palette,
    /// Source colour key, used by `Blt` and `BltFast` when asked (`DDBLT_KEYSRC`).
    pub src_key: Option<u32>,
}

static VTBL: OnceLock<Vec<usize>> = OnceLock::new();

fn vtbl() -> *const Slot {
    VTBL.get_or_init(|| {
        table(surface::stubs(), &[
            (0, query as Slot),
            (1, add_ref as Slot),
            (2, release as Slot),
            (5, blt as Slot),
            (7, blt_fast as Slot),
            (11, flip as Slot),
            (12, get_attached as Slot),
            (14, get_caps as Slot),
            (21, get_pixel_format as Slot),
            (22, get_desc as Slot),
            (24, is_lost as Slot),
            (25, lock as Slot),
            (27, restore as Slot),
            (28, set_clipper as Slot),
            (29, set_color_key as Slot),
            (31, set_palette as Slot),
            (32, unlock as Slot),
        ])
        .into_iter()
        .map(|p| p as usize)
        .collect()
    })
    .as_ptr() as *const Slot
}

impl Surface {
    pub fn new(w: u32, h: u32, bpp: u32, caps: u32) -> *mut Surface {
        let pitch = (w * bpp / 8 + 3) & !3;
        Box::into_raw(Box::new(Surface {
            vtbl: vtbl(),
            refs: 1,
            w,
            h,
            bpp,
            pitch,
            caps,
            mem: vec![0; (pitch * h) as usize],
            back: std::ptr::null_mut(),
            palette: std::ptr::null_mut(),
            src_key: None,
        }))
    }

    fn px(&self, x: u32, y: u32) -> u32 {
        let o = (y * self.pitch) as usize + (x * self.bpp / 8) as usize;
        if self.bpp == 8 { self.mem[o] as u32 } else { u16::from_le_bytes([self.mem[o], self.mem[o + 1]]) as u32 }
    }

    fn put(&mut self, x: u32, y: u32, v: u32) {
        let o = (y * self.pitch) as usize + (x * self.bpp / 8) as usize;
        if self.bpp == 8 {
            self.mem[o] = v as u8;
        } else {
            self.mem[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes());
        }
    }

    /// A copy of the pixels, for a copy from a surface to itself.
    fn snapshot(&self) -> Surface {
        Surface { vtbl: std::ptr::null(), refs: 0, mem: self.mem.clone(), back: std::ptr::null_mut(), palette: std::ptr::null_mut(), src_key: self.src_key, ..*self }
    }

    fn desc(&self) -> Desc {
        Desc { w: self.w, h: self.h, pitch: self.pitch, bpp: self.bpp, caps: self.caps, surface: self.mem.as_ptr() as u32, back_count: 0 }
    }
}

/// A rectangle from a `RECT*`, or the whole surface.
unsafe fn rect(r: *const i32, s: &Surface) -> (i32, i32, i32, i32) {
    if r.is_null() {
        (0, 0, s.w as i32, s.h as i32)
    } else {
        unsafe { (r.read_unaligned(), r.add(1).read_unaligned(), r.add(2).read_unaligned(), r.add(3).read_unaligned()) }
    }
}

unsafe extern "system" fn query(this: *mut Surface, _iid: *const u8, out: *mut *mut Surface) -> Hr {
    // Versions 2 and 3 of the interface start with the same slots: the same object answers.
    unsafe {
        (*this).refs += 1;
        out.write(this);
    }
    DD_OK
}

unsafe extern "system" fn add_ref(this: *mut Surface) -> u32 {
    unsafe {
        (*this).refs += 1;
        (*this).refs
    }
}

unsafe extern "system" fn release(this: *mut Surface) -> u32 {
    // Surfaces live as long as the game: a released one may still be named by a flip chain.
    unsafe {
        (*this).refs = (*this).refs.saturating_sub(1);
        (*this).refs
    }
}

const DDBLT_COLORFILL: u32 = 0x400;
const DDBLT_KEYSRC: u32 = 0x8000;
const DDBLTFAST_SRCCOLORKEY: u32 = 0x1;

/// Copy `src` (rectangle `sr`) into `dst` (rectangle `dr`), stretching by nearest pixel, and
/// skipping source pixels equal to `key`.
fn copy(dst: &mut Surface, dr: (i32, i32, i32, i32), src: &Surface, sr: (i32, i32, i32, i32), key: Option<u32>) {
    let (dw, dh) = (dr.2 - dr.0, dr.3 - dr.1);
    let (sw, sh) = (sr.2 - sr.0, sr.3 - sr.1);
    if dw <= 0 || dh <= 0 || sw <= 0 || sh <= 0 {
        return;
    }
    for y in 0..dh {
        let (ty, sy) = (dr.1 + y, sr.1 + y * sh / dh);
        if ty < 0 || ty >= dst.h as i32 || sy < 0 || sy >= src.h as i32 {
            continue;
        }
        for x in 0..dw {
            let (tx, sx) = (dr.0 + x, sr.0 + x * sw / dw);
            if tx < 0 || tx >= dst.w as i32 || sx < 0 || sx >= src.w as i32 {
                continue;
            }
            let v = src.px(sx as u32, sy as u32);
            if key != Some(v) {
                dst.put(tx as u32, ty as u32, v);
            }
        }
    }
}

unsafe extern "system" fn blt(this: *mut Surface, dr: *const i32, src: *mut Surface, sr: *const i32, flags: u32, fx: *const u32) -> Hr {
    let tmp;
    let s: Option<&Surface> = if src.is_null() {
        None
    } else if src == this {
        tmp = unsafe { (*src).snapshot() };
        Some(&tmp)
    } else {
        Some(unsafe { &*src })
    };
    let dst = unsafe { &mut *this };
    let d = unsafe { rect(dr, dst) };
    if flags & DDBLT_COLORFILL != 0 {
        // DDBLTFX.dwFillColor is at offset 0x50.
        let color = if fx.is_null() { 0 } else { unsafe { fx.add(0x50 / 4).read_unaligned() } };
        for y in d.1.max(0)..d.3.min(dst.h as i32) {
            for x in d.0.max(0)..d.2.min(dst.w as i32) {
                dst.put(x as u32, y as u32, color);
            }
        }
    } else if let Some(s) = s {
        let key = if flags & DDBLT_KEYSRC != 0 { s.src_key } else { None };
        copy(dst, d, s, unsafe { rect(sr, s) }, key);
    }
    if dst.caps & PRIMARYSURFACE != 0 {
        crate::present::show(dst);
    }
    DD_OK
}

unsafe extern "system" fn blt_fast(this: *mut Surface, x: i32, y: i32, src: *mut Surface, sr: *const i32, trans: u32) -> Hr {
    if src.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    let tmp;
    let s: &Surface = if src == this {
        tmp = unsafe { (*src).snapshot() };
        &tmp
    } else {
        unsafe { &*src }
    };
    let dst = unsafe { &mut *this };
    let r = unsafe { rect(sr, s) };
    let key = if trans & DDBLTFAST_SRCCOLORKEY != 0 { s.src_key } else { None };
    copy(dst, (x, y, x + r.2 - r.0, y + r.3 - r.1), s, r, key);
    if dst.caps & PRIMARYSURFACE != 0 {
        crate::present::show(dst);
    }
    DD_OK
}

/// The back buffer's memory becomes the front's, as a flip swaps the buffers a surface
/// object names; then the new front is shown.
unsafe extern "system" fn flip(this: *mut Surface, _target: *mut Surface, _flags: u32) -> Hr {
    let front = unsafe { &mut *this };
    if front.back.is_null() {
        return DDERR_UNSUPPORTED;
    }
    let back = unsafe { &mut *front.back };
    std::mem::swap(&mut front.mem, &mut back.mem);
    crate::present::show(front);
    DD_OK
}

unsafe extern "system" fn get_attached(this: *mut Surface, caps: *const u32, out: *mut *mut Surface) -> Hr {
    let s = unsafe { &mut *this };
    let want = if caps.is_null() { 0 } else { unsafe { caps.read_unaligned() } };
    if want & BACKBUFFER != 0 && !s.back.is_null() {
        unsafe {
            (*s.back).refs += 1;
            out.write(s.back);
        }
        return DD_OK;
    }
    log!("GetAttachedSurface caps {want:#x}: none");
    DDERR_NOTFOUND
}

unsafe extern "system" fn get_caps(this: *mut Surface, caps: *mut u32) -> Hr {
    unsafe { caps.write_unaligned((*this).caps) };
    DD_OK
}

unsafe extern "system" fn get_pixel_format(this: *mut Surface, pf: *mut u32) -> Hr {
    unsafe { desc::write_pixel_format(pf, (*this).bpp) };
    DD_OK
}

unsafe extern "system" fn get_desc(this: *mut Surface, d: *mut u32) -> Hr {
    unsafe { desc::write(d, &(*this).desc()) };
    DD_OK
}

unsafe extern "system" fn is_lost(_this: *mut Surface) -> Hr {
    DD_OK
}

unsafe extern "system" fn restore(_this: *mut Surface) -> Hr {
    DD_OK
}

unsafe extern "system" fn set_clipper(_this: *mut Surface, _clipper: *mut c_void) -> Hr {
    DD_OK
}

const DDCKEY_SRCBLT: u32 = 0x8;

unsafe extern "system" fn set_color_key(this: *mut Surface, flags: u32, key: *const u32) -> Hr {
    if flags & DDCKEY_SRCBLT != 0 {
        unsafe { (*this).src_key = if key.is_null() { None } else { Some(key.read_unaligned()) } };
    }
    DD_OK
}

unsafe extern "system" fn set_palette(this: *mut Surface, p: *mut crate::palette::Palette) -> Hr {
    unsafe { (*this).palette = p };
    DD_OK
}

/// The pointer into the surface's memory, at the rectangle's corner if one is given.
unsafe extern "system" fn lock(this: *mut Surface, r: *const i32, d: *mut u32, _flags: u32, _event: u32) -> Hr {
    let s = unsafe { &*this };
    let mut info = s.desc();
    if !r.is_null() {
        let (x, y, _, _) = unsafe { rect(r, s) };
        info.surface += y as u32 * s.pitch + x as u32 * s.bpp / 8;
    }
    unsafe { desc::write(d, &info) };
    DD_OK
}

unsafe extern "system" fn unlock(this: *mut Surface, _ptr: *mut c_void) -> Hr {
    let s = unsafe { &*this };
    if s.caps & PRIMARYSURFACE != 0 {
        crate::present::show(s);
    }
    DD_OK
}
