//! COM plumbing: interface tables are arrays of `__stdcall` function pointers, and every
//! method removes its own arguments from the stack. So each method the game never calls is a
//! stub with the right number of arguments, returning `DDERR_UNSUPPORTED`, and logged by name
//! (`IDirectDrawSurface::GetDC`) and counted with the other stubs (`shim::log`).

use std::ffi::c_void;

pub type Hr = u32;
pub const DD_OK: Hr = 0;
pub const DDERR_UNSUPPORTED: Hr = 0x8000_4001;
pub const DDERR_INVALIDPARAMS: Hr = 0x8007_0057;
pub const DDERR_NOTFOUND: Hr = 0x8876_00ff;

/// One slot of an interface table.
pub type Slot = *const c_void;

/// One stub method: logs `Interface::Method` with its arguments (`shim::log::unhandled`) and
/// returns `DDERR_UNSUPPORTED`, removing its arguments as COM's `__stdcall` does.
macro_rules! stub_method {
    ($name:expr, $m:ident, 0) => {
        pub unsafe extern "system" fn $m(_this: *mut c_void) -> Hr {
            shim::log::unhandled($name, &[]);
            DDERR_UNSUPPORTED
        }
    };
    ($name:expr, $m:ident, $n:tt, $($a:ident),+) => {
        pub unsafe extern "system" fn $m(_this: *mut c_void, $($a: u32),+) -> Hr {
            shim::log::unhandled($name, &[$($a),+]);
            DDERR_UNSUPPORTED
        }
    };
}

/// An interface's methods in table order, each with its argument count (not counting
/// `this`). `stubs()` gives a table of stubs, one per method, each named in the log.
macro_rules! interface {
    ($module:ident, $iface:literal: $($m:ident $n:tt),* $(,)?) => {
        #[allow(non_snake_case)]
        pub mod $module {
            use super::*;
            $( interface!(@one $iface, $m, $n); )*
            /// A stub for every method, in table order.
            pub fn stubs() -> Vec<Slot> {
                vec![$($m as Slot),*]
            }
        }
    };
    (@one $iface:literal, $m:ident, 0) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 0); };
    (@one $iface:literal, $m:ident, 1) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 1, a); };
    (@one $iface:literal, $m:ident, 2) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 2, a, b); };
    (@one $iface:literal, $m:ident, 3) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 3, a, b, c); };
    (@one $iface:literal, $m:ident, 4) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 4, a, b, c, d); };
    (@one $iface:literal, $m:ident, 5) => { stub_method!(concat!($iface, "::", stringify!($m)), $m, 5, a, b, c, d, e); };
}

// The methods of each interface (ddraw.h). IDirectDraw2 is IDirectDraw with SetDisplayMode
// taking 5 arguments and GetAvailableVidMem added; both are implemented, so one list serves.
interface!(direct_draw, "IDirectDraw":
    QueryInterface 2, AddRef 0, Release 0, Compact 0, CreateClipper 3, CreatePalette 4,
    CreateSurface 3, DuplicateSurface 2, EnumDisplayModes 4, EnumSurfaces 4, FlipToGDISurface 0,
    GetCaps 2, GetDisplayMode 1, GetFourCCCodes 2, GetGDISurface 1, GetMonitorFrequency 1,
    GetScanLine 1, GetVerticalBlankStatus 1, Initialize 1, RestoreDisplayMode 0,
    SetCooperativeLevel 2, SetDisplayMode 3, WaitForVerticalBlank 2, GetAvailableVidMem 3);
interface!(surface, "IDirectDrawSurface":
    QueryInterface 2, AddRef 0, Release 0, AddAttachedSurface 1, AddOverlayDirtyRect 1, Blt 5,
    BltBatch 3, BltFast 5, DeleteAttachedSurface 2, EnumAttachedSurfaces 2, EnumOverlayZOrders 3,
    Flip 2, GetAttachedSurface 2, GetBltStatus 1, GetCaps 1, GetClipper 1, GetColorKey 2, GetDC 1,
    GetFlipStatus 1, GetOverlayPosition 2, GetPalette 1, GetPixelFormat 1, GetSurfaceDesc 1,
    Initialize 2, IsLost 0, Lock 4, ReleaseDC 1, Restore 0, SetClipper 1, SetColorKey 2,
    SetOverlayPosition 2, SetPalette 1, Unlock 1, UpdateOverlay 5, UpdateOverlayDisplay 1,
    UpdateOverlayZOrder 2, GetDDInterface 1, PageLock 1, PageUnlock 1, SetSurfaceDesc 2);
interface!(palette, "IDirectDrawPalette":
    QueryInterface 2, AddRef 0, Release 0, GetCaps 1, GetEntries 4, Initialize 3, SetEntries 4);
interface!(clipper, "IDirectDrawClipper":
    QueryInterface 2, AddRef 0, Release 0, GetClipList 3, GetHWnd 1, Initialize 2,
    IsClipListChanged 1, SetClipList 2, SetHWnd 2);

/// An interface's stubs with the implemented methods put in.
pub fn table(stubs: Vec<Slot>, methods: &[(usize, Slot)]) -> Vec<Slot> {
    let mut t = stubs;
    for (i, m) in methods {
        t[*i] = *m;
    }
    t
}
