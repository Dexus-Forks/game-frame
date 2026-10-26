//! An installed game's usual lookup: a file in the install folder is read before the disc's
//! copy. The top layer is a folder of added or changed files, such as extra levels an installer
//! or a patch puts there; the base is the disc.
//! Names match case-blind, as on Windows: the top layer's paths are listed in upper case.

use crate::{FileInfo, Source, not_found};
use std::collections::BTreeMap;
use std::io;

pub struct Layered {
    top: Box<dyn Source>,
    base: Box<dyn Source>,
    /// Upper-case path to the top layer's own path.
    top_paths: BTreeMap<String, String>,
}

impl Layered {
    pub fn new(top: Box<dyn Source>, base: Box<dyn Source>) -> Layered {
        let top_paths = top
            .list()
            .into_iter()
            .map(|f| (f.path.to_ascii_uppercase(), f.path))
            .collect();
        Layered {
            top,
            base,
            top_paths,
        }
    }

    /// The top layer's path for `path`, when it has the file.
    fn on_top(&self, path: &str) -> Option<&str> {
        self.top_paths
            .get(&path.to_ascii_uppercase())
            .map(String::as_str)
    }
}

impl Source for Layered {
    fn describe(&self) -> String {
        format!("{} over {}", self.top.describe(), self.base.describe())
    }

    fn list(&self) -> Vec<FileInfo> {
        let mut all: BTreeMap<String, FileInfo> = BTreeMap::new();
        for f in self.base.list() {
            all.insert(f.path.to_ascii_uppercase(), f);
        }
        for f in self.top.list() {
            let key = f.path.to_ascii_uppercase();
            // A replaced file keeps the base's spelling, an added one is listed in upper case.
            let path = all.get(&key).map_or(key.clone(), |b| b.path.clone());
            all.insert(key, FileInfo { path, size: f.size });
        }
        let mut out: Vec<FileInfo> = all.into_values().collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        match self.on_top(path) {
            Some(p) => self.top.read_range(p, offset, len),
            None => self.base.read_range(path, offset, len),
        }
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        if let Some(p) = self.on_top(path) {
            return self.top.read(p);
        }
        match self.base.read(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // The base may spell the name in another case.
                let up = path.to_ascii_uppercase();
                let hit = self
                    .base
                    .list()
                    .into_iter()
                    .find(|f| f.path.to_ascii_uppercase() == up);
                hit.map_or(Err(not_found(path)), |f| self.base.read(&f.path))
            }
            r => r,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mem(Vec<(&'static str, &'static [u8])>);

    impl Source for Mem {
        fn describe(&self) -> String {
            "memory".into()
        }
        fn list(&self) -> Vec<FileInfo> {
            self.0
                .iter()
                .map(|(p, b)| FileInfo {
                    path: p.to_string(),
                    size: b.len() as u64,
                })
                .collect()
        }
        fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>> {
            let (_, b) = self
                .0
                .iter()
                .find(|(p, _)| *p == path)
                .ok_or_else(|| not_found(path))?;
            Ok(b[offset as usize..(offset + len) as usize].to_vec())
        }
    }

    #[test]
    fn the_top_layer_wins_case_blind() {
        let base = Mem(vec![
            ("BINARY/SCRIPTS/CIRCUITS.BIN", b"disc"),
            ("BINARY/MIMG/BELTANE.IMG", b"belt"),
        ]);
        let top = Mem(vec![
            ("Binary/Scripts/circuits.bin", b"installed"),
            ("Binary/Circuits/Tron.BL4", b"tron"),
        ]);
        let l = Layered::new(Box::new(top), Box::new(base));
        assert_eq!(l.read("BINARY/SCRIPTS/CIRCUITS.BIN").unwrap(), b"installed");
        assert_eq!(l.read("BINARY/CIRCUITS/TRON.BL4").unwrap(), b"tron");
        assert_eq!(l.read("BINARY/MIMG/BELTANE.IMG").unwrap(), b"belt");
        assert!(l.read("BINARY/MIMG/NONE.IMG").is_err());
        let paths: Vec<String> = l.list().into_iter().map(|f| f.path).collect();
        assert_eq!(
            paths,
            [
                "BINARY/CIRCUITS/TRON.BL4",
                "BINARY/MIMG/BELTANE.IMG",
                "BINARY/SCRIPTS/CIRCUITS.BIN"
            ]
        );
    }
}
