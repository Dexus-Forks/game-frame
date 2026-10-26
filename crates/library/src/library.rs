//! A store and the spec together: the rules of each section, and the hooks the game gives.

use crate::loaded::Loaded;
use crate::spec::{Kind, Section, Spec};
use crate::store::{Card, Entry, State, Store};
use game_source::GameFile;
use std::io;
use std::sync::Arc;

/// What the game adds to the library. Every method has a default, so a game implements only
/// what it needs.
pub trait Hooks {
    /// A card for a file the player adds or the game writes: a save's town and date, a mod's
    /// title. `None` shows the file name.
    ///
    /// For a file of at least `web::BIG` bytes (64 MiB) added in the browser
    /// ([`Library::add_file`]), `bytes` is only its first 1 MiB.
    fn describe(&self, _section: &Section, _name: &str, _bytes: &[u8]) -> Option<Card> {
        None
    }

    /// Refuse a file the game cannot use, with a sentence the page shows. The section's
    /// `accept` list is checked before this. For a big file added in the browser, `bytes` is
    /// only its first 1 MiB, as for [`Hooks::describe`].
    fn accept(&self, _section: &Section, _name: &str, _bytes: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

/// No hooks: bare names, and every file the section's `accept` list takes.
pub struct NoHooks;

impl Hooks for NoHooks {}

/// Any hooks, chosen at run time.
impl Hooks for Box<dyn Hooks> {
    fn describe(&self, section: &Section, name: &str, bytes: &[u8]) -> Option<Card> {
        (**self).describe(section, name, bytes)
    }
    fn accept(&self, section: &Section, name: &str, bytes: &[u8]) -> Result<(), String> {
        (**self).accept(section, name, bytes)
    }
}

/// A file the game needs and does not have.
#[derive(Debug, Clone, PartialEq)]
pub struct Missing {
    pub name: String,
    /// Set when a file of that name is kept but its size is wrong.
    pub found_size: Option<u64>,
    pub hint: Option<String>,
}

/// A kept file, with whether it is switched on.
#[derive(Debug, Clone, PartialEq)]
pub struct Listed {
    pub entry: Entry,
    pub on: bool,
}

pub struct Library<S: Store, H: Hooks = NoHooks> {
    spec: Spec,
    store: S,
    hooks: H,
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, msg)
}

/// The section's files in the page's order: the player's order for an ordered section, newest
/// first for saves, screenshots and recordings, and by name for the rest.
pub(crate) fn arrange(section: &Section, state: &State, mut entries: Vec<Entry>) -> Vec<Entry> {
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    if section.ordered() {
        let rank = |e: &Entry| state.order.iter().position(|n| *n == e.name).unwrap_or(usize::MAX);
        entries.sort_by_key(rank);
    } else if matches!(section.kind, Kind::Saves | Kind::Screenshots | Kind::Recordings) {
        entries.sort_by(|a, b| b.modified.total_cmp(&a.modified));
    }
    entries
}

impl<S: Store, H: Hooks> Library<S, H> {
    pub fn new(spec: Spec, store: S, hooks: H) -> Library<S, H> {
        Library { spec, store, hooks }
    }

    pub fn spec(&self) -> &Spec {
        &self.spec
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    fn section(&self, id: &str) -> io::Result<&Section> {
        self.spec.section(id).ok_or_else(|| invalid(format!("library.json has no section {id}")))
    }

    /// The game-data section's id.
    pub fn game_data_id(&self) -> &str {
        self.spec.of_kind(Kind::GameData).map_or("", |s| s.id.as_str())
    }

    /// The section's files in the page's order, each with whether it is switched on.
    pub async fn entries(&self, section: &str) -> io::Result<Vec<Listed>> {
        let s = self.section(section)?;
        let state = self.store.state(section).await?;
        let entries = arrange(s, &state, self.store.list(section).await?);
        Ok(entries
            .into_iter()
            .map(|entry| Listed { on: !s.toggles() || !state.off.contains(&entry.name), entry })
            .collect())
    }

    /// The names of the switched-on files, in order: the mods to load, first to last.
    pub async fn enabled(&self, section: &str) -> io::Result<Vec<String>> {
        Ok(self.entries(section).await?.into_iter().filter(|l| l.on).map(|l| l.entry.name).collect())
    }

    pub async fn read(&self, section: &str, name: &str) -> io::Result<Vec<u8>> {
        self.section(section)?;
        self.store.read(section, name).await
    }

    /// Keep a file the player added or the game wrote, after the section's checks and the
    /// game's. In a one-file section it replaces the file there. A new file of an ordered
    /// section goes last.
    pub async fn add(&self, section: &str, name: &str, bytes: &[u8]) -> io::Result<()> {
        let (s, card) = self.check(section, name, bytes)?;
        let old = self.store.list(section).await?;
        self.store.write(section, name, bytes, card).await?;
        self.added(s, name, &old).await
    }

    /// A file the player chose in the browser, kept as [`Library::add`] keeps bytes. A file of
    /// at least [`crate::web::BIG`] bytes is never read whole: the hooks see its first 1 MiB,
    /// and the store streams it ([`Store::write_file`]).
    #[cfg(target_arch = "wasm32")]
    pub async fn add_file(&self, section: &str, name: &str, file: &web_sys::File) -> io::Result<()> {
        let size = file.size() as u64;
        let head = crate::web::blob_bytes(file, 0, if size < crate::web::BIG { size } else { crate::web::HEAD }).await?;
        if size < crate::web::BIG {
            return self.add(section, name, &head).await;
        }
        let (s, card) = self.check(section, name, &head)?;
        let old = self.store.list(section).await?;
        self.store.write_file(section, name, file, card).await?;
        self.added(s, name, &old).await
    }

    /// A kept file as a browser `File`, to read in slices (such as a CD image for
    /// `game_source::web::open_files`).
    #[cfg(target_arch = "wasm32")]
    pub async fn file(&self, section: &str, name: &str) -> io::Result<web_sys::File> {
        self.section(section)?;
        self.store.file(section, name).await
    }

    /// The checks before a file is kept: the section, its accepted names, the game's own check;
    /// then the game's card for it.
    fn check(&self, section: &str, name: &str, bytes: &[u8]) -> io::Result<(&Section, Option<Card>)> {
        let s = self.section(section)?;
        if !s.accepts(name) {
            return Err(invalid(format!("{} takes {} files, not {name}", s.label, s.accept.join(", "))));
        }
        self.hooks.accept(s, name, bytes).map_err(invalid)?;
        Ok((s, self.hooks.describe(s, name, bytes)))
    }

    /// After a file is kept: a one-file section drops the files it had (`old`, listed before
    /// the write), and an ordered section puts a new file last.
    async fn added(&self, s: &Section, name: &str, old: &[Entry]) -> io::Result<()> {
        if !s.many() {
            for e in old.iter().filter(|e| e.name != name) {
                self.store.delete(&s.id, &e.name).await?;
            }
        }
        if s.ordered() {
            let mut state = self.store.state(&s.id).await?;
            if !state.order.iter().any(|n| n == name) {
                state.order.push(name.to_string());
                self.store.set_state(&s.id, &state).await?;
            }
        }
        Ok(())
    }

    pub async fn remove(&self, section: &str, name: &str) -> io::Result<()> {
        self.section(section)?;
        self.store.delete(section, name).await?;
        let mut state = self.store.state(section).await?;
        let before = (state.order.len(), state.off.len());
        state.order.retain(|n| n != name);
        state.off.retain(|n| n != name);
        if before != (state.order.len(), state.off.len()) {
            self.store.set_state(section, &state).await?;
        }
        Ok(())
    }

    /// Switch a file on or off.
    pub async fn set_on(&self, section: &str, name: &str, on: bool) -> io::Result<()> {
        self.section(section)?;
        let mut state = self.store.state(section).await?;
        state.off.retain(|n| n != name);
        if !on {
            state.off.push(name.to_string());
        }
        self.store.set_state(section, &state).await
    }

    /// Move a file to `index` in the order (0 loads first).
    pub async fn move_to(&self, section: &str, name: &str, index: usize) -> io::Result<()> {
        let names: Vec<String> = self.entries(section).await?.into_iter().map(|l| l.entry.name).collect();
        let mut order: Vec<String> = names.into_iter().filter(|n| n != name).collect();
        order.insert(index.min(order.len()), name.to_string());
        let mut state = self.store.state(section).await?;
        state.order = order;
        self.store.set_state(section, &state).await
    }

    /// The required game files that are not kept, or whose size is wrong. Empty when the game
    /// can start.
    pub async fn missing_game_data(&self) -> io::Result<Vec<Missing>> {
        let Some(s) = self.spec.of_kind(Kind::GameData) else { return Ok(Vec::new()) };
        let kept = self.store.list(&s.id).await?;
        Ok(s.required
            .iter()
            .filter_map(|r| {
                let found = kept.iter().find(|e| e.name.eq_ignore_ascii_case(&r.name));
                match (found, r.size) {
                    (Some(e), Some(size)) if e.size != size => {
                        Some(Missing { name: r.name.clone(), found_size: Some(e.size), hint: r.hint.clone() })
                    }
                    (Some(_), _) => None,
                    (None, _) => Some(Missing { name: r.name.clone(), found_size: None, hint: r.hint.clone() }),
                }
            })
            .collect())
    }

    /// Every kept game file, read whole, as a [`game_source::Source`].
    pub async fn game_data(&self) -> io::Result<Loaded> {
        let id = self.game_data_id().to_string();
        let mut files = Vec::new();
        for e in self.store.list(&id).await? {
            let bytes = self.store.read(&id, &e.name).await?;
            files.push(GameFile { path: e.name, bytes: Arc::new(bytes) });
        }
        Ok(Loaded::new(format!("kept in the {} library", self.spec.title), files))
    }
}

/// A `data:` URL for a card's thumbnail, such as `data_url("image/png", &png)`.
pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = format!("data:{mime};base64,");
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &b)| n | (b as u32) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}
