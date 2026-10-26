//! Where the files are kept. Every store keeps the same three things for each section: the
//! files, a small record for each file (its size, time and card), and the section's state (the
//! order and which files are off).
//!
//! The methods return futures because the browser's storage answers later. The folder and memory
//! stores answer at once: natively, run a call with [`now`].

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub type LocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// One kept file, without its bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    /// When it was written: milliseconds since 1970.
    pub modified: f64,
    /// What the game says about it, when it said anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<Card>,
}

/// What the page shows for a file instead of its bare name: a save's town and date, a mod's
/// title and author. The game makes it ([`crate::Hooks::describe`]).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Card {
    pub title: String,
    pub detail: Option<String>,
    /// A small picture as a `data:` URL ([`crate::data_url`]).
    pub thumbnail: Option<String>,
}

/// A section's order and its switched-off files. Files not in `order` come after, by name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub order: Vec<String>,
    pub off: Vec<String>,
}

pub trait Store {
    /// The section's files, in no set order.
    fn list<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<Vec<Entry>>>;
    fn read<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<Vec<u8>>>;
    /// Keep `bytes` as `name`, replacing a file of that name. `card` is kept with it.
    fn write<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        bytes: &'a [u8],
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>>;
    /// Remove the file and its record. Removing a missing file is not an error.
    fn delete<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<()>>;
    fn state<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<State>>;
    fn set_state<'a>(&'a self, section: &'a str, state: &'a State) -> LocalFuture<'a, io::Result<()>>;
}

/// Run a store or library call that answers at once (the folder and memory stores).
///
/// Panics if the future waits: browser stores must be awaited.
pub fn now<F: Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let waker = Waker::noop();
    match f.as_mut().poll(&mut Context::from_waker(waker)) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("library::now: this store answers later; await it instead"),
    }
}

/// A file name the stores can keep: no folders, no leading dot, no control characters.
pub fn check_name(name: &str) -> io::Result<()> {
    let bad = name.is_empty()
        || name.starts_with('.')
        || name.len() > 200
        || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':'));
    if bad {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{name:?} is not a file name the library keeps")));
    }
    Ok(())
}

/// Any store, chosen at run time: a folder on the desktop, IndexedDB in the browser.
impl Store for Box<dyn Store> {
    fn list<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<Vec<Entry>>> {
        (**self).list(section)
    }
    fn read<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<Vec<u8>>> {
        (**self).read(section, name)
    }
    fn write<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        bytes: &'a [u8],
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>> {
        (**self).write(section, name, bytes, card)
    }
    fn delete<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<()>> {
        (**self).delete(section, name)
    }
    fn state<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<State>> {
        (**self).state(section)
    }
    fn set_state<'a>(&'a self, section: &'a str, state: &'a State) -> LocalFuture<'a, io::Result<()>> {
        (**self).set_state(section, state)
    }
}
