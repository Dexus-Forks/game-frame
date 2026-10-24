//! A small log, one per DLL: `init` names the DLL (the line prefix) and the environment
//! variable holding a file to log to (a Windows path such as `C:\glide.log`); without it the
//! log goes to standard error, which Wine passes to the terminal. Write errors are ignored.
//!
//! Every DLL logs and counts what its library does not do yet, in the same way:
//! - `unhandled`: a call the library exports only as a stub (`stub!`, written by
//!   `wrappers/stubgen.py` from the original library's API table);
//! - `gap!`: a mode, state or kind of object an implemented call does not handle.
//!
//! A release build logs the first of each, with its arguments, and counts the rest. A debug
//! build logs the first `DEBUG_CALLS` of each. `summary`, called when the DLL unloads, lists
//! every one with its count. That list is what a game still needs.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

static NAME: OnceLock<(&'static str, Option<String>)> = OnceLock::new();
static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
static COUNTS: Mutex<BTreeMap<&'static str, u64>> = Mutex::new(BTreeMap::new());

/// In a debug build, how many calls of each stub or gap are logged one by one.
pub const DEBUG_CALLS: u64 = 50;

/// `debug` or `release`: which build of the library this is.
pub const BUILD: &str = if cfg!(debug_assertions) { "debug" } else { "release" };

/// Name the DLL and its log variable; the first call wins. Logs which build is loaded.
pub fn init(name: &'static str, env: &str) {
    if NAME.set((name, std::env::var(env).ok().filter(|v| !v.is_empty()))).is_ok() {
        write(format_args!("{BUILD} build, GameFrame wrappers {}", env!("CARGO_PKG_VERSION")));
    }
}

pub fn write(msg: std::fmt::Arguments) {
    let (name, path) = NAME.get().map(|(n, p)| (*n, p.as_deref())).unwrap_or(("wrapper", None));
    let mut f = FILE.lock().unwrap_or_else(|e| e.into_inner());
    if f.is_none() {
        if let Some(path) = path {
            *f = std::fs::OpenOptions::new().create(true).append(true).open(path).ok();
        }
    }
    match f.as_mut() {
        Some(file) => {
            let _ = writeln!(file, "{name}: {msg}");
            let _ = file.flush();
        }
        None => {
            let _ = writeln!(std::io::stderr(), "{name}: {msg}");
        }
    }
}

/// Count one more of `key`, and say whether to log it.
fn count(key: &'static str) -> (u64, bool) {
    let mut c = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
    let n = c.entry(key).or_insert(0);
    *n += 1;
    (*n, *n == 1 || (cfg!(debug_assertions) && *n <= DEBUG_CALLS))
}

/// Log a call to a stub. A stub is an export of the original library that this one does not
/// implement yet.
pub fn unhandled(name: &'static str, args: &[u32]) {
    let (n, show) = count(name);
    if show {
        let args: Vec<String> = args.iter().map(|a| format!("{a:#x}")).collect();
        write(format_args!("not implemented: {name}({}){}", args.join(", "), later(n)));
    }
}

/// Log something an implemented call does not handle. `key` names it (`"grTexMipMapMode"`).
/// `detail` describes this case.
pub fn gap(key: &'static str, detail: std::fmt::Arguments) {
    let (n, show) = count(key);
    if show {
        write(format_args!("not handled: {key}: {detail}{}", later(n)));
    }
}

fn later(n: u64) -> &'static str {
    match (n, cfg!(debug_assertions)) {
        (1, false) => " (later calls are counted)",
        (DEBUG_CALLS, true) => " (later calls are counted)",
        _ => "",
    }
}

/// Every stub and gap met so far, with its count, most frequent first.
pub fn counts() -> Vec<(&'static str, u64)> {
    let mut v: Vec<_> = COUNTS.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(k, n)| (*k, *n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    v
}

/// Log the counts. Each DLL calls this from `DllMain` when it unloads.
pub fn summary() {
    let v = counts();
    if v.is_empty() {
        write(format_args!("every call this game made is implemented"));
        return;
    }
    let list: Vec<String> = v.iter().map(|(k, n)| format!("{k} ×{n}")).collect();
    write(format_args!("not implemented or not handled, by count: {}", list.join(", ")));
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => { $crate::log::write(format_args!($($t)*)) };
}

/// Log a message the first time a call site reaches it.
#[macro_export]
macro_rules! log_once {
    ($($t:tt)*) => {{
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            $crate::log::write(format_args!($($t)*));
        }
    }};
}

/// Something not handled yet, counted under `key`: `gap!("grTexMipMapMode", "mode {mode}")`.
#[macro_export]
macro_rules! gap {
    ($key:literal, $($t:tt)*) => { $crate::log::gap($key, format_args!($($t)*)) };
}

/// A stub for an export of the original library, written by `wrappers/stubgen.py`. Each
/// argument is a 32-bit stack word. The forms: `stub!(system NAME(a0, a1) -> VALUE)` removes
/// its arguments (`__stdcall`); `stub!(C NAME(a0) -> VALUE)` leaves them to the caller
/// (`__cdecl`); `float` before the name returns 0.0 in st0, as a C `float` function does.
#[macro_export]
macro_rules! stub {
    (system $name:ident($($a:ident),*) -> $ret:expr) => {
        #[unsafe(no_mangle)]
        pub extern "system" fn $name($($a: u32),*) -> u32 {
            $crate::log::unhandled(stringify!($name), &[$($a),*]);
            $ret
        }
    };
    (system float $name:ident($($a:ident),*)) => {
        #[unsafe(no_mangle)]
        pub extern "system" fn $name($($a: u32),*) -> f32 {
            $crate::log::unhandled(stringify!($name), &[$($a),*]);
            0.0
        }
    };
    (C $name:ident($($a:ident),*) -> $ret:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($a: u32),*) -> u32 {
            $crate::log::unhandled(stringify!($name), &[$($a),*]);
            $ret
        }
    };
    (C float $name:ident($($a:ident),*)) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($a: u32),*) -> f32 {
            $crate::log::unhandled(stringify!($name), &[$($a),*]);
            0.0
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stubs_and_gaps_are_counted() {
        unhandled("test_stub", &[1, 2]);
        unhandled("test_stub", &[3, 4]);
        gap("test_gap", format_args!("mode 4"));
        let c = counts();
        assert!(c.contains(&("test_stub", 2)));
        assert!(c.contains(&("test_gap", 1)));
        assert_eq!(c.iter().position(|e| e.0 == "test_stub"), Some(0));
    }

    crate::stub!(C test_cdecl(a0, a1) -> a0 + a1);

    #[test]
    fn a_stub_returns_its_value() {
        assert_eq!(test_cdecl(2, 3), 5);
    }
}
