//! A game that is not there: the frame's default, so the frame can be shown, styled and tested
//! with no port. It lists every capability, three renderers and three sound cards with
//! different grades, a few
//! settings and a pad of controls, and answers each event with a console line. It also
//! presses its own controls every two seconds and sends each as `demo.press`, a topic of its
//! own, so the console shows traffic with no one at the keys.

use crate::device::{chosen, ids, slots, Slot};
use crate::event::{topics, Event};
use crate::game::{Game, Outbox};
use crate::manifest::{capabilities, Choice, Control, Grade, Kind, Manifest, OptionsFrom, Renderer, Setting, CONTRACT};
use serde_json::json;

#[derive(Default)]
pub struct Demo {
    running: bool,
    /// Ticks left before a started game reports that it runs.
    loading: u32,
    /// Ticks since the last dummy press, and how many it has made.
    idle: u32,
    presses: usize,
}

/// The ticks between dummy presses: two seconds at the frame's 50 ms tick.
pub const DEMO_PRESS_TICKS: u32 = 40;
const DEMO_KEYS: [&str; 6] = ["up", "right", "down", "left", "fire", "start"];

impl Game for Demo {
    fn manifest(&self) -> Manifest {
        let renderer = |id: &str, label: &str, detail: &str, grade, done, total, default| Renderer {
            id: id.into(),
            label: label.into(),
            detail: Some(detail.into()),
            grade,
            done,
            total,
            notes: if grade == Grade::Partial { vec!["Fog is drawn without the card's table.".into()] } else { vec![] },
            default,
        };
        let control = |name: &str, label: &str, key: &str| Control { name: name.into(), label: label.into(), key: Some(key.into()) };
        Manifest {
            contract: CONTRACT.into(),
            name: "No game".into(),
            version: "demo".into(),
            capabilities: [capabilities::START, capabilities::RENDERERS, capabilities::SETTINGS, capabilities::INPUT, capabilities::LIBRARY, capabilities::DEVICES]
                .map(String::from)
                .to_vec(),
            renderers: vec![
                renderer("software", "Software", "The original's own renderer", Grade::Exact, None, None, true),
                renderer("glide", "3dfx Glide", "Through GameFrame's Glide library", Grade::Partial, Some(131), Some(156), false),
                renderer("powervr", "PowerVR", "Through GameFrame's SGL library", Grade::Experimental, Some(61), Some(134), false),
            ],
            devices: vec![Slot {
                slot: slots::SOUND.into(),
                label: "Sound card".into(),
                options: vec![
                    renderer(ids::DIRECTSOUND, "DirectSound", "The original's own mixer", Grade::Exact, None, None, true),
                    renderer(ids::SOUND_BLASTER, "Sound Blaster 16", "Through GameFrame's Sound Blaster model", Grade::Experimental, Some(9), Some(24), false),
                    renderer(ids::NONE, "No sound", "Silent, as without a card", Grade::Exact, None, None, false),
                ],
            }],
            settings: vec![
                Setting {
                    key: "video.scale".into(),
                    label: "Scaling".into(),
                    help: Some("How each frame is enlarged to fit the window.".into()),
                    group: Some("Video".into()),
                    kind: Kind::Choice {
                        options: [("nearest", "Sharp pixels"), ("scale2x", "Scale2x"), ("crt", "CRT")]
                            .map(|(v, l)| Choice { value: v.into(), label: l.into() })
                            .to_vec(),
                        from: None,
                    },
                    default: json!("nearest"),
                    restart: false,
                },
                Setting {
                    key: "race.track".into(),
                    label: "Track".into(),
                    help: Some("The tracks the game found in its data. The demo sends three.".into()),
                    group: Some("Race".into()),
                    kind: Kind::Choice { options: Vec::new(), from: Some(OptionsFrom { files: None, game: true }) },
                    default: json!(""),
                    restart: false,
                },
                Setting {
                    key: "video.fog".into(),
                    label: "Fog".into(),
                    help: None,
                    group: Some("Video".into()),
                    kind: Kind::Toggle,
                    default: json!(true),
                    restart: false,
                },
                Setting {
                    key: "sound.volume".into(),
                    label: "Volume".into(),
                    help: None,
                    group: Some("Sound".into()),
                    kind: Kind::Range { min: 0.0, max: 100.0, step: 5.0 },
                    default: json!(80),
                    restart: false,
                },
                Setting {
                    key: "player.name".into(),
                    label: "Player name".into(),
                    help: Some("Shown on the high-score table.".into()),
                    group: Some("Player".into()),
                    kind: Kind::Text,
                    default: json!("Player"),
                    restart: true,
                },
            ],
            controls: vec![
                control("up", "Up", "↑"),
                control("down", "Down", "↓"),
                control("left", "Left", "←"),
                control("right", "Right", "→"),
                control("fire", "Fire", "Space"),
                control("start", "Start", "Enter"),
            ],
        }
    }

    fn on_event(&mut self, e: &Event, out: &mut Outbox) {
        match e.topic.as_str() {
            topics::FRAME_READY => {
                out.log("info", "the demo game is listening");
                let tracks = [("CANYON", "Canyon"), ("HARBOUR", "Harbour"), ("SNOWLINE", "Snowline")];
                let options: Vec<_> = tracks.iter().map(|(v, l)| json!({ "value": v, "label": l })).collect();
                out.send(topics::CONFIG_OPTIONS, json!({ "key": "race.track", "options": options }));
            }
            topics::GAME_START => {
                let renderer = e.str("renderer").unwrap_or("none").to_string();
                let sound = chosen(&e.data, slots::SOUND).unwrap_or("none").to_string();
                self.loading = 30;
                out.status("loading", &format!("starting with the {renderer} renderer and {sound} sound"));
            }
            topics::GAME_STOP => {
                self.running = false;
                self.loading = 0;
                out.status("stopped", "the player stopped the game");
            }
            topics::INPUT_PRESS if e.data["down"] == json!(true) => {
                let control = e.str("control").unwrap_or("?");
                let note = if self.running { "the game reacts" } else { "the game is not running" };
                out.log("info", &format!("pressed {control}: {note}"));
            }
            topics::CONFIG_SET => out.log("info", &format!("the game now uses {} = {}", e.str("key").unwrap_or("?"), e.data["value"])),
            topics::DEVICE_SELECT => out.log("info", &format!("the next start uses {} for {}", e.str("device").unwrap_or("?"), e.str("slot").unwrap_or("?"))),
            topics::RENDER_SELECT => out.log("info", &format!("the next start uses {}", e.str("renderer").unwrap_or("?"))),
            topics::LIBRARY_CHANGED => out.log("info", &format!("reading the {} section again", e.str("section").unwrap_or("?"))),
            _ => {}
        }
    }

    fn poll(&mut self, out: &mut Outbox) {
        self.idle += 1;
        if self.idle >= DEMO_PRESS_TICKS {
            self.idle = 0;
            let control = DEMO_KEYS[self.presses % DEMO_KEYS.len()];
            self.presses += 1;
            out.send("demo.press", json!({ "control": control, "count": self.presses, "running": self.running }));
        }
        if self.loading > 0 {
            self.loading -= 1;
            if self.loading == 0 {
                self.running = true;
                out.status("running", "the demo is running; press the console's controls");
            }
        }
    }
}
