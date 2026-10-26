//! A folder of files on disk: a copy of the disc, or the game's install folder.

use crate::{FileInfo, Source, not_found};
use std::io;
use std::path::{Path, PathBuf};

pub struct Folder {
    root: PathBuf,
    files: Vec<FileInfo>,
}

impl Folder {
    pub fn new(root: &Path) -> io::Result<Folder> {
        let mut files = Vec::new();
        let mut todo = vec![root.to_path_buf()];
        while let Some(dir) = todo.pop() {
            for entry in std::fs::read_dir(&dir)?.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    todo.push(p);
                } else {
                    let rel = p
                        .strip_prefix(root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/");
                    files.push(FileInfo {
                        path: rel,
                        size: p.metadata()?.len(),
                    });
                }
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Folder {
            root: root.to_path_buf(),
            files,
        })
    }
}

impl Source for Folder {
    fn describe(&self) -> String {
        format!("{} files from {}", self.files.len(), self.root.display())
    }

    fn list(&self) -> Vec<FileInfo> {
        self.files.clone()
    }

    fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        if !self.files.iter().any(|f| f.path == path) {
            return Err(not_found(path));
        }
        let src = disc_image::FileSource::open(self.root.join(path))?;
        let mut buf = vec![0; len as usize];
        disc_image::ReadAt::read_at(&src, offset, &mut buf)?;
        Ok(buf)
    }

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        if !self.files.iter().any(|f| f.path == path) {
            return Err(not_found(path));
        }
        std::fs::read(self.root.join(path))
    }
}
