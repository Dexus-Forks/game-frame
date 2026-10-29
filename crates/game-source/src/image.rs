//! The files of a CD image's ISO 9660 volume, read in place.

use crate::{FileInfo, Source, not_found};
use disc_image::{Entry, Volume};
use std::io;

pub struct Image {
    volume: Volume,
    /// "DATA/" when a subfolder is the root, else "".
    prefix: String,
    name: String,
}

impl Image {
    /// The files under `subfolder` (case-blind; all of them when `None`), named relative to it.
    pub fn new(volume: Volume, subfolder: Option<&str>, name: &str) -> Image {
        let prefix = subfolder
            .map(|s| format!("{}/", s.trim_matches(['/', '\\'])))
            .filter(|p| p != "/")
            .unwrap_or_default();
        Image {
            volume,
            prefix,
            name: name.to_string(),
        }
    }

    /// The path inside the root, or `None` for an entry outside it.
    fn inside<'a>(&self, e: &'a Entry) -> Option<&'a str> {
        let head = e.path.get(..self.prefix.len())?;
        head.eq_ignore_ascii_case(&self.prefix)
            .then(|| &e.path[self.prefix.len()..])
    }

    fn entry(&self, path: &str) -> io::Result<&Entry> {
        self.volume
            .find(&format!("{}{path}", self.prefix))
            .filter(|e| !e.directory)
            .ok_or_else(|| not_found(path))
    }

    pub fn volume(&self) -> &Volume {
        &self.volume
    }
}

impl Source for Image {
    fn describe(&self) -> String {
        let n = self
            .volume
            .files()
            .filter(|e| self.inside(e).is_some())
            .count();
        let root = if self.prefix.is_empty() {
            String::new()
        } else {
            format!(" in {}", self.prefix.trim_end_matches('/'))
        };
        format!(
            "{n} files{root} of volume {} in {}",
            self.volume.info.volume, self.name
        )
    }

    fn list(&self) -> Vec<FileInfo> {
        let mut out: Vec<FileInfo> = self
            .volume
            .files()
            .filter_map(|e| {
                Some(FileInfo {
                    path: self.inside(e)?.to_string(),
                    size: e.size,
                })
            })
            .collect();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        self.volume.read_range(self.entry(path)?, offset, len)
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        self.volume.read(self.entry(path)?)
    }
}
