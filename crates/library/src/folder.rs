//! A store in folders, for the desktop: `ROOT/SECTION/NAME` holds each file, so the player can
//! open the folder and copy saves in and out by hand. Each section folder also holds
//! `.library/state.json` and `.library/cards/NAME.json`; names that start with a dot are never
//! listed.

use crate::spec::Section;
use crate::store::{check_name, Card, Entry, LocalFuture, State, Store};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct Folder {
    root: PathBuf,
}

impl Folder {
    /// The store at `root`. Folders are made when the first file is written.
    pub fn new(root: impl Into<PathBuf>) -> Folder {
        Folder { root: root.into() }
    }

    /// The folder of a section, such as the game-data folder to open with
    /// `game_source::Folder::new`.
    pub fn section_dir(&self, section: &str) -> PathBuf {
        self.root.join(section)
    }

    /// Fill a section from a folder the player already has, such as the game's own folder or
    /// its saves, by hard links: nothing is copied, and the store counts a hard link as its own
    /// file (a symbolic link it would not). Links each file the section accepts and `keep`
    /// takes, unless the section has what it needs already: every required file, or with none
    /// required, any file at all. Names already in the section are left alone. Returns how many
    /// files were linked; a missing `from` links none. Both folders must be on one volume.
    pub fn link_in(&self, section: &Section, from: &Path, keep: impl Fn(&str) -> bool) -> io::Result<usize> {
        let have: Vec<String> = self.list_now(&section.id)?.into_iter().map(|e| e.name).collect();
        let has = |name: &str| have.iter().any(|h| h.eq_ignore_ascii_case(name));
        let ready = if section.required.is_empty() {
            have.iter().any(|n| section.accepts(n))
        } else {
            section.required.iter().all(|r| has(&r.name))
        };
        if ready || !from.is_dir() {
            return Ok(0);
        }
        let mut names: Vec<String> = Vec::new();
        for item in fs::read_dir(from)? {
            let item = item?;
            let name = item.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') && item.path().is_file() && section.accepts(&name) && keep(&name) {
                names.push(name);
            }
        }
        names.sort();
        let dir = self.section_dir(&section.id);
        fs::create_dir_all(&dir)?;
        let mut linked = 0;
        for name in names.iter().filter(|n| !has(n)) {
            check_name(name)?;
            fs::hard_link(from.join(name), dir.join(name))
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", from.join(name).display())))?;
            linked += 1;
        }
        Ok(linked)
    }

    fn hidden(&self, section: &str) -> PathBuf {
        self.section_dir(section).join(".library")
    }

    fn card_path(&self, section: &str, name: &str) -> PathBuf {
        self.hidden(section).join("cards").join(format!("{name}.json"))
    }
}

fn ready<'a, T: 'a>(v: T) -> LocalFuture<'a, T> {
    Box::pin(std::future::ready(v))
}

fn modified(path: &Path) -> f64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0.0, |d| d.as_millis() as f64)
}

/// Write through a temporary file, so a crash leaves the old file or the new one.
fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".library-tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

impl Folder {
    fn list_now(&self, section: &str) -> io::Result<Vec<Entry>> {
        let dir = self.section_dir(section);
        let read = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        for item in read {
            let item = item?;
            let name = item.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name.ends_with(".library-tmp") || !item.file_type()?.is_file() {
                continue;
            }
            let card = fs::read(self.card_path(section, &name)).ok().and_then(|b| serde_json::from_slice(&b).ok());
            out.push(Entry { size: item.metadata()?.len(), modified: modified(&item.path()), name, card });
        }
        Ok(out)
    }

    fn write_now(&self, section: &str, name: &str, bytes: &[u8], card: Option<Card>) -> io::Result<()> {
        check_name(name)?;
        replace(&self.section_dir(section).join(name), bytes)?;
        let card_path = self.card_path(section, name);
        match card {
            Some(c) => replace(&card_path, &serde_json::to_vec(&c)?),
            None => match fs::remove_file(&card_path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            },
        }
    }

    fn delete_now(&self, section: &str, name: &str) -> io::Result<()> {
        check_name(name)?;
        for path in [self.section_dir(section).join(name), self.card_path(section, name)] {
            match fs::remove_file(path) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
                _ => {}
            }
        }
        Ok(())
    }

    fn state_now(&self, section: &str) -> io::Result<State> {
        match fs::read(self.hidden(section).join("state.json")) {
            Ok(b) => serde_json::from_slice(&b).map_err(io::Error::other),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(State::default()),
            Err(e) => Err(e),
        }
    }
}

impl Store for Folder {
    fn list<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<Vec<Entry>>> {
        ready(self.list_now(section))
    }

    fn read<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<Vec<u8>>> {
        ready(check_name(name).and_then(|_| fs::read(self.section_dir(section).join(name))))
    }

    fn write<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        bytes: &'a [u8],
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>> {
        ready(self.write_now(section, name, bytes, card))
    }

    fn delete<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<()>> {
        ready(self.delete_now(section, name))
    }

    fn state<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<State>> {
        ready(self.state_now(section))
    }

    fn set_state<'a>(&'a self, section: &'a str, state: &'a State) -> LocalFuture<'a, io::Result<()>> {
        let path = self.hidden(section).join("state.json");
        ready(serde_json::to_vec_pretty(state).map_err(io::Error::other).and_then(|b| replace(&path, &b)))
    }
}
