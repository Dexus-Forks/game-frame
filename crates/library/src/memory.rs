//! A store in memory, for tests and for a page that keeps nothing.

use crate::store::{check_name, Card, Entry, LocalFuture, State, Store};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io;

#[derive(Default)]
pub struct Memory {
    files: RefCell<BTreeMap<(String, String), (Vec<u8>, Entry)>>,
    states: RefCell<BTreeMap<String, State>>,
    /// The time of the next write: each write is one millisecond after the last.
    clock: Cell<f64>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory::default()
    }
}

fn ready<'a, T: 'a>(v: T) -> LocalFuture<'a, T> {
    Box::pin(std::future::ready(v))
}

fn missing(section: &str, name: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("{section}/{name}: not kept"))
}

impl Store for Memory {
    fn list<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<Vec<Entry>>> {
        let files = self.files.borrow();
        ready(Ok(files.iter().filter(|((s, _), _)| s == section).map(|(_, (_, e))| e.clone()).collect()))
    }

    fn read<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<Vec<u8>>> {
        let files = self.files.borrow();
        ready(files.get(&(section.into(), name.into())).map(|(b, _)| b.clone()).ok_or_else(|| missing(section, name)))
    }

    fn write<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        bytes: &'a [u8],
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>> {
        if let Err(e) = check_name(name) {
            return ready(Err(e));
        }
        self.clock.set(self.clock.get() + 1.0);
        let entry = Entry { name: name.into(), size: bytes.len() as u64, modified: self.clock.get(), card };
        self.files.borrow_mut().insert((section.into(), name.into()), (bytes.to_vec(), entry));
        ready(Ok(()))
    }

    fn delete<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<()>> {
        self.files.borrow_mut().remove(&(section.into(), name.into()));
        ready(Ok(()))
    }

    fn state<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<State>> {
        ready(Ok(self.states.borrow().get(section).cloned().unwrap_or_default()))
    }

    fn set_state<'a>(&'a self, section: &'a str, state: &'a State) -> LocalFuture<'a, io::Result<()>> {
        self.states.borrow_mut().insert(section.into(), state.clone());
        ready(Ok(()))
    }
}
