//! Reads of the frame buffer: the early Glide 2 read pointer, and the regions Glide 2.43
//! copies out. The render thread gives a buffer at the game's resolution as 16-bit values,
//! 1024 per row: RGB565 for the colour buffers, Glide's depth values for the depth buffer.

use super::lfb::{finish_write, with};
use crate::api::sst::screen;
use crate::cmd::read;
use crate::consts::*;

/// One buffer, rows turned to the frame buffer origin the game uses.
fn rows(buffer: u32) -> Vec<u16> {
    finish_write();
    let mut data = read(buffer);
    if with(|l| l.lower_left) {
        let h = (screen().1 as usize).min(data.len() / 1024);
        let mut top = data[..h * 1024].chunks(1024).rev().flatten().copied().collect::<Vec<u16>>();
        top.extend_from_slice(&data[h * 1024..]);
        data = top;
    }
    data
}

#[unsafe(no_mangle)]
pub extern "system" fn grLfbGetReadPtr(buffer: u32) -> *const u16 {
    let data = rows(buffer);
    with(|l| {
        let dst = if buffer == BUFFER_DEPTH || buffer == BUFFER_AUX { &mut l.depth } else { &mut l.front };
        let n = data.len().min(dst.len());
        dst[..n].copy_from_slice(&data[..n]);
        dst.as_ptr()
    })
}

/// Copy a `w × h` rectangle at `(x, y)` of `buffer` to `dst`, `stride` bytes per row.
unsafe fn region(buffer: u32, x: u32, y: u32, w: u32, h: u32, stride: u32, dst: *mut u8) -> bool {
    let (sw, sh) = screen();
    if dst.is_null() || x + w > sw || y + h > sh {
        return false;
    }
    let data = rows(buffer);
    for r in 0..h as usize {
        let src = &data[(y as usize + r) * 1024 + x as usize..][..w as usize];
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr().cast::<u8>(), dst.add(r * stride as usize), w as usize * 2) };
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grLfbReadRegion(buffer: u32, x: u32, y: u32, w: u32, h: u32, stride: u32, dst: *mut u8) -> u32 {
    unsafe { region(buffer, x, y, w, h, stride, dst) as u32 }
}

/// The early Glide 2 read of a rectangle, from the back buffer the game draws in.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guFbReadRegion(x: u32, y: u32, w: u32, h: u32, dst: *mut u8, stride: u32) {
    unsafe { region(BUFFER_BACK, x, y, w, h, stride, dst) };
}
