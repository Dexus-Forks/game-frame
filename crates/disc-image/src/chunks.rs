//! A [`ReadAt`] for sources that cannot answer at once: an HTTP server, a browser file. The
//! caller fills fixed-size chunks; a read that needs a chunk it has not got fails with
//! [`Missing`], and the caller fetches those chunks and reads again.

use crate::source::{ReadAt, past_end};
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

/// The chunks a read needed and did not find, carried inside an `io::Error` of kind
/// `WouldBlock` (see [`Missing::of`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// The [`ChunkCache::id`] of the cache that lacks them.
    pub cache: u64,
    /// Chunk indices, ascending.
    pub chunks: Vec<u64>,
}

impl std::fmt::Display for Missing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chunks {:?} of source {} are not fetched yet",
            self.chunks, self.cache
        )
    }
}

impl std::error::Error for Missing {}

impl Missing {
    /// The `Missing` inside an error, if it is one.
    pub fn of(e: &io::Error) -> Option<&Missing> {
        e.get_ref()
            .and_then(|inner| inner.downcast_ref::<Missing>())
    }
}

pub struct ChunkCache {
    id: u64,
    len: u64,
    chunk_size: u64,
    chunks: Mutex<HashMap<u64, Vec<u8>>>,
}

impl ChunkCache {
    /// A cache over a source of `len` bytes cut into chunks of `chunk_size` bytes. `id` tells
    /// caches apart in a [`Missing`] when one image has several files.
    pub fn new(id: u64, len: u64, chunk_size: u64) -> Self {
        assert!(chunk_size > 0);
        ChunkCache {
            id,
            len,
            chunk_size,
            chunks: Mutex::new(HashMap::new()),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn chunk_size(&self) -> u64 {
        self.chunk_size
    }

    /// The byte range of chunk `index` (the last one may be short).
    pub fn range(&self, index: u64) -> std::ops::Range<u64> {
        let start = index * self.chunk_size;
        start..(start + self.chunk_size).min(self.len)
    }

    /// Store a chunk; it must be the full length of [`Self::range`].
    pub fn insert(&self, index: u64, bytes: Vec<u8>) -> io::Result<()> {
        let r = self.range(index);
        if bytes.len() as u64 != r.end.saturating_sub(r.start) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "chunk {index}: {} bytes, wanted {}",
                    bytes.len(),
                    r.end - r.start
                ),
            ));
        }
        self.chunks.lock().unwrap().insert(index, bytes);
        Ok(())
    }

    /// Drop every chunk held in memory (a persistent store may still have them).
    pub fn clear(&self) {
        self.chunks.lock().unwrap().clear();
    }

    /// How many chunks are held in memory.
    pub fn held(&self) -> usize {
        self.chunks.lock().unwrap().len()
    }
}

impl ReadAt for ChunkCache {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        let end = offset
            .checked_add(buf.len() as u64)
            .filter(|&e| e <= self.len)
            .ok_or_else(|| past_end(offset, buf.len(), self.len))?;
        if buf.is_empty() {
            return Ok(());
        }
        let (first, last) = (offset / self.chunk_size, (end - 1) / self.chunk_size);
        let chunks = self.chunks.lock().unwrap();
        let missing: Vec<u64> = (first..=last).filter(|i| !chunks.contains_key(i)).collect();
        if !missing.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                Missing {
                    cache: self.id,
                    chunks: missing,
                },
            ));
        }
        for i in first..=last {
            let r = self.range(i);
            let (from, to) = (offset.max(r.start), end.min(r.end));
            buf[(from - offset) as usize..(to - offset) as usize]
                .copy_from_slice(&chunks[&i][(from - r.start) as usize..(to - r.start) as usize]);
        }
        Ok(())
    }
    fn len(&self) -> u64 {
        self.len
    }
}
