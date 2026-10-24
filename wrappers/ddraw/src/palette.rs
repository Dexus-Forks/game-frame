//! `IDirectDrawPalette`: 256 entries (`PALETTEENTRY`: red, green, blue, flags).

use crate::com::*;
use std::sync::OnceLock;

#[repr(C)]
pub struct Palette {
    vtbl: *const Slot,
    refs: u32,
    pub entries: [[u8; 4]; 256],
}

static VTBL: OnceLock<Vec<usize>> = OnceLock::new();

fn vtbl() -> *const Slot {
    VTBL.get_or_init(|| {
        table(palette::stubs(), &[(1, add_ref as Slot), (2, release as Slot), (4, get_entries as Slot), (6, set_entries as Slot)])
            .into_iter()
            .map(|p| p as usize)
            .collect()
    })
    .as_ptr() as *const Slot
}

impl Palette {
    pub unsafe fn new(entries: *const [u8; 4]) -> *mut Palette {
        let mut p = Palette { vtbl: vtbl(), refs: 1, entries: [[0; 4]; 256] };
        if !entries.is_null() {
            unsafe { std::ptr::copy_nonoverlapping(entries, p.entries.as_mut_ptr(), 256) };
        }
        Box::into_raw(Box::new(p))
    }
}

unsafe extern "system" fn add_ref(this: *mut Palette) -> u32 {
    unsafe {
        (*this).refs += 1;
        (*this).refs
    }
}

unsafe extern "system" fn release(this: *mut Palette) -> u32 {
    unsafe {
        (*this).refs = (*this).refs.saturating_sub(1);
        (*this).refs
    }
}

unsafe extern "system" fn get_entries(this: *mut Palette, _flags: u32, start: u32, n: u32, out: *mut [u8; 4]) -> Hr {
    let p = unsafe { &*this };
    for i in 0..n.min(256u32.saturating_sub(start)) {
        unsafe { out.add(i as usize).write(p.entries[(start + i) as usize]) };
    }
    DD_OK
}

unsafe extern "system" fn set_entries(this: *mut Palette, _flags: u32, start: u32, n: u32, src: *const [u8; 4]) -> Hr {
    let p = unsafe { &mut *this };
    for i in 0..n.min(256u32.saturating_sub(start)) {
        p.entries[(start + i) as usize] = unsafe { src.add(i as usize).read() };
    }
    crate::present::palette_changed();
    DD_OK
}
