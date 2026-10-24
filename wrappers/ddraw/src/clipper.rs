//! `IDirectDrawClipper`: the windowed path asks for one and names its window; the frame is
//! shown in the window whatever the clipper says, so it only answers.

use crate::com::*;
use std::sync::OnceLock;

#[repr(C)]
pub struct Clipper {
    vtbl: *const Slot,
    refs: u32,
}

static VTBL: OnceLock<Vec<usize>> = OnceLock::new();

/// QueryInterface, AddRef, Release, GetClipList, GetHWnd, Initialize, IsClipListChanged,
/// SetClipList, SetHWnd.
const ARGS: [usize; 9] = [2, 0, 0, 3, 1, 2, 1, 2, 2];

unsafe extern "system" fn ok2(_this: *mut Clipper, _a: u32, _b: u32) -> Hr {
    DD_OK
}

unsafe extern "system" fn count(_this: *mut Clipper) -> u32 {
    1
}

pub fn new() -> *mut Clipper {
    let vtbl = VTBL.get_or_init(|| {
        table(&ARGS, &[(1, count as Slot), (2, count as Slot), (7, ok2 as Slot), (8, ok2 as Slot)])
            .into_iter()
            .map(|p| p as usize)
            .collect()
    });
    Box::into_raw(Box::new(Clipper { vtbl: vtbl.as_ptr() as *const Slot, refs: 1 }))
}
