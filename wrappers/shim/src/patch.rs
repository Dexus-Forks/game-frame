//! Patches to the game's code, applied in memory when a wrapper DLL loads (from its `DllMain`,
//! before the game's own code runs), so the EXE on disk stays unchanged. The patches come from
//! the file `WRAPPER_PATCHES` names (format: `config::parse_patches`); a game project keeps
//! that file in its own repository, as it holds bytes of the game's code. Each patch names the
//! build by its PE timestamp and checks the original bytes first; a different build, or bytes
//! that differ, leave the code alone and say so in the log.
//!
//! A typical use: a 1997 CPU check that knows only the CPUs of its day and quits on any other
//! (an emulator's CPU), patched to take a known kind.

use crate::config::{Patch, parse_patches, patch_file};
use crate::log;
use std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleA(name: *const u8) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
}

/// The running EXE's PE timestamp.
fn timestamp() -> u32 {
    unsafe {
        let base = GetModuleHandleA(std::ptr::null()) as *const u8;
        let pe = (base.add(0x3c) as *const u32).read_unaligned() as usize;
        (base.add(pe + 8) as *const u32).read_unaligned()
    }
}

fn load() -> Vec<Patch> {
    let Some(path) = patch_file() else { return Vec::new() };
    match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| parse_patches(&t)) {
        Ok(p) => p,
        Err(e) => {
            log!("patches {path} not read: {e}");
            Vec::new()
        }
    }
}

/// Apply every patch for the running build.
pub fn apply() {
    let stamp = timestamp();
    for p in load() {
        if p.stamp != stamp {
            continue;
        }
        let (name, va) = (&p.name, p.address);
        let at = va as *mut u8;
        let now = unsafe { std::slice::from_raw_parts(at, p.old.len()) };
        if now == p.new {
            continue; // another wrapper DLL in the process applied it already
        }
        if now != p.old {
            log!("patch {name} {va:#x} skipped: found {now:02x?}, expected {:02x?}", p.old);
            continue;
        }
        let mut prot = 0;
        unsafe {
            VirtualProtect(at.cast(), p.new.len(), 0x40, &mut prot); // PAGE_EXECUTE_READWRITE
            std::ptr::copy_nonoverlapping(p.new.as_ptr(), at, p.new.len());
            VirtualProtect(at.cast(), p.new.len(), prot, &mut prot);
        }
        log!("patch {name} {va:#x}: {}", p.why);
    }
}
