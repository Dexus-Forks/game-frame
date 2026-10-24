//! A small log, one per DLL: `init` names the DLL (the line prefix) and the environment
//! variable holding a file to log to (a Windows path such as `C:\glide.log`); without it the
//! log goes to standard error, which Wine passes to the terminal. Write errors are ignored.

use std::io::Write;
use std::sync::{Mutex, OnceLock};

static NAME: OnceLock<(&'static str, Option<String>)> = OnceLock::new();
static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

/// Name the DLL and its log variable; the first call wins.
pub fn init(name: &'static str, env: &str) {
    let _ = NAME.set((name, std::env::var(env).ok().filter(|v| !v.is_empty())));
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
