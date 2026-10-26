//! The player's library for a port: the game data, the saves, and the sections a game opts
//! into (mods, screenshots, recordings, its own). The game declares them in `library.json`; the
//! same file sets up the HTML page in GameFrame's `templates/library-page/`, which lets the
//! player add, order, switch off, download and delete the files before the game starts.
//!
//! The files are kept in a [`Store`]: [`Folder`] on the desktop, [`web::IndexedDb`] in the
//! browser (the page's own layout), [`Memory`] for tests. A [`Library`] puts a store and the
//! spec together and applies the section's rules: accepted names, one file or many, the load
//! order and the switched-off files.

mod library;
mod loaded;
mod memory;
mod spec;
mod store;

#[cfg(not(target_arch = "wasm32"))]
mod folder;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use folder::Folder;
pub use game_source;
pub use library::{data_url, Hooks, Library, Listed, Missing, NoHooks};
pub use loaded::Loaded;
pub use memory::Memory;
pub use spec::{IssueTarget, Kind, Link, Page, Panel, Required, Section, Spec};
pub use store::{check_name, now, Card, Entry, LocalFuture, State, Store};
