//! Events: everything that crosses between the frame, the game and the player.
//!
//! An event has a topic, `area.action` in lower case (`game.start`, `config.set`), and a JSON
//! value of data. Topics are plain strings, so a game can add its own (`pod.ghost.load`) and the
//! frame passes them on without knowing them. The well-known topics are in [`topics`], each
//! with the data it carries.
//!
//! On the wire (to a game in another language, or to a page's script) an event is one JSON
//! object: `{"topic": "game.start", "data": {...}, "from": "frame", "id": 7}`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Who sent an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// The frame: the launcher, the library, the settings.
    Frame,
    /// The game, through its adapter.
    Game,
    /// The player's input that the frame passes on (a key, a button on the console).
    Player,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub topic: String,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub data: Value,
    #[serde(rename = "from")]
    pub origin: Origin,
    /// Set by the bus: one more than the event before. A reply carries the id it answers in
    /// `data.re`.
    #[serde(default)]
    pub id: u64,
}

impl Event {
    pub fn new(topic: impl Into<String>, origin: Origin, data: Value) -> Event {
        Event { topic: topic.into(), data, origin, id: 0 }
    }

    /// Whether the topic matches a pattern: the whole topic (`game.start`), a prefix ending in
    /// `.*` (`game.*` takes `game.start` and `game.status`), or `*` for every event.
    pub fn matches(&self, pattern: &str) -> bool {
        matches(&self.topic, pattern)
    }

    /// A string field of the data.
    pub fn str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(Value::as_str)
    }
}

pub fn matches(topic: &str, pattern: &str) -> bool {
    match pattern.strip_suffix(".*") {
        _ if pattern == "*" => true,
        Some(prefix) => topic.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('.')),
        None => topic == pattern,
    }
}

/// Whether a topic is well formed: two or more lower-case words of letters, digits and `-`,
/// joined by dots.
pub fn valid_topic(topic: &str) -> bool {
    let parts: Vec<&str> = topic.split('.').collect();
    parts.len() >= 2
        && parts.iter().all(|p| {
            !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// The topics the frame knows, with their data. A game handles the ones its manifest's
/// capabilities name and may send any of them; the frame ignores none, it shows unknown
/// topics in the console.
pub mod topics {
    /// Frame to game, once, after the frame has read the manifest. Data: `{"settings": {...}}`,
    /// the saved values of every setting.
    pub const FRAME_READY: &str = "frame.ready";
    /// Either way: show the frame over a running game. The game sends it when the player asks
    /// for the launcher (a key, a menu item); the frame sends it when it opens by itself, so the
    /// game can pause. A game that stops or fails opens the frame too.
    pub const FRAME_OPEN: &str = "frame.open";
    /// Either way: hide the frame and go back to the game. The player's "Back to the game" sends
    /// it. A frame set to hide while the game runs also hides when the game reports `running`.
    pub const FRAME_CLOSE: &str = "frame.close";
    /// Either way: leave the program for the desktop. The frame stops a running game first, so
    /// the game can save. The player's "Exit to desktop" sends it, and so does a game that ended
    /// by itself when `frame.on-exit` is `"desktop"`. A game may send it from its own Quit item.
    pub const FRAME_EXIT: &str = "frame.exit";

    /// The frame's own preference changed, such as whether it opens at start. Data:
    /// `{"key": "frame.show-at-start", "value": false}`. Keys start with `frame.`; the game may
    /// ignore it.
    pub const FRAME_PREFERENCE: &str = "frame.preference";

    /// Frame to game: a setting changed. Data: `{"key": "fog", "value": true}`.
    pub const CONFIG_SET: &str = "config.set";
    /// Game to frame: the game changed a setting itself. Same data. The frame saves it.
    pub const CONFIG_CHANGED: &str = "config.changed";

    /// Game to frame: the options of a choice setting whose manifest says `"from": {"game":
    /// true}`, such as the tracks found on the disc. Data: `{"key": "race.track", "options":
    /// [{"value": "CANYON", "label": "Canyon"}]}`. A saved value that is not in the list becomes
    /// the first option.
    pub const CONFIG_OPTIONS: &str = "config.options";

    /// Frame to game: the player chose a renderer. Data: `{"renderer": "glide"}`.
    pub const RENDER_SELECT: &str = "render.select";
    /// Frame to game: the player chose a device in a slot other than graphics (contract 1.1).
    /// Data: `{"slot": "sound", "device": "none"}`.
    pub const DEVICE_SELECT: &str = "device.select";

    /// Frame to game: start. Data: `{"renderer": "glide", "devices": {"video": "glide",
    /// "sound": "directsound"}, "settings": {...}, "args": [...]}`. `frame.ready` carries
    /// `devices` too.
    pub const GAME_START: &str = "game.start";
    /// Frame to game: stop, as the game's own quit does.
    pub const GAME_STOP: &str = "game.stop";
    /// Game to frame: what the game is doing. Data: `{"state": "loading" | "running" |
    /// "paused" | "stopped" | "failed", "detail": "..."}`.
    pub const GAME_STATUS: &str = "game.status";

    /// Frame to game: a control pressed or released on the frame's console or a test.
    /// Data: `{"control": "start", "down": true}`. Control names are the game's, listed in its
    /// manifest's `controls`.
    pub const INPUT_PRESS: &str = "input.press";

    /// Frame to game: the player added, removed or reordered files in a library section.
    /// Data: `{"section": "mods"}`. The game reads the library again.
    pub const LIBRARY_CHANGED: &str = "library.changed";

    /// Game to frame: the game reached a place the port has not ported, or crashed: one
    /// `gameframe_report::Report` as JSON, the first time at each place and again with a new
    /// count. In Production it carries `issue`, a link to a filled-in GitHub issue.
    pub const REPORT_NEW: &str = "report.new";

    /// Either way: a line for the console. Data: `{"level": "info" | "warn" | "error",
    /// "text": "..."}`.
    pub const LOG: &str = "log.message";
}
