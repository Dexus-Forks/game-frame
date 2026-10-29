//! The run-length calls: `guEncodeRLE16`, and `ConvertAndDownloadRle`, which downloads a
//! run-length coded picture into a texture level.

use crate::cmd::{Cmd, push};
use crate::layout;
use crate::rle;
use voodoo::formats::{Format, size};

/// Code `width × height` RGB565 pixels from `src` as runs (`rle::encode`). With `dst` null,
/// only count. Returns the size of the code in bytes.
///
/// Original: `guEncodeRLE16` (glide2x.dll 2.43, 0x10003210).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guEncodeRLE16(dst: *mut u32, src: *const u16, width: u32, height: u32) -> u32 {
    shim::call!("guEncodeRLE16", dst, src, width, height);
    let count = width.wrapping_mul(height);
    if src.is_null() || count == 0 {
        return 0;
    }
    let words = rle::encode(unsafe { std::slice::from_raw_parts(src, count as usize) }, count);
    if !dst.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(words.as_ptr(), dst, words.len()) };
    }
    4 * words.len() as u32
}

/// The Voodoo's texture memory is two banks of 2 MB; one level may not cross between them.
const BANK: u32 = 2 << 20;

/// Download rows of a run-length coded picture into level `lod` of the texture at `start`.
///
/// What the original does (*measured*, the Win95 `glide2x.dll` of Glide 2.43, 0x1000b1f0):
///
/// - It reports an error when the level crosses the 2 MB line in texture memory.
/// - It does nothing unless `even_odd` has the level's bit (`GR_MIPMAPLEVELMASK_ODD` for an
///   odd `lod`, `EVEN` for an even one).
/// - `bm_data` holds four bytes, then `bm_h` bytes with the length of each coded line, then
///   the lines. It skips the first `v0` lines.
/// - For each of `height` rows it calls a line decoder with `tlut`, the line, and a buffer
///   of 256 16-bit texels. Then it writes `dest_width / 2` 32-bit words from that buffer,
///   from texel `u0` on, to row 0, 1, 2… of the level. When `dest_height` is more than
///   `height` it writes the last buffer once more, to row `height`.
///
/// In this build of the original, the line decoder is an empty routine (a lone `ret`, at
/// 0x10004a50). Nothing else writes the buffer, which starts as zeros. So every texel the
/// original downloads is 0, and so is every texel here. What the code's bytes mean, and
/// what `width` (rounded down to even and kept in a global the decoder would read) is
/// for, cannot be settled from this build.
///
/// Original: `ConvertAndDownloadRle` (glide2x.dll 2.43, 0x1000b1f0).
///
/// Deviation: the original goes on after the 2 MB error; so does this one, as its texture
/// memory has no banks. Rows and words beyond the level's own size are dropped here; on the
/// Voodoo they land in the texture memory the address wraps to.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ConvertAndDownloadRle(
    tmu: u32, start: u32, lod: u32, large_lod: u32, aspect: u32, format: u32, even_odd: u32, _bm_data: *const u8,
    _bm_h: i32, _u0: u32, _v0: u32, _width: u32, height: u32, dest_width: u32, dest_height: u32, _tlut: *const u16,
) {
    shim::call!("ConvertAndDownloadRle", tmu, start, lod, large_lod, aspect, format, even_odd, _bm_data, _bm_h, _u0, _v0, _width, height, dest_width, dest_height, _tlut);
    let (Some(f), Some((w, h))) = (Format::from_u32(format), size(lod, aspect)) else { return };
    let need = layout::mem_required(lod, lod, aspect, f);
    if start < BANK && start.saturating_add(need) > BANK {
        shim::gap!("ConvertAndDownloadRle", "the level crosses the 2 MB line in texture memory");
    }
    let mask = if lod & 1 == 1 { 2 } else { 1 };
    if tmu != 0 || even_odd & mask == 0 {
        return;
    }
    let Some(offset) = layout::level_offset(large_lod, lod, aspect, f) else { return };
    let row = f.texel().bytes(w, 1);
    let bytes = ((dest_width / 2) as usize * 4).min(row);
    let rows = height as usize + (dest_height > height) as usize;
    for t in 0..rows.min(h) {
        push(Cmd::Download { start: start + (offset + t * row) as u32, data: vec![0; bytes] });
    }
}
