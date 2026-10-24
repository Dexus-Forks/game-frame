//! The `gu` texture utilities: the texture memory manager (`guTexAllocateMemory`,
//! `guTexSource`, …), which hands out TMU memory and keeps a `GrMipMapInfo` per texture, and
//! 3DF files (`gu3dfGetInfo`, `gu3dfLoad`, read with `tdf`).

use super::tex::{GrTexInfo, TMU_MEMORY, download_chain, grTexClampMode, grTexFilterMode, grTexLodBiasValue, grTexMipMapMode, grTexNCCTable, grTexSource};
use crate::cmd::{Cmd, push};
use crate::layout;
use crate::tdf;
use gpu_1997::glide::{Format, size};
use std::ffi::{CStr, c_char};
use std::sync::Mutex;

/// `GuNccTable`: Y, I and Q, then the same in the TMU's register layout.
const NCC_BYTES: usize = 112;
const NULL_MIPMAP: i32 = -1;

/// `GrMipMapInfo` as the Glide 2.x SDK's `glide.h` lays it out (196 bytes). `lod_min` is the
/// smallest level (the larger `GR_LOD_*` number) and `lod_max` the largest.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMipMapInfo {
    sst: i32,
    valid: u32,
    width: i32,
    height: i32,
    aspect: u32,
    data: *const u8,
    format: u32,
    mipmap_mode: u32,
    mag_filter: u32,
    min_filter: u32,
    s_clamp: u32,
    t_clamp: u32,
    t_lod: u32,
    t_texture_mode: u32,
    /// The bias in 4.2 fixed point.
    lod_bias: u32,
    lod_min: u32,
    lod_max: u32,
    tmu: i32,
    odd_even: u32,
    base: u32,
    trilinear: u32,
    ncc: [u8; NCC_BYTES],
}

struct Manager {
    /// Boxed so that the pointers `guTexGetMipMapInfo` returns stay put.
    #[allow(clippy::vec_box)]
    maps: Vec<Box<GrMipMapInfo>>,
    /// Bytes given to each.
    sizes: Vec<u32>,
    next: u32,
    current: i32,
}

// The raw data pointer in GrMipMapInfo is the game's; the manager only stores it.
unsafe impl Send for Manager {}

static MANAGER: Mutex<Manager> = Mutex::new(Manager { maps: Vec::new(), sizes: Vec::new(), next: 0, current: NULL_MIPMAP });

fn with<R>(f: impl FnOnce(&mut Manager) -> R) -> R {
    f(&mut MANAGER.lock().unwrap_or_else(|e| e.into_inner()))
}

fn map(m: &Manager, id: i32) -> Option<GrMipMapInfo> {
    m.maps.get(usize::try_from(id).ok()?).map(|b| **b).filter(|i| i.valid != 0)
}

fn tex_info(i: &GrMipMapInfo) -> GrTexInfo {
    GrTexInfo { small_lod: i.lod_min, large_lod: i.lod_max, aspect: i.aspect, format: i.format, data: i.data }
}

/// Set the attributes both `guTexAllocateMemory` and `guTexChangeAttributes` take.
fn attributes(i: &mut GrMipMapInfo, a: [u32; 11]) {
    let [width, height, format, mode, small, large, aspect, s_clamp, t_clamp, min_filter, mag_filter] = a;
    (i.width, i.height, i.format, i.mipmap_mode) = (width as i32, height as i32, format, mode);
    (i.lod_min, i.lod_max, i.aspect) = (small, large, aspect);
    (i.s_clamp, i.t_clamp, i.min_filter, i.mag_filter) = (s_clamp, t_clamp, min_filter, mag_filter);
}

fn required(a: &[u32; 11]) -> u32 {
    Format::from_u32(a[2]).map_or(0, |f| layout::mem_required(a[4], a[5], a[6], f))
}

/// Room for one texture, at the next free address. Returns its id, or
/// `GR_NULL_MIPMAP_HANDLE` (-1) when TMU memory is full.
#[unsafe(no_mangle)]
pub extern "system" fn guTexAllocateMemory(
    tmu: i32, odd_even: u32, width: u32, height: u32, format: u32, mode: u32, small: u32, large: u32, aspect: u32,
    s_clamp: u32, t_clamp: u32, min_filter: u32, mag_filter: u32, lod_bias: f32, trilinear: u32,
) -> i32 {
    let a = [width, height, format, mode, small, large, aspect, s_clamp, t_clamp, min_filter, mag_filter];
    let need = required(&a);
    with(|m| {
        if need == 0 || m.next + need > TMU_MEMORY {
            return NULL_MIPMAP;
        }
        let mut i = GrMipMapInfo {
            sst: 0,
            valid: 1,
            width: 0,
            height: 0,
            aspect: 0,
            data: std::ptr::null(),
            format: 0,
            mipmap_mode: 0,
            mag_filter: 0,
            min_filter: 0,
            s_clamp: 0,
            t_clamp: 0,
            t_lod: 0,
            t_texture_mode: 0,
            lod_bias: (lod_bias * 4.0) as i32 as u32,
            lod_min: 0,
            lod_max: 0,
            tmu,
            odd_even,
            base: m.next,
            trilinear: (trilinear != 0) as u32,
            ncc: [0; NCC_BYTES],
        };
        attributes(&mut i, a);
        m.maps.push(Box::new(i));
        m.sizes.push(need);
        m.next += need;
        m.maps.len() as i32 - 1
    })
}

/// New attributes for a texture; fails if it would need more memory than it was given.
#[unsafe(no_mangle)]
pub extern "system" fn guTexChangeAttributes(
    id: i32, width: u32, height: u32, format: u32, mode: u32, small: u32, large: u32, aspect: u32, s_clamp: u32, t_clamp: u32,
    min_filter: u32, mag_filter: u32,
) -> u32 {
    let a = [width, height, format, mode, small, large, aspect, s_clamp, t_clamp, min_filter, mag_filter];
    with(|m| {
        let Some(k) = usize::try_from(id).ok().filter(|k| map(m, id).is_some() && required(&a) <= m.sizes[*k]) else {
            return 0;
        };
        attributes(&mut m.maps[k], a);
        1
    })
}

/// The whole chain from `src`, and the NCC table for an NCC format.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guTexDownloadMipMap(id: i32, src: *const u8, ncc: *const u8) {
    let Some(i) = with(|m| {
        let k = usize::try_from(id).ok()?;
        map(m, id)?;
        let i = &mut m.maps[k];
        i.data = src;
        if !ncc.is_null() {
            unsafe { std::ptr::copy_nonoverlapping(ncc, i.ncc.as_mut_ptr(), NCC_BYTES) };
        }
        Some(**i)
    }) else {
        return;
    };
    unsafe { download_chain(i.base, i.lod_min, i.lod_max, i.aspect, i.format, src) };
}

/// One level from `*src`, which then points past it.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guTexDownloadMipMapLevel(id: i32, lod: u32, src: *mut *const u8) {
    let Some(i) = with(|m| map(m, id)) else { return };
    let (Some(f), Some((w, h))) = (Format::from_u32(i.format), size(lod, i.aspect)) else { return };
    if src.is_null() || lod < i.lod_max || lod > i.lod_min {
        return;
    }
    unsafe {
        let p = *src;
        super::tex::grTexDownloadMipMapLevelPartial(0, i.base, lod, i.lod_max, i.aspect, i.format, 3, p, 0, h as i32 - 1);
        *src = p.add(f.texel().bytes(w, h));
    }
}

/// Draw with a texture: its address and levels, clamping, filters, mipmap mode and bias, and
/// its NCC table (loaded as table 0).
#[unsafe(no_mangle)]
pub extern "system" fn guTexSource(id: i32) {
    let Some(i) = with(|m| {
        let i = map(m, id)?;
        m.current = id;
        Some(i)
    }) else {
        return;
    };
    let info = tex_info(&i);
    unsafe { grTexSource(0, i.base, 3, &info) };
    grTexClampMode(0, i.s_clamp, i.t_clamp);
    grTexFilterMode(0, i.min_filter, i.mag_filter);
    grTexMipMapMode(0, i.mipmap_mode, i.trilinear);
    grTexLodBiasValue(0, i.lod_bias as i32 as f32 / 4.0);
    if matches!(Format::from_u32(i.format), Some(Format::Yiq422 | Format::Ayiq8422)) {
        push(Cmd::Ncc(0, Box::new(i.ncc)));
        grTexNCCTable(0, 0);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn guTexGetCurrentMipMap(_tmu: u32) -> i32 {
    with(|m| m.current)
}

#[unsafe(no_mangle)]
pub extern "system" fn guTexGetMipMapInfo(id: i32) -> *const GrMipMapInfo {
    with(|m| match usize::try_from(id).ok().and_then(|k| m.maps.get(k)) {
        Some(b) => &**b as *const GrMipMapInfo,
        None => std::ptr::null(),
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn guTexMemQueryAvail(_tmu: u32) -> u32 {
    with(|m| TMU_MEMORY - m.next)
}

/// Forget every texture. Deviation: the `GrMipMapInfo`s handed out stay readable (marked not
/// valid) instead of being freed, in case the game still holds one.
#[unsafe(no_mangle)]
pub extern "system" fn guTexMemReset() {
    with(|m| {
        for i in m.maps.iter_mut() {
            i.valid = 0;
        }
        std::mem::forget(std::mem::take(&mut m.maps));
        m.sizes.clear();
        m.next = 0;
        m.current = NULL_MIPMAP;
    });
}

/// A 256 × 256 RGB565 texture with each level in its own flat colour, to see which level
/// draws where. Deviation: the colours are this library's own.
#[unsafe(no_mangle)]
pub extern "system" fn guTexCreateColorMipMap() -> i32 {
    const COLOURS: [u16; 9] = [0xf800, 0x07e0, 0x001f, 0xffe0, 0xf81f, 0x07ff, 0xffff, 0x8410, 0x0000];
    let id = guTexAllocateMemory(0, 3, 256, 256, Format::Rgb565 as u32, 1, 8, 0, 3, 0, 0, 1, 1, 0.0, 0);
    if id == NULL_MIPMAP {
        return id;
    }
    let data: Vec<u8> = (0..=8u32).flat_map(|l| {
        let side = 256usize >> l;
        std::iter::repeat_n(COLOURS[l as usize].to_le_bytes(), side * side).flatten()
    }).collect();
    unsafe { guTexDownloadMipMap(id, data.as_ptr(), std::ptr::null()) };
    std::mem::forget(data); // the GrMipMapInfo points at it
    id
}

/// `Gu3dfInfo`: the header (width, height, smallLod, largeLod, aspect, format), the table (a
/// palette or an NCC table, 1024 bytes), the data pointer the caller fills in, and the memory
/// the texture needs.
#[repr(C)]
pub struct Gu3dfInfo {
    header: [u32; 6],
    table: [u8; 1024],
    data: *mut u8,
    mem_required: u32,
}

unsafe fn read_header(name: *const c_char) -> Option<(Vec<u8>, tdf::Header)> {
    if name.is_null() {
        return None;
    }
    let path = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let file = std::fs::read(&path).ok()?;
    let h = tdf::header(&file)?;
    Some((file, h))
}

/// Read a 3DF file's header into `info`: the size, levels, aspect and format, and the memory
/// to allocate for its data.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn gu3dfGetInfo(name: *const c_char, info: *mut Gu3dfInfo) -> u32 {
    let (Some((_, h)), Some(info)) = (unsafe { read_header(name) }, unsafe { info.as_mut() }) else { return 0 };
    info.header = [h.width, h.height, h.small_lod, h.large_lod, h.aspect, h.format as u32];
    info.mem_required = layout::mem_required(h.small_lod, h.large_lod, h.aspect, h.format);
    1
}

/// Read a 3DF file's table into `info` and its texels into `info.data`, which the caller has
/// allocated with `mem_required` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn gu3dfLoad(name: *const c_char, info: *mut Gu3dfInfo) -> u32 {
    let (Some((file, h)), Some(info)) = (unsafe { read_header(name) }, unsafe { info.as_mut() }) else { return 0 };
    let Some((table, data)) = tdf::load(&file, &h) else { return 0 };
    if info.data.is_null() {
        return 0;
    }
    info.header = [h.width, h.height, h.small_lod, h.large_lod, h.aspect, h.format as u32];
    match table {
        tdf::Table::Palette(p) => {
            for (k, w) in p.iter().enumerate() {
                info.table[4 * k..4 * k + 4].copy_from_slice(&w.to_le_bytes());
            }
        }
        tdf::Table::Ncc(b) => info.table[..b.len()].copy_from_slice(&b),
        tdf::Table::None => {}
    }
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), info.data, data.len()) };
    1
}
