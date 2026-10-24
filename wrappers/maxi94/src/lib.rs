//! `94DLL32.dll`: the Guillemot Maxi Sound 64's library ("Dream thunk for 9407 products",
//! Dream S.A. 1996: a 32-bit thunk down to `94DLL16.DLL` and the driver of the card's Dream
//! SAM9407 chip). POD Gold's two "64" builds (`PODMAX64.EXE`, `PODFXM64.EXE`) import six of
//! its calls by ordinal and send the card MIDI and sample banks. Under Wine there is no card and
//! no 16-bit driver behind the thunk, so this stand-in answers as a card that is present but
//! makes no sound: every call succeeds and does nothing.
//!
//! What POD expects (*observed*, `PODMAX64.EXE` `0x4b9bd0`–`0x4b9c96`):
//! - `mpuMessageAc` is retried up to 100 times until it returns non-zero;
//! - `GetMemMapAddress` gives a word pair the game passes to `UpLoad`, which copies `0xc0`
//!   bytes of the card's memory map into the game's table; the game then looks for entries
//!   marked `0x20` (free). Left empty, the table has no free entry (the game's answer is -1).
//! - the others' results are not looked at.
//!
//! All six are `__stdcall` (each ends `ret 0xc` or `ret 0x10` in the original DLL).

#![cfg(windows)]

use shim::log_once;
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        shim::log::init("maxi94", "MAXI94_LOG");
        shim::ports::install();
        shim::patch::apply();
    }
    1
}

/// A MIDI message to the card's MPU-401 side.
#[unsafe(no_mangle)]
pub extern "system" fn mpuMessage(status: u32, _data: u32, _count: u32) -> u32 {
    log_once!("mpuMessage {status:#x} (no card: dropped)");
    1
}

/// A message with an acknowledgement: non-zero means acknowledged.
#[unsafe(no_mangle)]
pub extern "system" fn mpuMessageAc(command: u32, _data: u32, _count: u32, _flags: u32) -> u32 {
    log_once!("mpuMessageAc {command:#x} (no card: acknowledged)");
    1
}

/// The card's memory map into `buffer`: left as the game set it (no free entries).
#[unsafe(no_mangle)]
pub extern "system" fn UpLoad(_buffer: *mut u8, _hi: u32, _lo: u32, size: u32) -> u32 {
    log_once!("UpLoad {size:#x} bytes (no card: nothing)");
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn GetMemMapAddress() -> u32 {
    log_once!("GetMemMapAddress (no card: 0)");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn waveSetAuxVolume(_a: u32, _b: u32, _c: u32) -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn waveSetMainVolume(_a: u32, _b: u32, _c: u32) -> u32 {
    1
}
