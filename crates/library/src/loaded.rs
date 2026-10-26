//! Files read whole into memory, as a [`game_source::Source`]: the game data kept in the
//! browser, read once when the game starts, then used through the same trait as a folder or a
//! CD image.

use game_source::{FileInfo, GameFile, Source};
use std::io;

pub struct Loaded {
    what: String,
    files: Vec<GameFile>,
}

impl Loaded {
    /// `what` names where the files came from, for the status line.
    pub fn new(what: impl Into<String>, mut files: Vec<GameFile>) -> Loaded {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Loaded { what: what.into(), files }
    }

    /// The file whose name matches `path` case-blind, as the game's own Windows or DOS lookup did.
    pub fn find(&self, path: &str) -> Option<&GameFile> {
        self.files.iter().find(|f| f.path.eq_ignore_ascii_case(path))
    }
}

impl Source for Loaded {
    fn describe(&self) -> String {
        format!("{} files {}", self.files.len(), self.what)
    }

    fn list(&self) -> Vec<FileInfo> {
        self.files.iter().map(|f| FileInfo { path: f.path.clone(), size: f.bytes.len() as u64 }).collect()
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let f = self.find(path).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{path}: no such file")))?;
        Ok(f.bytes.to_vec())
    }

    fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        let f = self
            .find(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{path}: no such file")))?;
        let start = (offset as usize).min(f.bytes.len());
        let end = start.saturating_add(len as usize).min(f.bytes.len());
        if end - start != len as usize {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("{path}: {len} bytes from {offset} run past the end")));
        }
        Ok(f.bytes[start..end].to_vec())
    }
}
