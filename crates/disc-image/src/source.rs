//! Where the image's bytes come from: one small trait and the two plain sources.

use std::io;
use std::sync::Arc;

/// Random access to the bytes of one image file.
pub trait ReadAt: Send + Sync {
    /// Fill all of `buf` from `offset`. Reading past the end fails with `UnexpectedEof`; a
    /// source that fetches on demand fails with `WouldBlock` and a [`crate::Missing`].
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<()>;
    /// The size in bytes.
    fn len(&self) -> u64;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: ReadAt + ?Sized> ReadAt for Arc<T> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_at(offset, buf)
    }
    fn len(&self) -> u64 {
        (**self).len()
    }
}

pub(crate) fn past_end(offset: u64, want: usize, len: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::UnexpectedEof,
        format!("read of {want} bytes at {offset} is past the end ({len} bytes)"),
    )
}

/// An image held in memory.
pub struct Memory(pub Arc<[u8]>);

impl Memory {
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Memory(bytes.into())
    }
}

impl ReadAt for Memory {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        let start = usize::try_from(offset).map_err(|_| past_end(offset, buf.len(), self.len()))?;
        let src = start
            .checked_add(buf.len())
            .and_then(|end| self.0.get(start..end));
        buf.copy_from_slice(src.ok_or_else(|| past_end(offset, buf.len(), self.len()))?);
        Ok(())
    }
    fn len(&self) -> u64 {
        self.0.len() as u64
    }
}

/// An image file on disk, read with positioned reads (no shared file cursor).
pub struct FileSource {
    file: std::fs::File,
    len: u64,
}

impl FileSource {
    pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let len = file.metadata()?.len();
        Ok(FileSource { file, len })
    }
}

impl ReadAt for FileSource {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        if offset
            .checked_add(buf.len() as u64)
            .is_none_or(|end| end > self.len)
        {
            return Err(past_end(offset, buf.len(), self.len));
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::FileExt::read_exact_at(&self.file, buf, offset)
        }
        #[cfg(windows)]
        {
            let mut done = 0;
            while done < buf.len() {
                let n = std::os::windows::fs::FileExt::seek_read(
                    &self.file,
                    &mut buf[done..],
                    offset + done as u64,
                )?;
                if n == 0 {
                    return Err(past_end(offset, buf.len(), self.len));
                }
                done += n;
            }
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (offset, buf, &self.file);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "no positioned file reads on this platform",
            ))
        }
    }
    fn len(&self) -> u64 {
        self.len
    }
}
