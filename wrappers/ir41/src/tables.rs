//! Where the codec finds Intel's tables. They are Intel's data, so they are not in the
//! codec: it reads them from the player's own copy of Intel's `IR41_32.DLL`, the first of:
//!
//! 1. the file `WRAPPER_IR41_TABLES` names;
//! 2. `ir41_32.intel.dll` beside this DLL (an installer copies the player's file there);
//! 3. `INSTALL\IR41_32.DLL` on a CD drive, D: to Z: (where POD Gold keeps Intel's setup).
//!
//! Plain Rust, tested on the host.

use std::path::{Path, PathBuf};

/// The places to look, in order, for a codec in `module_dir`.
pub fn candidates(setting: Option<&str>, module_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(s) = setting.filter(|s| !s.is_empty()) {
        out.push(PathBuf::from(s));
    }
    if let Some(d) = module_dir {
        out.push(d.join("ir41_32.intel.dll"));
    }
    for drive in b'D'..=b'Z' {
        out.push(PathBuf::from(format!("{}:\\INSTALL\\IR41_32.DLL", drive as char)));
    }
    out
}

/// The first candidate that holds Intel's tables.
pub fn load(candidates: &[PathBuf]) -> Option<(PathBuf, indeo::Tables)> {
    candidates.iter().find_map(|p| {
        let bytes = std::fs::read(p).ok()?;
        indeo::Tables::from_dll(&bytes).ok().map(|t| (p.clone(), t))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_comes_first_then_the_codec_folder_then_the_cd() {
        let c = candidates(Some("C:\\POD\\IR41_32.DLL"), Some(Path::new("C:\\WINDOWS\\SYSTEM")));
        assert_eq!(c[0], PathBuf::from("C:\\POD\\IR41_32.DLL"));
        assert_eq!(c[1], Path::new("C:\\WINDOWS\\SYSTEM").join("ir41_32.intel.dll"));
        assert_eq!(c[2], PathBuf::from("D:\\INSTALL\\IR41_32.DLL"));
        assert_eq!(candidates(Some(""), None)[0], PathBuf::from("D:\\INSTALL\\IR41_32.DLL"));
    }

    #[test]
    fn a_file_that_is_not_intels_gives_no_tables() {
        let dir = std::env::temp_dir().join(format!("ir41-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("not-a-dll");
        std::fs::write(&p, b"MZ but nothing else").unwrap();
        assert!(load(&[p]).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }
}
