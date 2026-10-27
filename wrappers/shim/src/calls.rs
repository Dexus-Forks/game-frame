//! The call log: every call the game makes into a stand-in library, one line each, to the
//! file `WRAPPER_CALLS` names. It is the game's side of the renderer's seam
//! (`docs/hardware/rendering-pipeline.md`): record it from the original, write the same lines
//! from the port's device interface, and `tools/verify/seqdiff.py` names the first call that
//! differs.
//!
//! Each line is `name(arg, arg)<TAB>frame N`, the format `seqdiff.py` reads: it compares the
//! part before the tab. Arguments are written by `Arg`:
//! - unsigned integers in hex (`0x1f`), signed integers in decimal, floats as Rust prints
//!   them (the shortest text that reads back as the same bits, so two equal floats always
//!   print the same);
//! - pointers as `null` or `ptr`, since an address differs between two runs. A library writes
//!   what a pointer points to with its own `Arg` types (Glide's vertices, its texture data as
//!   a size and a hash, `hash`).
//!
//! `frame` counts the library's end-of-frame calls (`grBufferSwap` for Glide), from 0.
//! `WRAPPER_CALLS_FRAMES` keeps a range: `100-102` writes frames 100 to 102, `100` writes
//! frame 100 on. Without it every frame is written, and a long run makes a large file.
//!
//! Each line is one write to a file opened for appending, so two libraries in one process
//! (Glide and DirectDraw) can share the file and their lines stay whole. Each library counts
//! its own frames. A `{dll}` in the path is replaced by the library's log name, for one file
//! each. Without `WRAPPER_CALLS` the check is one atomic load per call and nothing is written.

use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

static STATE: AtomicU8 = AtomicU8::new(0); // 0 not read yet, 1 off, 2 on
static FRAME: AtomicU64 = AtomicU64::new(0);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

struct Settings {
    file: Option<std::fs::File>,
    path: String,
    frames: (u64, u64),
}

/// `WRAPPER_CALLS_FRAMES` as an inclusive range: `A-B`, or `A` for A on. None when malformed.
pub fn parse_frames(text: &str) -> Option<(u64, u64)> {
    let text = text.trim();
    match text.split_once('-') {
        Some((a, b)) => {
            let (a, b) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
            (a <= b).then_some((a, b))
        }
        None => Some((text.parse().ok()?, u64::MAX)),
    }
}

fn read_settings() -> bool {
    let Some(path) = std::env::var("WRAPPER_CALLS").ok().filter(|p| !p.is_empty()) else {
        STATE.store(1, Ordering::Relaxed);
        return false;
    };
    let frames = match std::env::var("WRAPPER_CALLS_FRAMES") {
        Ok(text) if !text.trim().is_empty() => parse_frames(&text).unwrap_or_else(|| {
            crate::log::write(format_args!("WRAPPER_CALLS_FRAMES {text:?} is not A-B or A: every frame is written"));
            (0, u64::MAX)
        }),
        _ => (0, u64::MAX),
    };
    start(&path, frames);
    true
}

/// Start the log at `path`, keeping frames `frames.0` to `frames.1`. The environment does this
/// on the first call; tests call it directly.
pub fn start(path: &str, frames: (u64, u64)) {
    let path = path.replace("{dll}", crate::log::name());
    crate::log::write(format_args!("call log: {path}, frames {}-{}", frames.0,
                                   if frames.1 == u64::MAX { String::new() } else { frames.1.to_string() }));
    *SETTINGS.lock().unwrap_or_else(|e| e.into_inner()) = Some(Settings { file: None, path, frames });
    FRAME.store(0, Ordering::Relaxed);
    STATE.store(2, Ordering::Relaxed);
}

/// Whether this call is written: the log is on and this frame is in its range.
pub fn on() -> bool {
    let on = match STATE.load(Ordering::Relaxed) {
        0 => read_settings(),
        s => s == 2,
    };
    on && {
        let s = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
        let f = FRAME.load(Ordering::Relaxed);
        s.as_ref().is_some_and(|s| f >= s.frames.0 && f <= s.frames.1)
    }
}

/// Write one call. `args` is the text between the brackets.
pub fn write(name: &str, args: &str) {
    let line = format!("{name}({args})\tframe {}\n", FRAME.load(Ordering::Relaxed));
    let mut s = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(s) = s.as_mut() else { return };
    if s.file.is_none() {
        s.file = std::fs::OpenOptions::new().create(true).append(true).open(&s.path).ok();
    }
    if let Some(f) = s.file.as_mut() {
        let _ = f.write_all(line.as_bytes());
    }
}

/// The library's end-of-frame call was made (and written): count the next frame.
pub fn end_frame() {
    FRAME.fetch_add(1, Ordering::Relaxed);
}

/// The frame being written, counted from 0.
pub fn frame() -> u64 {
    FRAME.load(Ordering::Relaxed)
}

/// FNV-1a over the bytes, 64 bits: the same bytes give the same hash in the original's log
/// and the port's, so texture data can be compared without writing it out.
pub fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ *b as u64).wrapping_mul(0x0100_0000_01b3))
}

/// A value written as one argument of a call.
pub trait Arg {
    fn put(&self, out: &mut String);
}

macro_rules! unsigned { ($($t:ty),*) => { $(impl Arg for $t {
    fn put(&self, out: &mut String) { let _ = write!(out, "{:#x}", self); }
})* } }
macro_rules! signed { ($($t:ty),*) => { $(impl Arg for $t {
    fn put(&self, out: &mut String) { let _ = write!(out, "{}", self); }
})* } }
unsigned!(u8, u16, u32, u64, usize);
signed!(i8, i16, i32, i64, isize);

impl Arg for f32 {
    fn put(&self, out: &mut String) { let _ = write!(out, "{self:?}"); }
}
impl Arg for f64 {
    fn put(&self, out: &mut String) { let _ = write!(out, "{self:?}"); }
}
impl<T> Arg for *const T {
    fn put(&self, out: &mut String) { out.push_str(if self.is_null() { "null" } else { "ptr" }); }
}
impl<T> Arg for *mut T {
    fn put(&self, out: &mut String) { out.push_str(if self.is_null() { "null" } else { "ptr" }); }
}
impl Arg for str {
    fn put(&self, out: &mut String) { let _ = write!(out, "{self:?}"); }
}
impl Arg for String {
    fn put(&self, out: &mut String) { self.as_str().put(out) }
}

/// Bytes written as their length and hash: `[4096 bytes 0x1c2d…]`.
pub struct Bytes<'a>(pub &'a [u8]);
impl Arg for Bytes<'_> {
    fn put(&self, out: &mut String) { let _ = write!(out, "[{} bytes {:#018x}]", self.0.len(), hash(self.0)); }
}

/// Append one argument, with the separator.
pub fn arg<T: Arg + ?Sized>(out: &mut String, value: &T) {
    if !out.is_empty() {
        out.push_str(", ");
    }
    value.put(out);
}

thread_local! {
    static DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Marks one call into the library while it runs. Only the outermost call on a thread is
/// written: `guColorCombineFunction` calls `grColorCombine` inside the library, and the
/// game made one call.
pub struct Enter(bool);

impl Enter {
    pub fn outermost(&self) -> bool {
        self.0 && DEPTH.with(|d| d.get()) == 1
    }
}

impl Drop for Enter {
    fn drop(&mut self) {
        if self.0 {
            DEPTH.with(|d| d.set(d.get() - 1));
        }
    }
}

/// Enter a call. Costs one atomic load when the log is off.
pub fn enter() -> Enter {
    if STATE.load(Ordering::Relaxed) == 1 {
        return Enter(false);
    }
    DEPTH.with(|d| d.set(d.get() + 1));
    Enter(true)
}

/// Write a call to the call log, when it is on and the game made it:
/// `call!("grBufferClear", color, alpha, depth)`. Put it first in the exported function, as a
/// statement: it holds a guard to the end of the function, so the calls the library makes to
/// its own exports are not written. Each argument implements `calls::Arg` and is only
/// evaluated when the line is written.
#[macro_export]
macro_rules! call {
    ($name:expr $(, $arg:expr)* $(,)?) => {
        let _call = $crate::calls::enter();
        if _call.outermost() && $crate::calls::on() {
            #[allow(unused_mut)]
            let mut args = String::new();
            $( $crate::calls::arg(&mut args, &$arg); )*
            $crate::calls::write($name, &args);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_parse() {
        assert_eq!(parse_frames("100-102"), Some((100, 102)));
        assert_eq!(parse_frames(" 7 "), Some((7, u64::MAX)));
        assert_eq!(parse_frames("5-2"), None);
        assert_eq!(parse_frames("x"), None);
    }

    #[test]
    fn arguments_print_the_same_every_time() {
        let mut s = String::new();
        arg(&mut s, &31u32);
        arg(&mut s, &-2i32);
        arg(&mut s, &0.1f32);
        arg(&mut s, &std::ptr::null::<u8>());
        arg(&mut s, &Bytes(b"abc"));
        assert_eq!(s, "0x1f, -2, 0.1, null, [3 bytes 0xe71fa2190541574b]");
    }

    #[test]
    fn calls_are_written_for_the_frames_asked() {
        // Each call! holds its guard to the end of its block, as it does in an exported function.
        let path = std::env::temp_dir().join(format!("gameframe-calls-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        start(path.to_str().unwrap(), (1, 1));
        { crate::call!("first", 1u32); }
        end_frame();
        { crate::call!("grDrawTriangle", 1.5f32, 2u32); }
        { crate::call!("grBufferSwap", 1u32); }
        end_frame();
        { crate::call!("later"); }
        fn outer() {
            crate::call!("outer");
            inner();
        }
        fn inner() {
            crate::call!("inner");
        }
        outer();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "grDrawTriangle(1.5, 0x2)\tframe 1\ngrBufferSwap(0x1)\tframe 1\n");
        start(path.to_str().unwrap(), (0, u64::MAX));
        let _ = std::fs::remove_file(&path);
        outer();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "outer()\tframe 0\n");
        let _ = std::fs::remove_file(&path);
    }
}
