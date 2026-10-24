//! `DriverProc`, the one export of a Video for Windows codec: the driver messages and the
//! decompression messages (`ICM_*`) Video for Windows sends it.

use crate::formats::{self, Header, Output};
use indeo::Decoder;
use shim::{gap, log, log_once};
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::OnceLock;

#[link(name = "winmm")]
unsafe extern "system" {
    fn DefDriverProc(id: usize, driver: usize, msg: u32, p1: isize, p2: isize) -> isize;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleFileNameA(module: *mut c_void, name: *mut u8, size: u32) -> u32;
}

const DRV_LOAD: u32 = 1;
const DRV_ENABLE: u32 = 2;
const DRV_OPEN: u32 = 3;
const DRV_CLOSE: u32 = 4;
const DRV_DISABLE: u32 = 5;
const DRV_FREE: u32 = 6;
const DRV_CONFIGURE: u32 = 7;
const DRV_QUERYCONFIGURE: u32 = 8;
const DRV_INSTALL: u32 = 9;
const DRV_REMOVE: u32 = 10;

const ICM_GETSTATE: u32 = 0x5000;
const ICM_SETSTATE: u32 = 0x5001;
const ICM_GETINFO: u32 = 0x5002;
const ICM_CONFIGURE: u32 = 0x500A;
const ICM_ABOUT: u32 = 0x500B;
const ICM_DECOMPRESS_GET_FORMAT: u32 = 0x400A;
const ICM_DECOMPRESS_QUERY: u32 = 0x400B;
const ICM_DECOMPRESS_BEGIN: u32 = 0x400C;
const ICM_DECOMPRESS: u32 = 0x400D;
const ICM_DECOMPRESS_END: u32 = 0x400E;

const ICERR_OK: isize = 0;
const ICERR_UNSUPPORTED: isize = -1;
const ICERR_BADFORMAT: isize = -2;
const ICERR_MEMORY: isize = -3;
const ICERR_INTERNAL: isize = -4;

const ICDECOMPRESS_HURRYUP: u32 = 0x8000_0000;

static MODULE: OnceLock<usize> = OnceLock::new();
static TABLES: OnceLock<Option<(PathBuf, indeo::Tables)>> = OnceLock::new();

/// Intel's tables, found once (`tables`).
fn tables() -> Option<&'static indeo::Tables> {
    TABLES
        .get_or_init(|| {
            let mut name = [0u8; 260];
            let module = *MODULE.get().unwrap_or(&0) as *mut c_void;
            let n = unsafe { GetModuleFileNameA(module, name.as_mut_ptr(), 260) } as usize;
            let path = PathBuf::from(String::from_utf8_lossy(&name[..n]).into_owned());
            let setting = std::env::var("WRAPPER_IR41_TABLES").ok();
            let found = crate::tables::load(&crate::tables::candidates(setting.as_deref(), path.parent()));
            match &found {
                Some((p, _)) => log!("Intel's tables from {}", p.display()),
                None => log!("no Intel IR41_32.DLL found for the tables: set WRAPPER_IR41_TABLES"),
            }
            found
        })
        .as_ref()
        .map(|(_, t)| t)
}

/// One open codec (`DRV_OPEN` returns its address as the driver id).
struct Instance {
    decoder: Option<Decoder>,
    output: Option<(Output, bool)>,
}

unsafe fn header(p: isize) -> Option<Header> {
    if p == 0 {
        return None;
    }
    // A BITMAPINFOHEADER and, for BI_BITFIELDS, its three masks.
    let bytes = unsafe { std::slice::from_raw_parts(p as *const u8, 52) };
    Header::parse(bytes)
}

unsafe fn query(input: isize, output: isize) -> Result<Option<(Output, bool)>, isize> {
    let Some(i) = (unsafe { header(input) }) else { return Err(ICERR_BADFORMAT) };
    if !i.is_indeo41() {
        return Err(ICERR_BADFORMAT);
    }
    let Some(o) = (unsafe { header(output) }) else { return Ok(None) };
    match formats::output(&i, &o) {
        Some(out) => Ok(Some((out, o.height > 0))),
        None => Err(ICERR_BADFORMAT),
    }
}

/// ICM_DECOMPRESS_GET_FORMAT: 24-bit RGB, bottom-up, the input's size.
unsafe fn get_format(input: isize, output: isize) -> isize {
    let Some(i) = (unsafe { header(input) }) else { return ICERR_BADFORMAT };
    if !i.is_indeo41() {
        return ICERR_BADFORMAT;
    }
    if output == 0 {
        return 40;
    }
    let (w, h) = (i.width, i.height.abs());
    let size = Output::Rgb24.size(w as usize, h as usize) as u32;
    let mut b = [0u8; 40];
    b[0..4].copy_from_slice(&40u32.to_le_bytes());
    b[4..8].copy_from_slice(&w.to_le_bytes());
    b[8..12].copy_from_slice(&h.to_le_bytes());
    b[12..14].copy_from_slice(&1u16.to_le_bytes());
    b[14..16].copy_from_slice(&24u16.to_le_bytes());
    b[20..24].copy_from_slice(&size.to_le_bytes());
    unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), output as *mut u8, 40) };
    ICERR_OK
}

unsafe fn decompress(inst: &mut Instance, icd: isize) -> isize {
    // ICDECOMPRESS: flags, the input header and data, the output header and data, the chunk id.
    let f = unsafe { std::slice::from_raw_parts(icd as *const usize, 6) };
    let (flags, bi_in, data, bi_out, out) = (f[0] as u32, f[1] as isize, f[2], f[3] as isize, f[4]);
    let Some(i) = (unsafe { header(bi_in) }) else { return ICERR_BADFORMAT };
    let len = unsafe { (bi_in as *const u32).add(5).read_unaligned() } as usize;
    if inst.decoder.is_none() {
        let Some(t) = tables() else { return ICERR_INTERNAL };
        inst.decoder = Some(Decoder::new(t.clone()));
    }
    let (output, bottom_up) = match inst.output {
        Some(o) => o,
        None => match unsafe { query(bi_in, bi_out) } {
            Ok(Some(o)) => o,
            _ => return ICERR_BADFORMAT,
        },
    };
    let dec = inst.decoder.as_mut().unwrap();
    if len > 0 && data != 0 {
        let chunk = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
        if let Err(e) = dec.decode(chunk) {
            log_once!("a picture did not decode: {e}");
        }
    }
    if flags & ICDECOMPRESS_HURRYUP != 0 || out == 0 {
        return ICERR_OK;
    }
    if let Some(frame) = dec.last() {
        let (w, h) = (i.width as usize, i.height.unsigned_abs() as usize);
        if (frame.width, frame.height) == (w, h) {
            let dst = unsafe { std::slice::from_raw_parts_mut(out as *mut u8, output.size(w, h)) };
            formats::write(frame, output, bottom_up, dst);
        }
    }
    ICERR_OK
}

unsafe fn info(p: isize, size: isize) -> isize {
    const LEN: usize = 568;
    if p == 0 || (size as usize) < LEN {
        return LEN as isize;
    }
    let mut b = vec![0u8; LEN];
    let put = |b: &mut [u8], o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    put(&mut b, 0, LEN as u32);
    put(&mut b, 4, formats::fourcc(b"VIDC"));
    put(&mut b, 8, formats::fourcc(b"IV41"));
    put(&mut b, 16, 0x0004_0000);
    put(&mut b, 20, 0x0104);
    let wide = |b: &mut [u8], o: usize, s: &str| {
        for (k, c) in s.encode_utf16().enumerate() {
            b[o + 2 * k..o + 2 * k + 2].copy_from_slice(&c.to_le_bytes());
        }
    };
    wide(&mut b, 24, "IV41");
    wide(&mut b, 56, "Indeo Video Interactive 4.1 (GameFrame, clean room)");
    wide(&mut b, 312, "ir41_32.dll");
    unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), p as *mut u8, LEN) };
    LEN as isize
}

/// The codec's entry point (`DriverProc@20`, stdcall).
#[unsafe(no_mangle)]
pub extern "system" fn DriverProc(id: usize, driver: usize, msg: u32, p1: isize, p2: isize) -> isize {
    match msg {
        DRV_LOAD | DRV_FREE | DRV_ENABLE | DRV_DISABLE | DRV_INSTALL | DRV_REMOVE => 1,
        DRV_QUERYCONFIGURE | DRV_CONFIGURE => 0,
        DRV_OPEN => {
            // ICOPEN: size, fccType, fccHandler, version, flags, error. Only video.
            if p2 != 0 {
                let open = unsafe { std::slice::from_raw_parts_mut(p2 as *mut u32, 6) };
                let kind = open[1].to_ascii_lowercase_u32();
                if kind != formats::fourcc(b"vidc") {
                    return 0;
                }
                open[5] = 0;
            }
            Box::into_raw(Box::new(Instance { decoder: None, output: None })) as isize
        }
        DRV_CLOSE => {
            if id != 0 && id != 1 {
                drop(unsafe { Box::from_raw(id as *mut Instance) });
            }
            1
        }
        _ if id == 0 || id == 1 => unsafe { DefDriverProc(id, driver, msg, p1, p2) },
        _ => {
            let inst = unsafe { &mut *(id as *mut Instance) };
            match msg {
                ICM_GETSTATE | ICM_SETSTATE => 0,
                ICM_GETINFO => unsafe { info(p1, p2) },
                ICM_CONFIGURE | ICM_ABOUT => ICERR_UNSUPPORTED,
                ICM_DECOMPRESS_QUERY => match unsafe { query(p1, p2) } {
                    Ok(_) => ICERR_OK,
                    Err(e) => e,
                },
                ICM_DECOMPRESS_GET_FORMAT => unsafe { get_format(p1, p2) },
                ICM_DECOMPRESS_BEGIN => match unsafe { query(p1, p2) } {
                    Ok(o) => {
                        let Some(t) = tables() else { return ICERR_MEMORY };
                        inst.decoder = Some(Decoder::new(t.clone()));
                        inst.output = o;
                        log!("decompress begin: {o:?}");
                        ICERR_OK
                    }
                    Err(e) => e,
                },
                ICM_DECOMPRESS => unsafe { decompress(inst, p1) },
                ICM_DECOMPRESS_END => {
                    inst.decoder = None;
                    ICERR_OK
                }
                _ => {
                    gap!("message", "message {msg:#x} not handled");
                    unsafe { DefDriverProc(id, driver, msg, p1, p2) }
                }
            }
        }
    }
}

trait Lower {
    fn to_ascii_lowercase_u32(self) -> u32;
}

impl Lower for u32 {
    fn to_ascii_lowercase_u32(self) -> u32 {
        u32::from_le_bytes(self.to_le_bytes().map(|b| b.to_ascii_lowercase()))
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == 1 {
        let _ = MODULE.set(module as usize);
        shim::log::init("ir41", "IR41_LOG");
    }
    if reason == 0 {
        shim::log::summary();
    }
    1
}
