//! The tracks of an image and their sectors: where each track starts in its file, how big a
//! sector is there, and where the 2048 bytes of user data sit inside a raw sector.

use crate::SECTOR;
use crate::cue::{Cue, CueFile, parse_cue};
use crate::iso9660::Volume;
use crate::source::{FileSource, ReadAt};
use std::io;
use std::path::Path;
use std::sync::Arc;

/// The sync pattern that opens every raw data sector.
const SYNC: [u8; 12] = [
    0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Audio,
    /// 2048 bytes of user data after a 16-byte header.
    Mode1,
    /// CD-ROM XA: Form 1 sectors hold 2048 bytes after the 16-byte header and an 8-byte
    /// subheader; Form 2 sectors (video, streamed sound) hold 2324.
    Mode2,
}

#[derive(Clone)]
pub struct Track {
    pub number: u8,
    pub mode: Mode,
    /// Bytes per sector in the image file: 2048 (cooked), 2336 or 2352 (raw).
    pub sector_size: u32,
    /// The disc address of INDEX 01, in sectors from the start of the program area: the
    /// address ISO 9660 counts in.
    pub start: u32,
    /// Sectors in the image from INDEX 01 to the end of the track.
    pub sectors: u32,
    /// Sectors of silence before INDEX 01 that the image leaves out (PREGAP).
    pub pregap: u32,
    /// The file it is in (from the CUE sheet), or the image's name.
    pub file: String,
    source: Arc<dyn ReadAt>,
    /// Where INDEX 01 starts in `source`.
    offset: u64,
}

impl std::fmt::Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Track")
            .field("number", &self.number)
            .field("mode", &self.mode)
            .field("sector_size", &self.sector_size)
            .field("start", &self.start)
            .field("sectors", &self.sectors)
            .field("pregap", &self.pregap)
            .finish()
    }
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

impl Track {
    pub fn is_data(&self) -> bool {
        self.mode != Mode::Audio
    }

    /// `count` whole sectors as they are in the image, from sector `first` of the track.
    pub fn read_raw(&self, first: u32, count: u32) -> io::Result<Vec<u8>> {
        if first
            .checked_add(count)
            .is_none_or(|end| end > self.sectors)
        {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "sectors {first}+{count} are past the end of track {} ({} sectors)",
                    self.number, self.sectors
                ),
            ));
        }
        let mut buf = vec![0; count as usize * self.sector_size as usize];
        self.source.read_at(
            self.offset + first as u64 * self.sector_size as u64,
            &mut buf,
        )?;
        Ok(buf)
    }

    /// The 2048 bytes of user data of `count` sectors from sector `first` of the track.
    /// A Mode 2 Form 2 sector fails: it holds 2324 bytes, not part of an ISO 9660 file.
    pub fn read_user(&self, first: u32, count: u32) -> io::Result<Vec<u8>> {
        if !self.is_data() {
            return Err(invalid(format!("track {} is audio", self.number)));
        }
        if self.sector_size == SECTOR as u32 {
            return self.read_raw(first, count);
        }
        let mut out = Vec::with_capacity(count as usize * SECTOR);
        // A few hundred sectors at a time: one read each, without holding a whole file twice.
        let mut done = 0;
        while done < count {
            let n = (count - done).min(256);
            let raw = self.read_raw(first + done, n)?;
            for (i, s) in raw.chunks_exact(self.sector_size as usize).enumerate() {
                out.extend_from_slice(user_data(s, first + done + i as u32)?);
            }
            done += n;
        }
        Ok(out)
    }

    /// Whether sector `index` is a Mode 2 Form 2 sector.
    pub fn is_form2(&self, index: u32) -> io::Result<bool> {
        if self.mode != Mode::Mode2 || self.sector_size == SECTOR as u32 {
            return Ok(false);
        }
        let s = self.read_raw(index, 1)?;
        let sub = if self.sector_size == 2352 {
            &s[16..24]
        } else {
            &s[0..8]
        };
        Ok(sub[2] & 0x20 != 0)
    }
}

/// The user data of one raw sector (2336 or 2352 bytes).
fn user_data(s: &[u8], index: u32) -> io::Result<&[u8]> {
    let (mode, rest) = if s.len() == 2352 {
        (s[15], &s[16..])
    } else {
        (2, s)
    };
    match mode {
        0 => Ok(&ZERO),
        1 => Ok(&rest[..SECTOR]),
        2 if rest[2] & 0x20 != 0 => Err(invalid(format!("sector {index} is Mode 2 Form 2"))),
        2 => Ok(&rest[8..8 + SECTOR]),
        m => Err(invalid(format!("sector {index}: unknown mode {m}"))),
    }
}

static ZERO: [u8; SECTOR] = [0; SECTOR];

/// Mode and sector size of a CUE track type.
fn cue_mode(mode: &str) -> io::Result<(Mode, u32)> {
    Ok(match mode {
        "AUDIO" => (Mode::Audio, 2352),
        "MODE1/2048" => (Mode::Mode1, 2048),
        "MODE1/2352" => (Mode::Mode1, 2352),
        "MODE2/2048" => (Mode::Mode2, 2048),
        "MODE2/2336" | "CDI/2336" => (Mode::Mode2, 2336),
        "MODE2/2352" | "CDI/2352" => (Mode::Mode2, 2352),
        m => return Err(invalid(format!("track mode {m} is not supported"))),
    })
}

/// A CD image: its tracks in disc order.
pub struct Disc {
    pub tracks: Vec<Track>,
}

impl Disc {
    /// The tracks a CUE sheet describes; `open` gives the bytes of each FILE it names.
    pub fn from_cue(
        cue: &Cue,
        mut open: impl FnMut(&CueFile) -> io::Result<Arc<dyn ReadAt>>,
    ) -> io::Result<Disc> {
        let mut tracks = Vec::new();
        let (mut file_base, mut pregaps) = (0u32, 0u32);
        for file in &cue.files {
            if file.tracks.is_empty() {
                continue;
            }
            let source = open(file)?;
            let mut offset = 0u64; // byte offset of the current track's first frame
            let mut at = 0u32; // its frame in the file
            let mut prev_index1 = 0u32; // the previous track's INDEX 01, in the file
            let mut file_frames = 0u32;
            for (i, t) in file.tracks.iter().enumerate() {
                let (mode, size) = cue_mode(&t.mode)?;
                let start = t
                    .start()
                    .ok_or_else(|| invalid(format!("track {} has no INDEX", t.number)))?
                    .0;
                let first = t.first().unwrap_or_default().0;
                if first < at || start < first {
                    return Err(invalid(format!(
                        "track {}: its indexes are out of order",
                        t.number
                    )));
                }
                // The frames from the previous track's first frame to this one's belong to the
                // previous track, at its sector size.
                if i > 0 {
                    let prev = tracks.last_mut().expect("a track before this one");
                    let prev: &mut Track = prev;
                    offset += (first - at) as u64 * prev.sector_size as u64;
                    prev.sectors = first.saturating_sub(prev_index1);
                }
                at = first;
                prev_index1 = start;
                let index1 = offset + (start - first) as u64 * size as u64;
                pregaps += t.pregap.0;
                let remaining = (source.len().saturating_sub(index1) / size as u64) as u32;
                tracks.push(Track {
                    number: t.number,
                    mode,
                    sector_size: size,
                    start: file_base + pregaps + start,
                    sectors: remaining,
                    pregap: t.pregap.0,
                    file: file.name.clone(),
                    source: source.clone(),
                    offset: index1,
                });
                file_frames = start + remaining;
            }
            file_base += file_frames;
        }
        Ok(Disc { tracks })
    }

    /// An image with no CUE sheet: a raw BIN (2352 or 2336 bytes a sector) or a cooked ISO,
    /// taken as one data track.
    pub fn from_raw(source: Arc<dyn ReadAt>, name: &str) -> io::Result<Disc> {
        let len = source.len();
        // Probe a few bytes; a source that fetches on demand passes its Missing error up.
        let peek = |at: u64, n: usize| -> io::Result<Option<Vec<u8>>> {
            let mut b = vec![0; n];
            match source.read_at(at, &mut b) {
                Ok(()) => Ok(Some(b)),
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
                Err(e) => Err(e),
            }
        };
        let has_sync = |sector: u64| -> io::Result<Option<Vec<u8>>> {
            Ok(peek(sector * 2352, 16)?.filter(|b| b[..12] == SYNC))
        };
        let cd001 = |at: u64| -> io::Result<bool> {
            Ok(peek(at, 6)?.is_some_and(|b| &b[1..6] == b"CD001"))
        };
        let sync = match has_sync(16)? {
            Some(h) => Some(h),
            None => has_sync(0)?,
        };
        let (mode, size) = if let Some(h) = sync {
            (if h[15] == 2 { Mode::Mode2 } else { Mode::Mode1 }, 2352)
        } else if cd001(16 * 2048)? || (len % 2048 == 0 && len > 0) {
            (Mode::Mode1, 2048)
        } else if cd001(16 * 2336 + 8)? {
            (Mode::Mode2, 2336)
        } else {
            return Err(invalid(format!(
                "{name}: not a CD image (no sync pattern, no ISO 9660 volume)"
            )));
        };
        let track = Track {
            number: 1,
            mode,
            sector_size: size,
            start: 0,
            sectors: (len / size as u64) as u32,
            pregap: 0,
            file: name.to_string(),
            source,
            offset: 0,
        };
        Ok(Disc {
            tracks: vec![track],
        })
    }

    /// Open a `.cue` sheet (its files beside it, names matched case-blind) or a raw image.
    pub fn open_path(path: impl AsRef<Path>) -> io::Result<Disc> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.to_ascii_lowercase().ends_with(".cue") {
            return Disc::from_raw(Arc::new(FileSource::open(path)?), &name);
        }
        let cue = parse_cue(&String::from_utf8_lossy(&std::fs::read(path)?))?;
        let dir = path.parent().unwrap_or(Path::new("."));
        Disc::from_cue(&cue, |f| {
            let want = f.name.rsplit(['/', '\\']).next().unwrap_or(&f.name);
            let mut p = dir.join(want);
            if !p.exists() {
                // Images copied between systems often change the case of the names.
                let hit = std::fs::read_dir(dir)?
                    .flatten()
                    .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(want));
                p = hit.map(|e| e.path()).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "{}: the CUE sheet names {want}, which is not beside it",
                            path.display()
                        ),
                    )
                })?;
            }
            Ok(Arc::new(FileSource::open(p)?) as Arc<dyn ReadAt>)
        })
    }

    /// The first data track.
    pub fn data_track(&self) -> Option<&Track> {
        self.tracks.iter().find(|t| t.is_data())
    }

    /// The ISO 9660 volume of the first data track that has one (Joliet names when present).
    pub fn volume(&self) -> io::Result<Volume> {
        let mut last = io::Error::new(io::ErrorKind::NotFound, "the image has no data track");
        for t in self.tracks.iter().filter(|t| t.is_data()) {
            match Volume::open(t.clone()) {
                Ok(v) => return Ok(v),
                // A source that fetches on demand must get its chunks first.
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Err(e),
                Err(e) => last = e,
            }
        }
        Err(last)
    }
}
