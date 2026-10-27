//! Glide's PCI library (`fxpci`): finding the card, its configuration space, mapping its
//! memory, and the CPU's memory type registers (MTRRs). There is no card here. The library
//! shows one Voodoo Graphics (vendor 0x121a, device 0x0001) as device number 0.
//!
//! What the original checks is copied (*measured*, the Win95 `glide2x.dll` of Glide 2.43):
//! `pciDeviceExists`, `pciGetConfigData` and `pciSetConfigData` fail until `pciOpen` has
//! run, and fail for a device number above 0x200 or one with no card. A read fails for a
//! register marked write-only, a write for one marked read-only. `pciFindCard`,
//! `pciFindCardMulti` and the map calls open the library themselves. `pciMapCard` is
//! `pciMapCardMulti` with card 0.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const VENDOR: u32 = 0x121a;
const DEVICE: u32 = 0x0001;
/// `PciIOFlag`: `READ_ONLY` 0, `WRITE_ONLY` 1, `READ_WRITE` 2.
const READ_ONLY: u32 = 0;
const WRITE_ONLY: u32 = 1;
/// The card's memory: 16 MB, as its base address register asks for.
const CARD_BYTES: usize = 16 << 20;

static OPEN: AtomicBool = AtomicBool::new(false);

/// The card's 256 bytes of configuration space, as a Voodoo Graphics reports them after
/// reset. *Inferred* from the PCI specification and the card's usual listing: class 0x04
/// (multimedia) subclass 0x00 (video), revision 2, 16 MB of prefetchable memory at BAR 0.
/// Deviation: the base address 0xfb000000 is this library's choice; a real PC's BIOS picks it.
static CONFIG: Mutex<[u8; 256]> = Mutex::new(config());

const fn config() -> [u8; 256] {
    let mut c = [0u8; 256];
    let words: [(usize, u32); 6] = [
        (0x00, VENDOR | DEVICE << 16),
        (0x04, 0x0000_0002),  // command: memory space on
        (0x08, 0x0400_0002),  // class 04, subclass 00, interface 00; revision 2
        (0x0c, 0x0000_0000),
        (0x10, 0xfb00_0008),  // BAR 0: memory, prefetchable
        (0x3c, 0x0000_010b),  // interrupt pin A, line 11
    ];
    let mut i = 0;
    while i < words.len() {
        let (at, w) = words[i];
        c[at] = w as u8;
        c[at + 1] = (w >> 8) as u8;
        c[at + 2] = (w >> 16) as u8;
        c[at + 3] = (w >> 24) as u8;
        i += 1;
    }
    c
}

/// The block every map call returns. Deviation: writes there change nothing; the card's
/// registers and frame buffer are this library's calls.
struct Block(*mut u8);
unsafe impl Send for Block {}
static BLOCK: Mutex<Option<Block>> = Mutex::new(None);

fn block() -> *mut u8 {
    let mut b = BLOCK.lock().unwrap_or_else(|e| e.into_inner());
    b.get_or_insert_with(|| Block(Box::leak(vec![0u8; CARD_BYTES].into_boxed_slice()).as_mut_ptr())).0
}

/// Open, and the one device. The original's other device numbers up to 0x200 hold no card.
fn usable(dev: u32) -> bool {
    OPEN.load(Ordering::Relaxed) && dev == 0
}

/// `PciRegister`'s size in bytes: 1, 2 or 4. Bytes past the end of the space read as zero.
fn span(at: u32, size: u32) -> std::ops::Range<usize> {
    let at = (at as usize).min(256);
    at..(at + size.min(4) as usize).min(256)
}

#[unsafe(no_mangle)]
pub extern "system" fn pciOpen() -> u32 {
    shim::call!("pciOpen");
    OPEN.store(true, Ordering::Relaxed);
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn pciClose() -> u32 {
    shim::call!("pciClose");
    OPEN.store(false, Ordering::Relaxed);
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn pciDeviceExists(dev: u32) -> u32 {
    shim::call!("pciDeviceExists", dev);
    usable(dev) as u32
}

/// Read 1, 2 or 4 bytes of the configuration space into `*data`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciGetConfigData(at: u32, size: u32, rw: u32, dev: u32, data: *mut u32) -> u32 {
    shim::call!("pciGetConfigData", at, size, rw, dev, data);
    if !usable(dev) || rw == WRITE_ONLY || data.is_null() {
        return 0;
    }
    let c = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    let v = c[span(at, size)].iter().rev().fold(0u32, |v, b| v << 8 | *b as u32);
    unsafe { data.write_unaligned(v) };
    1
}

/// Write 1, 2 or 4 bytes of `*data` to the configuration space. They are stored, and read
/// back as written. Deviation: a real card ignores writes to its read-only registers.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciSetConfigData(at: u32, size: u32, rw: u32, dev: u32, data: *const u32) -> u32 {
    shim::call!("pciSetConfigData", at, size, rw, dev, data);
    if !usable(dev) || rw == READ_ONLY || data.is_null() {
        return 0;
    }
    let v = unsafe { data.read_unaligned() };
    let mut c = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    for (k, i) in span(at, size).enumerate() {
        c[i] = (v >> (8 * k)) as u8;
    }
    1
}

/// Card `card` (counting from 0) with these IDs. Only the one Voodoo Graphics is there.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciFindCardMulti(vendor: u32, device: u32, dev: *mut u32, card: u32) -> u32 {
    shim::call!("pciFindCardMulti", vendor, device, dev, card);
    pciOpen();
    if vendor != VENDOR || device != DEVICE || card != 0 {
        return 0;
    }
    if let Some(d) = unsafe { dev.as_mut() } {
        *d = 0;
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciFindCard(vendor: u32, device: u32, dev: *mut u32) -> u32 {
    shim::call!("pciFindCard", vendor, device, dev);
    unsafe { pciFindCardMulti(vendor, device, dev, 0) }
}

/// The card's memory at base address register `bar`, as a pointer: the library's own
/// zeroed 16 MB block. Null when the card is not found.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciMapCardMulti(vendor: u32, device: u32, _length: i32, dev: *mut u32, card: u32, _bar: u32) -> *mut u8 {
    shim::call!("pciMapCardMulti", vendor, device, _length, dev, card, _bar);
    if unsafe { pciFindCardMulti(vendor, device, dev, card) } == 0 {
        return std::ptr::null_mut();
    }
    block()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciMapCard(vendor: u32, device: u32, length: i32, dev: *mut u32, bar: u32) -> *mut u8 {
    shim::call!("pciMapCard", vendor, device, length, dev, bar);
    unsafe { pciMapCardMulti(vendor, device, length, dev, 0, bar) }
}

/// Any physical address maps to the same block. Deviation: it is not the memory at that
/// address. `*length` is left as asked, and capped at the block's size.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciMapPhysicalToLinear(linear: *mut u32, _physical: u32, length: *mut u32) -> u32 {
    shim::call!("pciMapPhysicalToLinear", linear, _physical, length);
    let Some(linear) = (unsafe { linear.as_mut() }) else { return 0 };
    pciOpen();
    *linear = block() as usize as u32;
    if let Some(l) = unsafe { length.as_mut() } {
        *l = (*l).min(CARD_BYTES as u32);
    }
    1
}

/// The block stays: every map shares it.
#[unsafe(no_mangle)]
pub extern "system" fn pciUnmapPhysical(_linear: u32, _length: u32) {
    shim::call!("pciUnmapPhysical", _linear, _length);
}

/// The MTRR calls succeed and change nothing: Windows owns the CPU's memory types. A free
/// register is number 0.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciFindFreeMTRR(mtrr: *mut u32) -> u32 {
    shim::call!("pciFindFreeMTRR", mtrr);
    if let Some(m) = unsafe { mtrr.as_mut() } {
        *m = 0;
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn pciFindMTRRMatch(_base: u32, _size: u32, _kind: u32, mtrr: *mut u32) -> u32 {
    shim::call!("pciFindMTRRMatch", _base, _size, _kind, mtrr);
    unsafe { pciFindFreeMTRR(mtrr) }
}

#[unsafe(no_mangle)]
pub extern "system" fn pciSetMTRR(_mtrr: u32, _base: u32, _size: u32, _kind: u32) -> u32 {
    shim::call!("pciSetMTRR", _mtrr, _base, _size, _kind);
    1
}
