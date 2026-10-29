//! The six exported calls (cdecl: the caller removes the arguments; each original function
//! ends in a plain `ret`, wrappers/s3dtkw/api.tsv).

use shim::log;

/// `S3DTK_OK`; the game treats -1, -2 and -12 as failures (`0x4a648a`).
const OK: i32 = 0;

unsafe fn words(p: *const u32, n: usize) -> String {
    (0..n).map(|i| format!("{:#x}", unsafe { p.add(i).read_unaligned() })).collect::<Vec<_>>().join(" ")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn S3DTK_InitLib(init: *const u32) -> i32 {
    shim::call!("S3DTK_InitLib", init);
    if !init.is_null() {
        log!("S3DTK_InitLib {}", unsafe { words(init, 4) });
    }
    OK
}

/// Fills `*table` with the function list; the game checks for 1 (`0x4a64d1`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn S3DTK_CreateRenderer(init: *const u32, table: *mut *mut crate::table::Table) -> i32 {
    shim::call!("S3DTK_CreateRenderer", init, table);
    if !init.is_null() {
        log!("S3DTK_CreateRenderer {}", unsafe { words(init, 4) });
    }
    if !table.is_null() {
        unsafe { table.write(crate::table::new()) };
    }
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn S3DTK_DestroyRenderer(_table: *mut *mut crate::table::Table) -> i32 {
    shim::call!("S3DTK_DestroyRenderer", _table);
    1
}

/// The physical address of a linear one. This library draws through linear addresses, so the
/// "physical" address is the linear one: the game adds page offsets to it and gives it back
/// in `sfOffset` (`0x4a5fd6`).
#[unsafe(no_mangle)]
pub extern "C" fn S3DTK_LinearToPhysical(linear: u32) -> u32 {
    shim::call!("S3DTK_LinearToPhysical", linear);
    linear
}

/// The original returns 1 (its code sets the result to 1 before it frees its mappings).
#[unsafe(no_mangle)]
pub extern "C" fn S3DTK_ExitLib() -> i32 {
    shim::call!("S3DTK_ExitLib");
    1
}

/// The linear address of a physical one, for `size` bytes. The original maps it through DPMI
/// (function 0x800); here physical addresses are linear ones, as in `S3DTK_LinearToPhysical`.
#[unsafe(no_mangle)]
pub extern "C" fn S3DTK_PhysicalToLinear(physical: u32, _size: u32) -> u32 {
    shim::call!("S3DTK_PhysicalToLinear", physical, _size);
    physical
}
