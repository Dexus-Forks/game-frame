//! COM plumbing: interface tables are arrays of `__stdcall` function pointers, and every
//! method removes its own arguments from the stack. So each method the game never calls is a
//! stub with the right number of arguments, returning `DDERR_UNSUPPORTED`, and logged once.

use std::ffi::c_void;

pub type Hr = u32;
pub const DD_OK: Hr = 0;
pub const DDERR_UNSUPPORTED: Hr = 0x8000_4001;
pub const DDERR_INVALIDPARAMS: Hr = 0x8007_0057;
pub const DDERR_NOTFOUND: Hr = 0x8876_00ff;

/// One slot of an interface table.
pub type Slot = *const c_void;

macro_rules! stub {
    ($name:ident, $n:literal, $($a:ident),*) => {
        pub unsafe extern "system" fn $name(_this: *mut c_void, $($a: u32),*) -> Hr {
            $(let _ = $a;)*
            shim::log_once!("unhandled DirectDraw call with {} arguments", $n);
            DDERR_UNSUPPORTED
        }
    };
}

pub unsafe extern "system" fn stub0(_this: *mut c_void) -> Hr {
    shim::log_once!("unhandled DirectDraw call with 0 arguments");
    DDERR_UNSUPPORTED
}
stub!(stub1, 1, a);
stub!(stub2, 2, a, b);
stub!(stub3, 3, a, b, c);
stub!(stub4, 4, a, b, c, d);
stub!(stub5, 5, a, b, c, d, e);

/// The stub for a method of `n` arguments (not counting `this`).
pub fn stub(n: usize) -> Slot {
    match n {
        0 => stub0 as Slot,
        1 => stub1 as Slot,
        2 => stub2 as Slot,
        3 => stub3 as Slot,
        4 => stub4 as Slot,
        _ => stub5 as Slot,
    }
}

/// A table from each method's argument count, with the implemented methods put in after.
pub fn table(counts: &[usize], methods: &[(usize, Slot)]) -> Vec<Slot> {
    let mut t: Vec<Slot> = counts.iter().map(|n| stub(*n)).collect();
    for (i, m) in methods {
        t[*i] = *m;
    }
    t
}
