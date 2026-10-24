//! The running EXE's import table, hooked: one imported function replaced by the wrapper's
//! own, which can call on to the original. A wrapper uses it to see calls the game makes to
//! another library (POD's Maxi Sound 64 build opens its wave streams through WINMM, and the
//! card's library is asked about them afterwards).
//!
//! Only calls through the EXE's import address table are seen: the game's own `call [iat]`
//! and its `jmp [iat]` thunks. Calls from other DLLs go to the original.

use crate::log;
use std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleA(name: *const u8) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
}

unsafe fn c_str_eq(p: *const u8, s: &str) -> bool {
    let b = s.as_bytes();
    let here = unsafe { std::slice::from_raw_parts(p, b.len()) };
    here.eq_ignore_ascii_case(b) && unsafe { p.add(b.len()).read() } == 0
}

/// Replace the EXE's import of `dll!name` with `new`; the original address is returned, or
/// `None` when the EXE does not import it by that name. Calling it again for an import that
/// already points at `new` returns `None` too, so a hook is never installed twice.
pub fn hook(dll: &str, name: &str, new: usize) -> Option<usize> {
    unsafe {
        let base = GetModuleHandleA(std::ptr::null()) as *const u8;
        let at = |rva: u32| base.add(rva as usize);
        let rd = |p: *const u8| (p as *const u32).read_unaligned();
        let pe = rd(base.add(0x3c));
        let dir = rd(at(pe + 24 + 104)); // optional header, data directory 1 (imports)
        if dir == 0 {
            return None;
        }
        let mut desc = at(dir);
        loop {
            let (lookup, dll_name, iat) = (rd(desc), rd(desc.add(12)), rd(desc.add(16)));
            if dll_name == 0 {
                return None;
            }
            if c_str_eq(at(dll_name), dll) {
                let names = if lookup != 0 { lookup } else { iat };
                for i in 0.. {
                    let entry = rd(at(names + 4 * i));
                    if entry == 0 {
                        break;
                    }
                    if entry & 0x8000_0000 == 0 && c_str_eq(at(entry + 2), name) {
                        let slot = at(iat + 4 * i) as *mut usize;
                        let old = slot.read();
                        if old == new {
                            return None;
                        }
                        let mut prot = 0;
                        VirtualProtect(slot.cast(), 4, 0x04, &mut prot); // PAGE_READWRITE
                        slot.write(new);
                        VirtualProtect(slot.cast(), 4, prot, &mut prot);
                        log!("hooked {dll}!{name}");
                        return Some(old);
                    }
                }
            }
            desc = desc.add(20);
        }
    }
}
