//! Texture calls. Texture memory is the game's to lay out (`grTexMinAddress`,
//! `grTexCalcMemRequired`); downloads are copied into the render thread's copy of it, and
//! `grTexSource` names an address and a `GrTexInfo` there. Only TMU 0 draws: POD uses one
//! (POD's `docs/hardware/3d-cards.md`).

use crate::cmd::{Cmd, TexInfo, push};
use crate::log::log_once;
use gpu_1997::glide::{Format, chain_bytes};

/// `GrTexInfo`: smallLod, largeLod, aspectRatio, format, data.
#[repr(C)]
pub struct GrTexInfo {
    pub small_lod: u32,
    pub large_lod: u32,
    pub aspect: u32,
    pub format: u32,
    pub data: *const u8,
}

impl GrTexInfo {
    fn info(&self) -> TexInfo {
        TexInfo { small_lod: self.small_lod, large_lod: self.large_lod, aspect: self.aspect, format: self.format }
    }
}

fn bytes(small_lod: u32, large_lod: u32, aspect: u32, format: u32) -> usize {
    Format::from_u32(format).and_then(|f| chain_bytes(small_lod, large_lod, aspect, f)).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexMinAddress(_tmu: u32) -> u32 {
    0
}

/// The chain's size, rounded up to 8 bytes as the Voodoo's texture addresses are.
#[unsafe(no_mangle)]
pub extern "system" fn grTexCalcMemRequired(small_lod: u32, large_lod: u32, aspect: u32, format: u32) -> u32 {
    (bytes(small_lod, large_lod, aspect, format) as u32 + 7) & !7
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexSource(tmu: u32, start: u32, _even_odd: u32, info: *const GrTexInfo) {
    if tmu != 0 || info.is_null() {
        return;
    }
    push(Cmd::TexSource { start, info: unsafe { &*info }.info() });
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadMipMap(tmu: u32, start: u32, even_odd: u32, info: *const GrTexInfo) {
    if tmu != 0 || info.is_null() {
        return;
    }
    let info = unsafe { &*info };
    if even_odd != 3 {
        log_once!("grTexDownloadMipMap: even/odd mask {even_odd} treated as both");
    }
    let n = bytes(info.small_lod, info.large_lod, info.aspect, info.format);
    if n == 0 || info.data.is_null() {
        return;
    }
    let data = unsafe { std::slice::from_raw_parts(info.data, n) }.to_vec();
    push(Cmd::Download { start, data });
}

/// Only `GR_TEXTABLE_PALETTE` (2): 256 words `0x00RRGGBB`. POD has no NCC textures.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadTable(tmu: u32, kind: u32, data: *const u32) {
    if tmu != 0 || data.is_null() {
        return;
    }
    if kind != gpu_1997::glide::TEXTABLE_PALETTE {
        log_once!("grTexDownloadTable: NCC tables are not handled");
        return;
    }
    let mut p = Box::new([0u32; 256]);
    unsafe { std::ptr::copy_nonoverlapping(data, p.as_mut_ptr(), 256) };
    push(Cmd::Palette(p));
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexFilterMode(tmu: u32, min: u32, mag: u32) {
    if tmu == 0 {
        push(Cmd::TexFilter(min, mag));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexClampMode(tmu: u32, s: u32, t: u32) {
    if tmu == 0 {
        push(Cmd::TexClamp(s, t));
    }
}

/// POD passes `GR_MIPMAP_DISABLE`; with one level per texture there is nothing to choose.
#[unsafe(no_mangle)]
pub extern "system" fn grTexMipMapMode(_tmu: u32, mode: u32, _lod_blend: u32) {
    if mode != 0 {
        log_once!("grTexMipMapMode {mode}: mipmaps are not handled");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexCombineFunction(tmu: u32, f: u32) {
    if tmu == 0 {
        push(Cmd::TexCombine(f));
    }
}

/// `guTexCombineFunction`: the same `GR_TEXTURECOMBINE_*` values as `grTexCombineFunction`.
#[unsafe(no_mangle)]
pub extern "system" fn guTexCombineFunction(tmu: u32, f: u32) {
    grTexCombineFunction(tmu, f);
}

/// The general texture combiner. With one TMU, `local` is the texel, `other` is 0 and the LOD
/// fraction is 0 (no mipmaps), so every setting comes out as the texel, zero or one: POD
/// Gold's `(SCALE_MINUS_LOCAL_ADD_LOCAL, LOD_FRACTION)` is the texel (decal).
#[unsafe(no_mangle)]
pub extern "system" fn grTexCombine(tmu: u32, rgb_func: u32, rgb_factor: u32, _alpha_func: u32, _alpha_factor: u32, rgb_invert: u32, _alpha_invert: u32) {
    if tmu != 0 {
        return;
    }
    // The factor's value: 0 or 1 for the constant ones; None when it depends on the texel.
    let factor = match rgb_factor {
        0x0 | 0x2 | 0x4 | 0x5 => Some(0.0),
        0x8 | 0xa | 0xc | 0xd => Some(1.0),
        _ => None,
    };
    // GR_TEXTURECOMBINE_ZERO 0, DECAL 1, ONE 10.
    let f = match (rgb_func, factor) {
        (0x0, _) | (0x3, _) => 0,
        (0x1, _) | (0x4, _) => 1,
        (0x6, Some(_)) => 0,
        (0x7 | 0x9, Some(k)) => if k == 0.0 { 1 } else { 0 },
        _ => {
            log_once!("grTexCombine({rgb_func}, {rgb_factor}): taken as decal");
            1
        }
    };
    let f = match (rgb_invert != 0, f) {
        (false, f) => f,
        (true, 0) => 10,
        (true, 10) => 0,
        (true, _) => {
            log_once!("grTexCombine: an inverted texel is taken as decal");
            1
        }
    };
    push(Cmd::TexCombine(f));
}
