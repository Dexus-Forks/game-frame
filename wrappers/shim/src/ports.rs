//! Port I/O from the game, as Windows 95 allowed it. A game of the time may probe the CPU
//! through ports. POD's `WPOD3DFX.EXE`, for one, checks for a Cyrix CPU at start (`FUN_004897d8`, called from `FUN_004ab87c`) by writing and reading the Cyrix
//! configuration registers through ports 0x22 and 0x23, between `cli` and `sti`. Windows 95
//! let a program do that; Windows NT and Wine stop it with a privileged-instruction fault.
//! A vectored exception handler, installed when this library loads (before the game's code
//! runs), steps over `cli`, `sti` and `out`, and makes `in` read 0xff: what a machine without a
//! Cyrix chip answers on those ports (*inferred*: no device decodes them), so the game takes
//! its non-Cyrix path.

use std::ffi::c_void;

const EXCEPTION_PRIV_INSTRUCTION: u32 = 0xc000_0096;
const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
// i386 CONTEXT offsets.
const EAX: usize = 0xb0;
const EIP: usize = 0xb8;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn AddVectoredExceptionHandler(first: u32, f: unsafe extern "system" fn(*mut [*mut u8; 2]) -> i32) -> *mut c_void;
}

unsafe extern "system" fn handler(info: *mut [*mut u8; 2]) -> i32 {
    unsafe {
        let [record, context] = *info;
        if (record as *const u32).read() != EXCEPTION_PRIV_INSTRUCTION {
            return EXCEPTION_CONTINUE_SEARCH;
        }
        let eip = (context.add(EIP) as *mut u32).read_unaligned();
        let op = (eip as *const u8).read();
        let eax = context.add(EAX) as *mut u32;
        let len = match op {
            0xfa | 0xfb => 1,              // cli, sti
            0xe6 | 0xe7 => 2,              // out imm8, al / eax
            0xee | 0xef => 1,              // out dx, al / eax
            0xe4 | 0xec => {               // in al, imm8 / dx
                eax.write_unaligned(eax.read_unaligned() | 0xff);
                if op == 0xe4 { 2 } else { 1 }
            }
            0xe5 | 0xed => {               // in eax, imm8 / dx
                eax.write_unaligned(0xffff_ffff);
                if op == 0xe5 { 2 } else { 1 }
            }
            _ => return EXCEPTION_CONTINUE_SEARCH,
        };
        crate::log_once!("emulating port I/O (first at {eip:#x}): reads give 0xff");
        (context.add(EIP) as *mut u32).write_unaligned(eip + len);
        EXCEPTION_CONTINUE_EXECUTION
    }
}

pub fn install() {
    unsafe { AddVectoredExceptionHandler(1, handler) };
}
