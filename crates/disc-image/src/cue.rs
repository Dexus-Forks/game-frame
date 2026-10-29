//! CUE sheets: the FILE, TRACK, INDEX and PREGAP lines. Other commands (REM, TITLE,
//! PERFORMER, FLAGS, CATALOG, ISRC, POSTGAP, ...) are read past.

use std::io;

/// A position or length in frames (sectors): 75 frames a second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Msf(pub u32);

impl Msf {
    pub fn parse(s: &str) -> Option<Msf> {
        let mut it = s.split(':').map(|p| p.parse::<u32>().ok());
        let (m, sec, f) = (it.next()??, it.next()??, it.next()??);
        (it.next().is_none() && sec < 60 && f < 75).then_some(Msf((m * 60 + sec) * 75 + f))
    }
}

impl std::fmt::Display for Msf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.0 / 75 / 60,
            self.0 / 75 % 60,
            self.0 % 75
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueTrack {
    pub number: u8,
    /// As written: "MODE1/2352", "MODE2/2352", "AUDIO", ...
    pub mode: String,
    /// (index number, position in the file).
    pub indexes: Vec<(u8, Msf)>,
    /// Silence the image leaves out before INDEX 01.
    pub pregap: Msf,
}

impl CueTrack {
    /// Where the track starts in its file: INDEX 01, or the first index.
    pub fn start(&self) -> Option<Msf> {
        self.indexes
            .iter()
            .find(|i| i.0 == 1)
            .or(self.indexes.first())
            .map(|i| i.1)
    }
    /// The first frame of this track in the file (INDEX 00 when there is one).
    pub fn first(&self) -> Option<Msf> {
        self.indexes.iter().map(|i| i.1).min()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueFile {
    pub name: String,
    /// BINARY, MOTOROLA, WAVE, MP3, ...
    pub kind: String,
    pub tracks: Vec<CueTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cue {
    pub files: Vec<CueFile>,
}

fn bad(line: usize, msg: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("CUE line {line}: {msg}"),
    )
}

/// Split a line into words; a quoted word keeps its spaces.
fn words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = line.trim().chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' {
            chars.next();
            out.push(chars.by_ref().take_while(|&c| c != '"').collect());
        } else {
            let mut w = String::new();
            while let Some(&c) = chars.peek().filter(|c| !c.is_whitespace()) {
                w.push(c);
                chars.next();
            }
            out.push(w);
        }
    }
    out
}

pub fn parse_cue(text: &str) -> io::Result<Cue> {
    let mut cue = Cue::default();
    for (n, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let n = n + 1;
        let w = words(line);
        let Some(cmd) = w.first() else { continue };
        match cmd.to_ascii_uppercase().as_str() {
            "FILE" => {
                let name = w
                    .get(1)
                    .ok_or_else(|| bad(n, "FILE without a name"))?
                    .clone();
                let kind = w
                    .get(2)
                    .map(|k| k.to_ascii_uppercase())
                    .unwrap_or_else(|| "BINARY".into());
                cue.files.push(CueFile {
                    name,
                    kind,
                    tracks: Vec::new(),
                });
            }
            "TRACK" => {
                let file = cue
                    .files
                    .last_mut()
                    .ok_or_else(|| bad(n, "TRACK before FILE"))?;
                let number = w
                    .get(1)
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| bad(n, "TRACK without a number"))?;
                let mode = w
                    .get(2)
                    .ok_or_else(|| bad(n, "TRACK without a mode"))?
                    .to_ascii_uppercase();
                file.tracks.push(CueTrack {
                    number,
                    mode,
                    indexes: Vec::new(),
                    pregap: Msf(0),
                });
            }
            "INDEX" | "PREGAP" => {
                let track = cue
                    .files
                    .last_mut()
                    .and_then(|f| f.tracks.last_mut())
                    .ok_or_else(|| bad(n, "INDEX or PREGAP before TRACK"))?;
                if cmd.eq_ignore_ascii_case("PREGAP") {
                    track.pregap = w
                        .get(1)
                        .and_then(|s| Msf::parse(s))
                        .ok_or_else(|| bad(n, "PREGAP needs mm:ss:ff"))?;
                } else {
                    let i = w
                        .get(1)
                        .and_then(|s| s.parse().ok())
                        .ok_or_else(|| bad(n, "INDEX without a number"))?;
                    let at = w
                        .get(2)
                        .and_then(|s| Msf::parse(s))
                        .ok_or_else(|| bad(n, "INDEX needs mm:ss:ff"))?;
                    track.indexes.push((i, at));
                }
            }
            _ => {}
        }
    }
    if cue.files.iter().all(|f| f.tracks.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "CUE sheet has no tracks",
        ));
    }
    Ok(cue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_two_tracks_and_a_pregap() {
        let cue = parse_cue("FILE \"MY DISC.BIN\" BINARY\r\n  TRACK 01 MODE1/2352\r\n    INDEX 01 00:00:00\r\n  TRACK 02 AUDIO\r\n    PREGAP 00:02:00\r\n    INDEX 01 38:37:16\r\n").unwrap();
        assert_eq!(cue.files.len(), 1);
        assert_eq!(cue.files[0].name, "MY DISC.BIN");
        let t = &cue.files[0].tracks;
        assert_eq!(
            (t[0].number, t[0].mode.as_str(), t[0].start()),
            (1, "MODE1/2352", Some(Msf(0)))
        );
        assert_eq!((t[1].pregap, t[1].start()), (Msf(150), Some(Msf(173_791))));
        assert_eq!(Msf(173_791).to_string(), "38:37:16");
    }

    #[test]
    fn rejects_an_index_outside_a_track() {
        assert!(parse_cue("INDEX 01 00:00:00").is_err());
        assert!(parse_cue("REM nothing").is_err());
        assert_eq!(Msf::parse("00:60:00"), None);
    }
}
