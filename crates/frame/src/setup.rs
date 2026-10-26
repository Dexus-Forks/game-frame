//! The devices a start runs on, worked out the same way in every kernel.
//!
//! The player picks one device per slot in the frame (`render.select`, `device.select`), and
//! `frame.ready` and `game.start` carry the picks. [`Held`] keeps the latest pick per slot from
//! those events. [`Manifest::resolve`] turns them into a [`Setup`] for one start: per slot the
//! device asked for, the device used (a stand-in for one not ported yet), and the [`Gap`]s.
//! [`Setup::report_gaps`] writes one `gameframe_report` report per gap.
//!
//! ```
//! # use gameframe_frame::{Manifest, setup::Held};
//! # use serde_json::json;
//! let manifest = Manifest::default(); // the game's own
//! let mut held = Held::default();      // in the kernel; `held.note(&event)` on every event
//! let start = json!({ "devices": {} }); // `game.start`'s data
//! let setup = manifest.resolve(&start, &held).unwrap(); // `Err` is the reason for `failed`
//! assert!(setup.gaps.is_empty());
//! ```

use crate::device::{chosen, slots};
use crate::event::{topics, Event};
use crate::manifest::Manifest;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// The latest device the frame picked in each slot, from `frame.ready`, `render.select` and
/// `device.select`. A kernel keeps one and passes every event to [`Held::note`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Held(BTreeMap<String, String>);

impl Held {
    pub fn note(&mut self, e: &Event) {
        match e.topic.as_str() {
            topics::FRAME_READY => {
                if let Some(Value::Object(devices)) = e.data.get("devices") {
                    for (slot, id) in devices {
                        if let Some(id) = id.as_str() {
                            self.set(slot, id);
                        }
                    }
                }
                if let Some(id) = e.str("renderer") {
                    self.0.entry(slots::VIDEO.into()).or_insert_with(|| id.into());
                }
            }
            topics::RENDER_SELECT => {
                if let Some(id) = e.str("renderer") {
                    self.set(slots::VIDEO, id);
                }
            }
            topics::DEVICE_SELECT => {
                if let (Some(slot), Some(id)) = (e.str("slot"), e.str("device")) {
                    self.set(slot, id);
                }
            }
            _ => {}
        }
    }

    pub fn set(&mut self, slot: &str, id: &str) {
        if !id.is_empty() {
            self.0.insert(slot.into(), id.into());
        }
    }

    pub fn get(&self, slot: &str) -> Option<&str> {
        self.0.get(slot).map(String::as_str)
    }
}

/// A device asked for and not ported yet: `used` runs in its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    pub slot: String,
    pub asked: String,
    pub used: String,
}

/// The devices of one start.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Setup {
    /// Per slot, the device the player asked for.
    pub asked: BTreeMap<String, String>,
    /// Per slot, the device the kernel runs: the one asked for, or its stand-in.
    pub used: BTreeMap<String, String>,
    pub gaps: Vec<Gap>,
}

impl Setup {
    /// The device the kernel runs in `slot`.
    pub fn used(&self, slot: &str) -> Option<&str> {
        self.used.get(slot).map(String::as_str)
    }

    /// The devices asked for, as `game.start` carries them: `{"video": "glide", "sound": "none"}`.
    /// The reports record this ([`crate::Link::send_reports`]).
    pub fn asked_json(&self) -> Value {
        Value::Object(self.asked.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<Map<_, _>>())
    }

    /// One `unported` report per gap, at the caller's place in the port. The original is the
    /// device's `detail` (its build or library), else its id; the context holds the slot, the
    /// device asked for and the device used.
    #[track_caller]
    pub fn report_gaps(&self, manifest: &Manifest) {
        let at = std::panic::Location::caller();
        for g in &self.gaps {
            let device = manifest.slot(&g.slot).and_then(|s| s.options.into_iter().find(|d| d.id == g.asked));
            let original = device.and_then(|d| d.detail).unwrap_or_else(|| g.asked.clone());
            let what = format!("the {} device {} is not ported; {} runs in its place", g.slot, g.asked, g.used);
            let context = vec![("slot", json!(g.slot)), ("asked", json!(g.asked)), ("used", json!(g.used))];
            gameframe_report::hit(gameframe_report::Kind::Unported, &what, Some(&original), (at.file(), at.line(), at.column()), context);
        }
    }
}

impl Manifest {
    /// The devices for a start. In each slot the device is `game.start`'s pick ([`chosen`]),
    /// else the one `held` (the last pick the frame sent), else the slot's default. A slot with
    /// none of the three is left out. Each one must be [`Manifest::playable`]; one not ported
    /// yet runs on its stand-in and is a [`Gap`]. `Err` is a sentence for `game.status` `failed`.
    pub fn resolve(&self, start: &Value, held: &Held) -> Result<Setup, String> {
        let mut setup = Setup::default();
        for slot in self.slots() {
            let asked = chosen(start, &slot.slot)
                .filter(|id| !id.is_empty())
                .or_else(|| held.get(&slot.slot))
                .map(str::to_owned)
                .or_else(|| slot.default_device().map(|d| d.id.clone()));
            let Some(asked) = asked else { continue };
            self.playable(&slot.slot, &asked)?;
            let used = match self.stand_in(&slot.slot, &asked) {
                Some(used) => {
                    setup.gaps.push(Gap { slot: slot.slot.clone(), asked: asked.clone(), used: used.clone() });
                    used
                }
                None => asked.clone(),
            };
            setup.asked.insert(slot.slot.clone(), asked);
            setup.used.insert(slot.slot.clone(), used);
        }
        Ok(setup)
    }
}
