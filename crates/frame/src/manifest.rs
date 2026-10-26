//! What a game tells the frame about itself: its name, what it can do, its renderers and other
//! devices with how complete each is, its settings and its controls. The frame draws only what the manifest
//! names, so a game opts into each feature by listing it.
//!
//! A game written in Rust returns a [`Manifest`] from [`crate::Game::manifest`]. A game in
//! another language prints it as JSON, and a game with no code gives it in `frame.json`.

use serde::{Deserialize, Serialize};
use crate::device::{slots, Slot};
use serde_json::Value;

/// The version of the contract in this crate. The frame refuses a manifest with a larger
/// major number and accepts a smaller one. A minor version only adds fields, and a frame
/// ignores fields it does not know, so a 1.1 game still opens in a 1.0 frame.
pub const CONTRACT: &str = "1.1";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    /// The contract version the game was written for, such as "1.0".
    pub contract: String,
    pub name: String,
    pub version: String,
    /// What the game handles. The well-known ones are in [`capabilities`]; a game can name its
    /// own. The frame shows a feature only when its capability is listed.
    pub capabilities: Vec<String>,
    /// The ways the game can draw, such as one per 3D card the original supported: the
    /// graphics slot ([`crate::device`]).
    pub renderers: Vec<Renderer>,
    /// The other hardware slots, such as the sound card, each with its devices (contract 1.1).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<Slot>,
    /// Settings the player can change in the frame.
    pub settings: Vec<Setting>,
    /// Controls the console can press, by the game's own names ("start", "fire", "left").
    pub controls: Vec<Control>,
}

/// The capabilities the frame knows.
pub mod capabilities {
    /// The game starts and stops on `game.start` and `game.stop`.
    pub const START: &str = "start";
    /// The game has more than one renderer and takes `render.select`.
    pub const RENDERERS: &str = "renderers";
    /// The game reads its settings from `frame.ready` and `config.set`.
    pub const SETTINGS: &str = "settings";
    /// The game takes `input.press` from the frame's console.
    pub const INPUT: &str = "input";
    /// The game reads the player's library and takes `library.changed`.
    pub const LIBRARY: &str = "library";
    /// The game has slots besides the renderer (`devices`) and takes `device.select`.
    pub const DEVICES: &str = "devices";
}

/// How complete a renderer is, for the player choosing one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    /// Not started: listed so the player knows it is planned.
    Missing,
    /// Draws something; expect gaps.
    Experimental,
    /// Most of the game draws; known gaps are listed.
    Partial,
    /// Everything the game uses draws; some details differ.
    #[default]
    Good,
    /// Checked against the original, frame for frame.
    Exact,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Renderer {
    pub id: String,
    pub label: String,
    /// One line for the player: the card or API it stands for.
    pub detail: Option<String>,
    pub grade: Grade,
    /// How much of the API is done, such as 131 of 156 Glide calls.
    pub done: Option<u32>,
    pub total: Option<u32>,
    /// Known gaps, one sentence each.
    pub notes: Vec<String>,
    /// The renderer the frame selects first.
    pub default: bool,
    /// For a device not ported yet: the id of the device in the same slot that stands in for
    /// it, so the game still starts. The kernel runs that one and reports the gap
    /// (`gameframe_report::unported!`), so a start on any listed device works end to end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stand_in: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Kind {
    /// On or off.
    Toggle,
    /// One of the options. With `from`, the options are filled in later: from the game's
    /// files, or by the game itself with `config.options`. They may then be empty here.
    Choice {
        #[serde(default)]
        options: Vec<Choice>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<OptionsFrom>,
    },
    /// A number from `min` to `max`.
    Range { min: f64, max: f64, step: f64 },
    /// Free text.
    Text,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

/// Where a choice's options come from when the manifest cannot list them, such as the tracks
/// and cars on the game's disc.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OptionsFrom {
    /// The files that match a pattern, from the game's folder: `DATA/TRACKS/*.TRK`. Each file
    /// is one option: its name without the extension is the value and the label. Only the last
    /// part of the path may hold `*` and `?`, and letter case does not matter. A `frame.json`
    /// game fills these when it loads (`Configured::load`).
    pub files: Option<String>,
    /// The game sends the options itself, with `config.options`.
    pub game: bool,
}

/// A setting. Its `type` (toggle, choice, range, text) and that type's fields sit beside the
/// others in JSON. Unknown fields are ignored here, since serde cannot refuse them beside a
/// flattened type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    /// The key in `config.set` and the saved settings, such as "fog" or "video.scale".
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub help: Option<String>,
    /// A group heading, such as "Video" or "Sound".
    #[serde(default)]
    pub group: Option<String>,
    #[serde(flatten)]
    pub kind: Kind,
    pub default: Value,
    /// The setting takes effect only when the game starts again.
    #[serde(default)]
    pub restart: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Control {
    pub name: String,
    pub label: String,
    /// The key the console shows for it, such as "Enter".
    pub key: Option<String>,
}

impl Manifest {
    pub fn from_json(text: &str) -> Result<Manifest, String> {
        let m: Manifest = serde_json::from_str(text).map_err(|e| format!("manifest: {e}"))?;
        m.check()?;
        Ok(m)
    }

    pub fn can(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// The renderer to select first: the one marked default, else the best graded.
    pub fn default_renderer(&self) -> Option<&Renderer> {
        self.renderers.iter().find(|r| r.default).or_else(|| self.renderers.iter().max_by_key(|r| r.grade))
    }

    /// Every slot in the order the frame shows them: the graphics slot from `renderers` first,
    /// when it has any, then `devices`.
    pub fn slots(&self) -> Vec<Slot> {
        let video = (!self.renderers.is_empty()).then(|| Slot {
            slot: slots::VIDEO.into(),
            label: "Graphics".into(),
            options: self.renderers.clone(),
        });
        video.into_iter().chain(self.devices.iter().cloned()).collect()
    }

    /// The slot `slot`, the graphics slot included.
    pub fn slot(&self, slot: &str) -> Option<Slot> {
        self.slots().into_iter().find(|s| s.slot == slot)
    }

    /// Whether the game can start with `id` in `slot`: it is listed there and not graded
    /// `missing`. The error is a sentence for the player, for `game.status` `failed`.
    pub fn playable(&self, slot: &str, id: &str) -> Result<(), String> {
        let s = self.slot(slot).ok_or_else(|| format!("{} has no {slot} slot", self.name))?;
        match s.options.iter().find(|d| d.id == id) {
            None => Err(format!("{} has no {} called {id}", self.name, s.label.to_lowercase())),
            Some(d) if d.grade == Grade::Missing && d.stand_in.is_none() => Err(format!(
                "{}{} is not ported yet; choose another in {}",
                d.label,
                d.detail.as_ref().map(|t| format!(" ({t})")).unwrap_or_default(),
                s.label
            )),
            Some(_) => Ok(()),
        }
    }

    /// The device a kernel runs for `id` in `slot`: `id` itself, or the device standing in for
    /// it. `Some` only for a stand-in; report the gap then.
    pub fn stand_in(&self, slot: &str, id: &str) -> Option<String> {
        self.slot(slot)?.options.into_iter().find(|d| d.id == id)?.stand_in
    }

    pub fn setting(&self, key: &str) -> Option<&Setting> {
        self.settings.iter().find(|s| s.key == key)
    }

    /// The contract version, the ids and the settings' defaults.
    pub fn check(&self) -> Result<(), String> {
        let major = |v: &str| v.split('.').next().and_then(|m| m.parse::<u32>().ok());
        match (major(&self.contract), major(CONTRACT)) {
            (Some(theirs), Some(ours)) if theirs <= ours => {}
            _ => return Err(format!("manifest: contract {:?} is not one this frame ({CONTRACT}) reads", self.contract)),
        }
        let mut seen = std::collections::HashSet::new();
        for r in &self.renderers {
            if r.id.is_empty() || !seen.insert(("renderer", r.id.clone())) {
                return Err(format!("manifest: renderer id {:?} is empty or repeated", r.id));
            }
        }
        for slot in &self.devices {
            let video = slot.slot == slots::VIDEO && !self.renderers.is_empty();
            if slot.slot.is_empty() || video || !seen.insert(("slot", slot.slot.clone())) {
                return Err(format!("manifest: device slot {:?} is empty, repeated or the renderers'", slot.slot));
            }
            let mut ids = std::collections::HashSet::new();
            if slot.options.iter().any(|d| d.id.is_empty() || !ids.insert(d.id.clone())) {
                return Err(format!("manifest: a device id in the {} slot is empty or repeated", slot.slot));
            }
        }
        for slot in self.slots() {
            for d in &slot.options {
                let stand_in = d.stand_in.as_deref().and_then(|s| slot.options.iter().find(|o| o.id == s));
                if d.stand_in.is_some() && !stand_in.is_some_and(|o| o.stand_in.is_none() && o.id != d.id) {
                    return Err(format!("manifest: {} stands in with a device its slot lacks", d.id));
                }
            }
        }
        for s in &self.settings {
            if s.key.is_empty() || !seen.insert(("setting", s.key.clone())) {
                return Err(format!("manifest: setting key {:?} is empty or repeated", s.key));
            }
            let later = matches!(&s.kind, Kind::Choice { from: Some(_), .. });
            if later && !s.default.is_string() || !later && !s.accepts(&s.default) {
                return Err(format!("manifest: setting {} has a default its kind does not take", s.key));
            }
        }
        Ok(())
    }
}

impl Setting {
    /// Fill a choice's options, as `config.options` and a `frame.json` game's file lists do.
    /// False for a setting that is not a choice.
    pub fn fill(&mut self, list: Vec<Choice>) -> bool {
        match &mut self.kind {
            Kind::Choice { options, .. } => {
                *options = list;
                true
            }
            _ => false,
        }
    }

    /// Whether a value fits the setting: a bool for a toggle, one of the options, a number in
    /// range, a string.
    pub fn accepts(&self, v: &Value) -> bool {
        match &self.kind {
            Kind::Toggle => v.is_boolean(),
            // A list not filled in yet takes any text, so a saved choice waits for its list.
            Kind::Choice { options, from: Some(_) } if options.is_empty() => v.is_string(),
            Kind::Choice { options, .. } => v.as_str().is_some_and(|s| options.iter().any(|o| o.value == s)),
            Kind::Range { min, max, .. } => v.as_f64().is_some_and(|n| n >= *min && n <= *max),
            Kind::Text => v.is_string(),
        }
    }
}
