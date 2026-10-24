//! The `gu` movie recorder: while it runs, each frame shown is written to a file.
//!
//! Deviation: in Glide 2.43 for the Voodoo Graphics these three calls only report
//! "unsupported on SST hardware" (*measured*, the Win95 `glide2x.dll`). Here they record. At
//! each `grBufferSwap` the frame just shown is read back at the game's resolution (RGB565,
//! as `grLfbReadRegion` gives it) and written as an uncompressed 24-bit TGA file named
//! `NAME0000.tga`, `NAME0001.tga` and so on. The file format is this library's choice.

use super::lfb::with as lfb;
use super::lfbread::grLfbReadRegion;
use super::sst::screen;
use crate::consts::BUFFER_FRONT;
use crate::log::log;
use std::ffi::{CStr, c_char};
use std::sync::Mutex;

struct Movie {
    name: String,
    recording: bool,
    frame: u32,
}

static MOVIE: Mutex<Movie> = Mutex::new(Movie { name: String::new(), recording: false, frame: 0 });

fn with<R>(f: impl FnOnce(&mut Movie) -> R) -> R {
    f(&mut MOVIE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// The start of each file's name, a path the game gives. The default is `movie`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn guMovieSetName(name: *const c_char) {
    if name.is_null() {
        return;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    with(|m| m.name = name);
}

/// Start recording. The frame count starts again at 0.
#[unsafe(no_mangle)]
pub extern "system" fn guMovieStart() {
    with(|m| {
        if m.name.is_empty() {
            m.name = "movie".into();
        }
        m.recording = true;
        m.frame = 0;
        log!("guMovieStart: recording to {}0000.tga on", m.name);
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn guMovieStop() {
    with(|m| m.recording = false);
}

/// Called by `grBufferSwap` after the swap: write the frame now shown, while recording.
pub fn frame() {
    let Some((path, n)) = with(|m| {
        m.recording.then(|| {
            m.frame += 1;
            (format!("{}{:04}.tga", m.name, m.frame - 1), m.frame - 1)
        })
    }) else {
        return;
    };
    let (w, h) = screen();
    let mut px = vec![0u16; (w * h) as usize];
    if unsafe { grLfbReadRegion(BUFFER_FRONT, 0, 0, w, h, w * 2, px.as_mut_ptr().cast()) } == 0 {
        return;
    }
    // Rows come in the game's origin: bottom first for a lower-left origin, as TGA stores.
    let bottom_first = lfb(|l| l.lower_left);
    if let Err(e) = std::fs::write(&path, tga(w, h, &px, bottom_first)) {
        log!("guMovie: frame {n} not written to {path}: {e}");
    }
}

/// An uncompressed 24-bit TGA of RGB565 pixels, each channel widened to 8 bits.
fn tga(w: u32, h: u32, px: &[u16], bottom_first: bool) -> Vec<u8> {
    let mut f = Vec::with_capacity(18 + px.len() * 3);
    f.extend_from_slice(&[0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    f.extend_from_slice(&(w as u16).to_le_bytes());
    f.extend_from_slice(&(h as u16).to_le_bytes());
    // 24 bits; descriptor bit 5 says the first row is the top.
    f.extend_from_slice(&[24, if bottom_first { 0 } else { 0x20 }]);
    for &p in px {
        let (r, g, b) = ((p >> 11) as u8, ((p >> 5) & 0x3f) as u8, (p & 0x1f) as u8);
        f.extend_from_slice(&[b << 3 | b >> 2, g << 2 | g >> 4, r << 3 | r >> 2]);
    }
    f
}
