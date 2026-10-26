//! The game's side of the contract: a manifest, and a handler for the frame's events.

use crate::event::{topics, Event, Origin};
use crate::manifest::Manifest;
use serde_json::{json, Value};

/// A game the frame can drive. Implement it in Rust, or use an adapter: [`crate::Process`]
/// for a game in another language, [`crate::Configured`] for one set up from `frame.json`.
pub trait Game {
    /// What the game supports. Read once, when the frame starts.
    fn manifest(&self) -> Manifest;

    /// An event for the game: from the frame (`game.start`, `config.set`) or the player
    /// (`input.press`). Answer and send the game's own events through `out`. Ignore topics the
    /// game does not know.
    fn on_event(&mut self, event: &Event, out: &mut Outbox);

    /// Called on each frame tick, so the game can send events no one asked for: a status
    /// change, a log line, a process's output.
    fn poll(&mut self, _out: &mut Outbox) {}
}

/// Where a game puts the events it sends.
#[derive(Debug, Default)]
pub struct Outbox {
    pub(crate) events: Vec<Event>,
}

impl Outbox {
    pub fn send(&mut self, topic: &str, data: Value) {
        self.events.push(Event::new(topic, Origin::Game, data));
    }

    /// `game.status`: "loading", "running", "paused", "stopped" or "failed", with a detail line.
    pub fn status(&mut self, state: &str, detail: &str) {
        self.send(topics::GAME_STATUS, json!({ "state": state, "detail": detail }));
    }

    /// A console line: "info", "warn" or "error".
    pub fn log(&mut self, level: &str, text: &str) {
        self.send(topics::LOG, json!({ "level": level, "text": text }));
    }

    pub fn take(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}
