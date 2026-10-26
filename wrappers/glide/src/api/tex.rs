//! Texture calls. Texture memory is the game's to lay out (`grTexMinAddress`,
//! `grTexCalcMemRequired`); downloads are copied into the render thread's copy of it, at the
//! level addresses `layout` gives, and `grTexSource` names an address and a `GrTexInfo` there.
//! Only TMU 0 draws: `grSstQueryHardware` reports one TMU, as POD uses (POD's
//! `docs/hardware/3d-cards.md`).

use super::shadow::{self, send};
use crate::cmd::{Cmd, TexInfo, push};
use crate::layout;
use voodoo::formats::{Format, TEXTABLE_PALETTE, size};

/// The TMU's memory as `grSstQueryHardware` reports it: 4 MB.
pub const TMU_MEMORY: u32 = 4 << 20;

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
    pub fn info(&self) -> TexInfo {
        TexInfo { small_lod: self.small_lod, large_lod: self.large_lod, aspect: self.aspect, format: self.format }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexMinAddress(_tmu: u32) -> u32 {
    0
}

/// The highest address a texture may start at: the memory size less 8 (*measured*,
/// `grTexMaxAddress` in the Win95 `glide2x.dll`).
#[unsafe(no_mangle)]
pub extern "system" fn grTexMaxAddress(_tmu: u32) -> u32 {
    TMU_MEMORY - 8
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexCalcMemRequired(small_lod: u32, large_lod: u32, aspect: u32, format: u32) -> u32 {
    Format::from_u32(format).map_or(0, |f| layout::mem_required(small_lod, large_lod, aspect, f))
}

/// The memory a `GrTexInfo` needs. Deviation: the even/odd split between two TMUs is not
/// made, so the whole chain is counted.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexTextureMemRequired(_even_odd: u32, info: *const GrTexInfo) -> u32 {
    let Some(i) = (unsafe { info.as_ref() }) else { return 0 };
    grTexCalcMemRequired(i.small_lod, i.large_lod, i.aspect, i.format)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexSource(tmu: u32, start: u32, _even_odd: u32, info: *const GrTexInfo) {
    if tmu != 0 || info.is_null() {
        return;
    }
    shadow::source(start, unsafe { &*info }.info());
}

/// Copy levels `large`..=`small` from the game's packed data into TMU memory at `start`,
/// each level at its padded offset. Deviation: the even/odd mask is taken as both, as with
/// one TMU every level is on it.
pub unsafe fn download_chain(start: u32, small: u32, large: u32, aspect: u32, format: u32, data: *const u8) {
    let Some(f) = Format::from_u32(format) else { return };
    let (Some(total), Some(host)) = (layout::level_offset(large, small + 1, aspect, f), layout::host_offset(large, small + 1, aspect, f)) else {
        return;
    };
    if data.is_null() || total == 0 {
        return;
    }
    let src = unsafe { std::slice::from_raw_parts(data, host) };
    let mut buf = vec![0u8; total];
    for lod in large..=small {
        let (to, from, n) = (
            layout::level_offset(large, lod, aspect, f).unwrap_or(0),
            layout::host_offset(large, lod, aspect, f).unwrap_or(0),
            layout::host_level_bytes(lod, aspect, f).unwrap_or(0),
        );
        buf[to..to + n].copy_from_slice(&src[from..from + n]);
    }
    push(Cmd::Download { start, data: buf });
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadMipMap(tmu: u32, start: u32, _even_odd: u32, info: *const GrTexInfo) {
    let Some(i) = (unsafe { info.as_ref() }) else { return };
    if tmu == 0 {
        unsafe { download_chain(start, i.small_lod, i.large_lod, i.aspect, i.format, i.data) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadMipMapLevel(
    tmu: u32, start: u32, lod: u32, large_lod: u32, aspect: u32, format: u32, even_odd: u32, data: *const u8,
) {
    let h = size(lod, aspect).map_or(1, |(_, h)| h as i32);
    unsafe { grTexDownloadMipMapLevelPartial(tmu, start, lod, large_lod, aspect, format, even_odd, data, 0, h - 1) };
}

/// Rows `first`..=`last` of one level. `data` holds those rows only, from row `first` on (the
/// Glide 2.x reference manual's reading; the original's own code was not traced for it).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadMipMapLevelPartial(
    tmu: u32, start: u32, lod: u32, large_lod: u32, aspect: u32, format: u32, _even_odd: u32, data: *const u8, first: i32, last: i32,
) {
    let (Some(f), Some((w, h))) = (Format::from_u32(format), size(lod, aspect)) else { return };
    if tmu != 0 || data.is_null() || first < 0 || last < first || last as usize >= h {
        return;
    }
    let Some(offset) = layout::level_offset(large_lod, lod, aspect, f) else { return };
    let row = f.texel().bytes(w, 1);
    let n = row * (last - first + 1) as usize;
    let data = unsafe { std::slice::from_raw_parts(data, n) }.to_vec();
    push(Cmd::Download { start: start + (offset + row * first as usize) as u32, data });
}

/// `GR_TEXTABLE_PALETTE`: 256 words `0x00RRGGBB`. `GR_TEXTABLE_NCC0` and `NCC1`: a
/// `GuNccTable`, of which the first 64 bytes are read.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadTable(tmu: u32, kind: u32, data: *const u32) {
    unsafe { grTexDownloadTablePartial(tmu, kind, data, 0, 255) };
}

/// Palette entries `first`..=`last`, read from their own places in the full table. An NCC
/// table is always loaded whole.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexDownloadTablePartial(tmu: u32, kind: u32, data: *const u32, first: i32, last: i32) {
    if tmu != 0 || data.is_null() {
        return;
    }
    if kind == TEXTABLE_PALETTE {
        let (first, last) = (first.clamp(0, 255) as usize, last.clamp(0, 255) as usize);
        if first <= last {
            let words = (first..=last).map(|i| unsafe { data.add(i).read_unaligned() }).collect();
            push(Cmd::Palette { first, words });
        }
    } else if kind < 2 {
        let mut t = Box::new([0u8; 112]);
        unsafe { std::ptr::copy_nonoverlapping(data.cast::<u8>(), t.as_mut_ptr(), 112) };
        push(Cmd::Ncc(kind as u8, t));
    }
}

/// Which NCC table (`GR_NCCTABLE_NCC0` or `NCC1`) the NCC formats read.
#[unsafe(no_mangle)]
pub extern "system" fn grTexNCCTable(tmu: u32, table: u32) {
    if tmu == 0 {
        shadow::set(|s| s.ncc = table & 1);
        shadow::resend_source();
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexMultibase(tmu: u32, enable: u32) {
    if tmu == 0 {
        shadow::set(|s| s.multibase = (enable != 0) as u32);
        shadow::resend_source();
    }
}

/// The address of one multibase range (`GR_TEXBASE_256`, `_128`, `_64`, `_32_TO_1`).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grTexMultibaseAddress(tmu: u32, range: u32, start: u32, _even_odd: u32, _info: *const GrTexInfo) {
    if tmu == 0 && range < 4 {
        shadow::set(|s| s.bases[range as usize] = start);
        shadow::resend_source();
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexFilterMode(tmu: u32, min: u32, mag: u32) {
    if tmu == 0 {
        send(Cmd::TexFilter(min, mag));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexClampMode(tmu: u32, s: u32, t: u32) {
    if tmu == 0 {
        send(Cmd::TexClamp(s, t));
    }
}

/// `GR_MIPMAP_DISABLE` draws the largest level only; `GR_MIPMAP_NEAREST` (and its dithered
/// form, drawn without dither) picks a level per pixel, and `lod_blend` blends between two
/// (trilinear).
#[unsafe(no_mangle)]
pub extern "system" fn grTexMipMapMode(tmu: u32, mode: u32, lod_blend: u32) {
    if tmu == 0 {
        send(Cmd::MipMap(mode, lod_blend != 0));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexLodBiasValue(tmu: u32, bias: f32) {
    if tmu == 0 {
        send(Cmd::LodBias(bias));
    }
}

/// Deviation: detail textures blend a second texture by the level of detail, which needs a
/// second TMU; with one, nothing changes.
#[unsafe(no_mangle)]
pub extern "system" fn grTexDetailControl(_tmu: u32, _lod_bias: i32, _scale: u32, _max: f32) {
    shim::gap!("grTexDetailControl", "detail textures need a second TMU; ignored");
}

#[unsafe(no_mangle)]
pub extern "system" fn grTexCombineFunction(tmu: u32, f: u32) {
    if tmu == 0 {
        send(Cmd::TexCombine(f));
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
            shim::gap!("grTexCombine function", "({rgb_func}, {rgb_factor}) taken as decal");
            1
        }
    };
    let f = match (rgb_invert != 0, f) {
        (false, f) => f,
        (true, 0) => 10,
        (true, 10) => 0,
        (true, _) => {
            shim::gap!("grTexCombine invert", "an inverted texel is taken as decal");
            1
        }
    };
    send(Cmd::TexCombine(f));
}
