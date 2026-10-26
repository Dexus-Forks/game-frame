//! ISO 9660 (ECMA-119): the volume descriptors from sector 16, the directory tree, and the
//! files' bytes. When a supplementary descriptor carries a Joliet escape sequence, its tree
//! (UCS-2 names) is used for the names.

use crate::SECTOR;
use crate::disc::Track;
use std::collections::HashSet;
use std::io;

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b)
        .trim_end_matches([' ', '\0'])
        .to_string()
}

fn ucs2(b: &[u8]) -> String {
    let units: Vec<u16> = b
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
        .trim_end_matches([' ', '\0'])
        .to_string()
}

/// A date and time with its offset from UTC, in quarter hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Timestamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub utc_offset_quarters: i8,
}

impl Timestamp {
    /// The 7-byte form of a directory record.
    fn from_record(b: &[u8]) -> Option<Timestamp> {
        (b[1] != 0).then(|| Timestamp {
            year: 1900 + b[0] as u16,
            month: b[1],
            day: b[2],
            hour: b[3],
            minute: b[4],
            second: b[5],
            utc_offset_quarters: b[6] as i8,
        })
    }

    /// The 17-byte digits form of a volume descriptor ("1997011313531900" and an offset).
    fn from_digits(b: &[u8]) -> Option<Timestamp> {
        let n = |r: std::ops::Range<usize>| std::str::from_utf8(&b[r]).ok()?.parse::<u16>().ok();
        let year = n(0..4).filter(|&y| y > 0)?;
        Some(Timestamp {
            year,
            month: n(4..6)? as u8,
            day: n(6..8)? as u8,
            hour: n(8..10)? as u8,
            minute: n(10..12)? as u8,
            second: n(12..14)? as u8,
            utc_offset_quarters: b[16] as i8,
        })
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let q = self.utc_offset_quarters as i32 * 15;
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}:{:02}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            if q < 0 { '-' } else { '+' },
            q.abs() / 60,
            q.abs() % 60
        )
    }
}

/// The identifiers and dates of the primary volume descriptor.
#[derive(Debug, Clone, Default)]
pub struct VolumeInfo {
    pub system: String,
    pub volume: String,
    pub volume_set: String,
    pub publisher: String,
    pub preparer: String,
    pub application: String,
    /// The volume's size in logical blocks.
    pub blocks: u32,
    pub created: Option<Timestamp>,
    pub modified: Option<Timestamp>,
    /// The names come from a Joliet tree.
    pub joliet: bool,
}

/// A file or directory of the volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// From the root, '/' between names, no version suffix ("DATA/BINARY/MIMG/62.IMG").
    pub path: String,
    pub size: u64,
    /// The disc address of its first sector.
    pub lba: u32,
    pub date: Option<Timestamp>,
    pub directory: bool,
    /// The record's "existence" flag: the file is hidden from directory listings.
    pub hidden: bool,
}

/// An ISO 9660 volume on a data track.
pub struct Volume {
    track: Track,
    pub info: VolumeInfo,
    entries: Vec<Entry>,
}

impl Volume {
    /// Read the volume descriptors from sector 16 of `track` and the whole directory tree.
    pub fn open(track: Track) -> io::Result<Volume> {
        let (mut primary, mut joliet) = (None, None);
        for i in 16..16 + 64 {
            let d = track.read_user(i, 1)?;
            if &d[1..6] != b"CD001" {
                break;
            }
            match d[0] {
                1 if primary.is_none() => primary = Some(d),
                2 if joliet.is_none()
                    && [b"%/@", b"%/C", b"%/E"].iter().any(|e| &d[88..91] == *e) =>
                {
                    joliet = Some(d)
                }
                255 => break,
                _ => {}
            }
        }
        let pvd = primary.ok_or_else(|| {
            invalid(format!(
                "track {}: no ISO 9660 primary volume descriptor at sector 16",
                track.number
            ))
        })?;
        let block = u16::from_le_bytes([pvd[128], pvd[129]]);
        if block as usize != SECTOR {
            return Err(invalid(format!(
                "logical block size {block} is not supported"
            )));
        }
        let info = VolumeInfo {
            system: text(&pvd[8..40]),
            volume: text(&pvd[40..72]),
            volume_set: text(&pvd[190..318]),
            publisher: text(&pvd[318..446]),
            preparer: text(&pvd[446..574]),
            application: text(&pvd[574..702]),
            blocks: le32(&pvd, 80),
            created: Timestamp::from_digits(&pvd[813..830]),
            modified: Timestamp::from_digits(&pvd[830..847]),
            joliet: joliet.is_some(),
        };
        let mut v = Volume {
            track,
            info,
            entries: Vec::new(),
        };
        let root = joliet.as_ref().unwrap_or(&pvd);
        let mut seen = HashSet::new();
        v.walk(&root[156..190], "", joliet.is_some(), &mut seen)?;
        Ok(v)
    }

    fn walk(
        &mut self,
        record: &[u8],
        prefix: &str,
        joliet: bool,
        seen: &mut HashSet<u32>,
    ) -> io::Result<()> {
        let (lba, size) = (le32(record, 2), le32(record, 10));
        if !seen.insert(lba) {
            return Ok(()); // a directory listed twice: never loop
        }
        let data = self.read_extent(lba, size as u64)?;
        let mut subdirs = Vec::new();
        for sector in data.chunks(SECTOR) {
            let mut at = 0;
            while at < sector.len() {
                let len = sector[at] as usize;
                if len == 0 {
                    break; // the rest of the sector is padding
                }
                let r = sector
                    .get(at..at + len)
                    .filter(|r| r.len() >= 34)
                    .ok_or_else(|| {
                        invalid(format!("directory at {lba}: a record runs past its sector"))
                    })?;
                at += len;
                let name_len = r[32] as usize;
                let raw = r.get(33..33 + name_len).ok_or_else(|| {
                    invalid(format!("directory at {lba}: a name runs past its record"))
                })?;
                if raw == [0] || raw == [1] {
                    continue; // "." and ".."
                }
                let name = if joliet { ucs2(raw) } else { text(raw) };
                let name = name.split(';').next().unwrap_or_default();
                let name = name.strip_suffix('.').unwrap_or(name);
                let flags = r[25];
                if flags & 0x80 != 0 {
                    return Err(invalid(format!(
                        "{prefix}{name}: files in several extents are not supported"
                    )));
                }
                let entry = Entry {
                    path: format!("{prefix}{name}"),
                    size: le32(r, 10) as u64,
                    lba: le32(r, 2),
                    date: Timestamp::from_record(&r[18..25]),
                    directory: flags & 0x02 != 0,
                    hidden: flags & 0x01 != 0,
                };
                if entry.directory {
                    subdirs.push((r.to_vec(), format!("{}/", entry.path)));
                }
                self.entries.push(entry);
            }
        }
        for (r, p) in subdirs {
            self.walk(&r, &p, joliet, seen)?;
        }
        Ok(())
    }

    /// `len` bytes of the extent at disc address `lba`.
    fn read_extent(&self, lba: u32, len: u64) -> io::Result<Vec<u8>> {
        self.read_range_at(lba, len, 0, len)
    }

    fn read_range_at(&self, lba: u32, size: u64, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        let end = offset
            .checked_add(len)
            .filter(|&e| e <= size)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("range {offset}+{len} is past the end of a {size}-byte file"),
                )
            })?;
        if len == 0 {
            return Ok(Vec::new());
        }
        let rel = lba
            .checked_sub(self.track.start)
            .ok_or_else(|| invalid(format!("sector {lba} is before the track")))?;
        let first = offset / SECTOR as u64;
        let last = (end - 1) / SECTOR as u64;
        let bytes = self
            .track
            .read_user(rel + first as u32, (last - first + 1) as u32)?;
        let skip = (offset - first * SECTOR as u64) as usize;
        Ok(bytes[skip..skip + len as usize].to_vec())
    }

    /// Every file and directory, parents before their children.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The files only.
    pub fn files(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.directory)
    }

    /// The entry at `path` ('/' or '\' between names), case-blind.
    pub fn find(&self, path: &str) -> Option<&Entry> {
        let want = path.trim_start_matches(['/', '\\']).replace('\\', "/");
        self.entries
            .iter()
            .find(|e| e.path.eq_ignore_ascii_case(&want))
    }

    /// `len` bytes of a file from `offset`: only the sectors that hold them are read.
    pub fn read_range(&self, entry: &Entry, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        self.read_range_at(entry.lba, entry.size, offset, len)
    }

    /// The whole file.
    pub fn read(&self, entry: &Entry) -> io::Result<Vec<u8>> {
        self.read_range(entry, 0, entry.size)
    }

    /// The track the volume is on.
    pub fn track(&self) -> &Track {
        &self.track
    }
}
