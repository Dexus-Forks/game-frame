//! Where the game's files come from, behind one interface: a folder of extracted files, a CD
//! image read in place (`disc-image`), or a URL the user gives. Natively a URL is downloaded
//! once, with resume, into a cache folder and checked against the MD5 the user gives; in the
//! browser it is read in ranges (HTTP range requests) and the chunks are kept in the browser's
//! private file system (OPFS) or IndexedDB.
//!
//! The crate takes URLs; it holds none.

mod folder;
mod image;
mod layered;
pub mod md5;
pub mod remote;
pub mod url;

#[cfg(not(target_arch = "wasm32"))]
pub mod download;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub use disc_image;
pub use folder::Folder;
pub use image::Image;
pub use layered::Layered;

use std::io;
use std::sync::Arc;

/// One file of the game, read whole.
#[derive(Clone)]
pub struct GameFile {
    /// Relative to the source's root, '/' between names ("BINARY/MIMG/62.IMG").
    pub path: String,
    pub bytes: Arc<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
}

/// A place the files can be read from.
pub trait Source {
    /// Short source summary for the viewer's status line, including its kind and location.
    fn describe(&self) -> String;
    /// Every file, sorted by path.
    fn list(&self) -> Vec<FileInfo>;
    /// `len` bytes of the file at `path` from `offset`.
    fn read_range(&self, path: &str, offset: u64, len: u64) -> io::Result<Vec<u8>>;

    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let size = self
            .list()
            .into_iter()
            .find(|f| f.path == path)
            .ok_or_else(|| not_found(path))?
            .size;
        self.read_range(path, 0, size)
    }

    /// Every file, read whole.
    fn read_all(&self) -> io::Result<Vec<GameFile>> {
        self.list()
            .into_iter()
            .map(|f| {
                Ok(GameFile {
                    bytes: Arc::new(self.read(&f.path)?),
                    path: f.path,
                })
            })
            .collect()
    }
}

pub(crate) fn not_found(path: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("{path}: no such file"))
}

/// Whether `name` looks like a CD image or its CUE sheet.
pub fn is_image_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".cue", ".bin", ".iso", ".img"]
        .iter()
        .any(|e| n.ends_with(e))
}

/// Open a folder, or a CD image (`.cue`, `.bin`, `.iso`). In an image, `subfolder` (such as
/// "DATA") becomes the root, so the paths match a copy of that folder.
#[cfg(not(target_arch = "wasm32"))]
pub fn open(path: &std::path::Path, subfolder: Option<&str>) -> io::Result<Box<dyn Source>> {
    if path.is_dir() {
        return Ok(Box::new(Folder::new(path)?));
    }
    let disc = disc_image::Disc::open_path(path)?;
    Ok(Box::new(Image::new(
        disc.volume()?,
        subfolder,
        &path.display().to_string(),
    )))
}
