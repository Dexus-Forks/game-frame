//! The contract between a port and the frame around it.
//!
//! A port is a game core. The frame is everything around it the player sees before and beside
//! the game: the launcher, the library of game files and saves, the settings, the renderer
//! choice, help pages, a console. The two talk only through [`Event`]s, so either side can grow
//! without the other changing:
//!
//! - the game says what it supports in a [`Manifest`]: capabilities, renderers with a
//!   [`Grade`] of how complete each is, settings, controls;
//! - the frame sends [`topics`] such as `game.start` and `config.set`, and the game answers with
//!   `game.status`, `config.changed` and log lines, or topics of its own;
//! - a [`Session`] keeps the state both sides agree on (settings, renderer, status, the log).
//!
//! A game joins in one of three ways: a Rust type that implements [`Game`]; a program in any
//! language that speaks JSON lines ([`Process`], and [`Link`] for the program's own end; [`Respawn`]
//! for a program that runs one game and ends); or a `frame.json` with a manifest and a start
//! command ([`Configured`]). [`Demo`] stands in for a game, so the frame runs with none.
//!
//! The views are in `gameframe-frame-ui`. This crate has no UI and builds for the browser.

mod demo;
mod event;
mod game;
mod manifest;
#[cfg(not(target_arch = "wasm32"))]
mod configured;
#[cfg(not(target_arch = "wasm32"))]
mod files;
#[cfg(not(target_arch = "wasm32"))]
mod link;
#[cfg(not(target_arch = "wasm32"))]
mod process;
#[cfg(not(target_arch = "wasm32"))]
pub mod respawn;
mod session;
mod settings;

pub use demo::{Demo, DEMO_PRESS_TICKS};
pub use event::{matches, topics, valid_topic, Event, Origin};
pub use game::{Game, Outbox};
pub use manifest::{capabilities, Choice, Control, Grade, Kind, Manifest, OptionsFrom, Renderer, Setting, CONTRACT};
#[cfg(not(target_arch = "wasm32"))]
pub use configured::{Configured, FrameFile, StartCommand};
#[cfg(not(target_arch = "wasm32"))]
pub use files::{list_files, list_image_files};
#[cfg(not(target_arch = "wasm32"))]
pub use process::Process;
#[cfg(not(target_arch = "wasm32"))]
pub use link::Link;
#[cfg(not(target_arch = "wasm32"))]
pub use respawn::Respawn;
pub use session::{Session, LOG_LENGTH};
#[cfg(not(target_arch = "wasm32"))]
pub use settings::file as settings_file;
pub use settings::{Settings, ON_EXIT, PRESET, SHOW_AT_START};
