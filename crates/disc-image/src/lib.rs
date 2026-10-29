//! Read CD images directly, without extracting them: a CUE sheet and its BIN files, a raw
//! BIN alone, or a plain 2048-byte ISO. It lists the tracks, opens the ISO 9660 volume of a
//! data track (with Joliet names when the disc has them), lists its files and reads any
//! range of a file's bytes without loading the whole image.
//!
//! Everything reads through [`ReadAt`], so the same code works on a local file, a buffer in
//! memory, or a source that fetches on demand (an HTTP server that answers range requests, a
//! file dropped into a browser). Such a source fills a [`ChunkCache`]; a read of bytes that
//! are not there yet fails with a [`Missing`] error that names them, the caller fetches them
//! and tries again.
//!
//! Nothing here knows any particular game.

mod chunks;
mod cue;
mod disc;
mod iso9660;
mod source;

pub use chunks::{ChunkCache, Missing};
pub use cue::{Cue, CueFile, CueTrack, Msf, parse_cue};
pub use disc::{Disc, Mode, Track};
pub use iso9660::{Entry, Timestamp, Volume, VolumeInfo};
pub use source::{FileSource, Memory, ReadAt};

/// The user-data size of a sector in an ISO 9660 track (Mode 1, Mode 2 Form 1, cooked ISO).
pub const SECTOR: usize = 2048;
