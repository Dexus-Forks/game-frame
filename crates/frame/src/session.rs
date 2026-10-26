//! A session: one game, its settings and the event log, which every view of the frame reads.
//!
//! Each event goes through the session, which keeps the frame's own state (the settings, the
//! renderer, the game's status), passes the event to the game and to every listener, then
//! does the same for the events the game sends back.

use crate::event::{matches, topics, valid_topic, Event, Origin};
use crate::game::{Game, Outbox};
use crate::manifest::{Choice, Kind, Manifest};
use crate::settings::{Settings, ON_EXIT};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, VecDeque};

/// How many events the log keeps for the console.
pub const LOG_LENGTH: usize = 500;

type Listener = Box<dyn FnMut(&Event)>;

pub struct Session {
    game: Box<dyn Game>,
    manifest: Manifest,
    settings: Settings,
    renderer: Option<String>,
    /// The device chosen in each slot besides graphics, saved as `frame.device.SLOT`.
    devices: BTreeMap<String, String>,
    state: String,
    /// The game's reports (`report.new`), one per place, in the order they came.
    reports: Vec<gameframe_report::Report>,
    log: VecDeque<Event>,
    next_id: u64,
    listeners: Vec<(String, Listener)>,
    shown: bool,
    hide_while_running: bool,
    /// The frame sent `game.stop`: the game did not end by itself.
    stopping: bool,
    exit: bool,
}

impl Session {
    /// Read the game's manifest and fill in the settings it does not have yet with their
    /// defaults. Send `frame.ready` with [`Session::ready`] once the views are listening.
    pub fn new(game: Box<dyn Game>, mut settings: Settings) -> Result<Session, String> {
        let manifest = game.manifest();
        manifest.check()?;
        for s in &manifest.settings {
            if !settings.get(&s.key).is_some_and(|v| s.accepts(v)) {
                // A list filled from the game's files may not hold the default: take its first.
                let first = match &s.kind {
                    Kind::Choice { options, .. } if !s.accepts(&s.default) => options.first().map(|c| json!(c.value)),
                    _ => None,
                };
                settings.set(&s.key, first.unwrap_or_else(|| s.default.clone()));
            }
        }
        let renderer = settings
            .get("frame.renderer")
            .and_then(Value::as_str)
            .filter(|id| manifest.renderers.iter().any(|r| r.id == *id))
            .map(str::to_string)
            .or_else(|| manifest.default_renderer().map(|r| r.id.clone()));
        let mut devices = BTreeMap::new();
        for slot in &manifest.devices {
            let saved = settings
                .get(&device_key(&slot.slot))
                .and_then(Value::as_str)
                .filter(|id| slot.options.iter().any(|d| d.id == *id))
                .map(str::to_string);
            if let Some(id) = saved.or_else(|| slot.default_device().map(|d| d.id.clone())) {
                devices.insert(slot.slot.clone(), id);
            }
        }
        Ok(Session {
            game,
            manifest,
            settings,
            renderer,
            devices,
            state: "stopped".into(),
            reports: Vec::new(),
            log: VecDeque::new(),
            next_id: 1,
            listeners: Vec::new(),
            shown: true,
            hide_while_running: false,
            stopping: false,
            exit: false,
        })
    }

    /// Whether the frame should be on screen. It starts shown. `frame.close` and, with
    /// [`Session::hide_while_running`], a game that reports `running` hide it; `frame.open` and a
    /// game that stops or fails show it again.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// Hide the frame while the game runs: for a launcher whose game opens its own window.
    pub fn hide_while_running(&mut self, on: bool) {
        self.hide_while_running = on;
    }

    /// Whether the program should end now: after `frame.exit`, from the player, the game, or a
    /// game that ended by itself while `frame.on-exit` is `"desktop"`.
    pub fn wants_exit(&self) -> bool {
        self.exit
    }

    /// The player's "Exit to desktop": stop a running game, then send `frame.exit`.
    pub fn exit(&mut self) {
        if matches!(self.state.as_str(), "loading" | "running" | "paused") {
            self.stop();
        }
        self.send(topics::FRAME_EXIT, Value::Null);
    }

    /// Show the frame over the running game, and tell the game (`frame.open`).
    pub fn open_frame(&mut self) {
        self.send(topics::FRAME_OPEN, Value::Null);
    }

    /// The player's "Back to the game": hide the frame (`frame.close`).
    pub fn back_to_game(&mut self) {
        self.send(topics::FRAME_CLOSE, Value::Null);
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
    /// Whether the settings changed since the last call: then save them.
    pub fn settings_changed(&mut self) -> bool {
        self.settings.take_changed()
    }
    pub fn renderer(&self) -> Option<&str> {
        self.renderer.as_deref()
    }
    /// The device chosen in `slot`; the graphics slot is the renderer.
    pub fn device(&self, slot: &str) -> Option<&str> {
        if slot == crate::device::slots::VIDEO {
            return self.renderer();
        }
        self.devices.get(slot).map(String::as_str)
    }
    /// The chosen device of every slot, as `frame.ready` and `game.start` carry it.
    pub fn devices(&self) -> Value {
        let mut all = Map::new();
        if let Some(r) = &self.renderer {
            all.insert(crate::device::slots::VIDEO.into(), json!(r));
        }
        for (slot, id) in &self.devices {
            all.insert(slot.clone(), json!(id));
        }
        Value::Object(all)
    }
    /// The reports the game sent: places it reached that the port has not ported, and crashes.
    pub fn reports(&self) -> &[gameframe_report::Report] {
        &self.reports
    }
    /// The game's last `game.status` state.
    pub fn state(&self) -> &str {
        &self.state
    }
    /// The newest events, oldest first.
    pub fn log(&self) -> &VecDeque<Event> {
        &self.log
    }

    /// Call `f` for every event whose topic matches `pattern` (see [`Event::matches`]).
    pub fn on(&mut self, pattern: &str, f: impl FnMut(&Event) + 'static) {
        self.listeners.push((pattern.to_string(), Box::new(f)));
    }

    /// Send an event from the frame.
    pub fn send(&mut self, topic: &str, data: Value) {
        self.dispatch(Event::new(topic, Origin::Frame, data));
    }

    pub fn ready(&mut self) {
        let settings = self.settings.to_json();
        let devices = self.devices();
        self.send(topics::FRAME_READY, json!({ "settings": settings, "renderer": self.renderer, "devices": devices }));
    }

    /// A control on the console: `input.press`, from the player.
    pub fn press(&mut self, control: &str, down: bool) {
        self.dispatch(Event::new(topics::INPUT_PRESS, Origin::Player, json!({ "control": control, "down": down })));
    }

    /// Change a setting the manifest lists, with a value its kind takes.
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        let s = self.manifest.setting(key).ok_or_else(|| format!("no setting {key}"))?;
        if !s.accepts(&value) {
            return Err(format!("{} does not take {value}", s.label));
        }
        self.send(topics::CONFIG_SET, json!({ "key": key, "value": value }));
        Ok(())
    }

    pub fn select_renderer(&mut self, id: &str) -> Result<(), String> {
        if !self.manifest.renderers.iter().any(|r| r.id == id) {
            return Err(format!("no renderer {id}"));
        }
        self.send(topics::RENDER_SELECT, json!({ "renderer": id }));
        Ok(())
    }

    /// Choose a device in a slot. The graphics slot is the renderer (`render.select`); any
    /// other sends `device.select`.
    pub fn select_device(&mut self, slot: &str, id: &str) -> Result<(), String> {
        if slot == crate::device::slots::VIDEO {
            return self.select_renderer(id);
        }
        let s = self.manifest.devices.iter().find(|s| s.slot == slot).ok_or_else(|| format!("no slot {slot}"))?;
        if !s.options.iter().any(|d| d.id == id) {
            return Err(format!("no {slot} device {id}"));
        }
        self.send(topics::DEVICE_SELECT, json!({ "slot": slot, "device": id }));
        Ok(())
    }

    pub fn start(&mut self, args: Vec<String>) {
        let data = json!({ "renderer": self.renderer, "devices": self.devices(), "settings": self.settings.to_json(), "args": args });
        self.send(topics::GAME_START, data);
    }

    pub fn stop(&mut self) {
        self.send(topics::GAME_STOP, Value::Null);
    }

    /// Let the game send what it has: call it on every tick of the frame's loop.
    pub fn tick(&mut self) {
        let mut out = Outbox::default();
        self.game.poll(&mut out);
        self.deliver(out.take());
    }

    fn dispatch(&mut self, event: Event) {
        let mut out = Outbox::default();
        if let Some(e) = self.record(event) {
            self.game.on_event(&e, &mut out);
        }
        self.deliver(out.take());
    }

    /// Record the game's events. The game does not hear its own events.
    fn deliver(&mut self, events: Vec<Event>) {
        for e in events {
            let _ = self.record(e);
        }
    }

    /// Number the event, keep the frame's state, log it and tell the listeners. `None` when
    /// the topic is malformed.
    fn record(&mut self, mut e: Event) -> Option<Event> {
        if !valid_topic(&e.topic) {
            self.note("warn", &format!("dropped an event with the topic {:?}", e.topic));
            return None;
        }
        e.id = self.next_id;
        self.next_id += 1;
        self.apply(&e);
        self.log.push_back(e.clone());
        while self.log.len() > LOG_LENGTH {
            self.log.pop_front();
        }
        for (pattern, f) in &mut self.listeners {
            if matches(&e.topic, pattern) {
                f(&e);
            }
        }
        Some(e)
    }

    /// `config.options` from the game: fill the choice's list, and move a saved value that is
    /// not in it to the first option.
    fn fill_options(&mut self, e: &Event) {
        let (Some(key), Some(list)) = (e.str("key"), e.data.get("options")) else { return };
        let Ok(list) = serde_json::from_value::<Vec<Choice>>(list.clone()) else {
            return self.note("warn", &format!("config.options for {key}: the options are not a list of value and label"));
        };
        let Some(s) = self.manifest.settings.iter_mut().find(|s| s.key == key) else { return };
        if !matches!(&s.kind, Kind::Choice { from: Some(_), .. }) {
            return self.note("warn", &format!("config.options for {key}: its manifest does not say the options come later"));
        }
        let first = list.first().map(|c| c.value.clone()).unwrap_or_default();
        s.fill(list);
        let fits = self.settings.get(key).is_some_and(|v| s.accepts(v));
        if !fits {
            self.settings.set(key, json!(first));
        }
    }

    fn apply(&mut self, e: &Event) {
        match e.topic.as_str() {
            topics::CONFIG_SET | topics::CONFIG_CHANGED => {
                if let (Some(key), Some(value)) = (e.str("key"), e.data.get("value")) {
                    self.settings.set(key, value.clone());
                }
            }
            topics::FRAME_PREFERENCE => {
                if let (Some(key), Some(value)) = (e.str("key"), e.data.get("value")) {
                    if key.starts_with("frame.") {
                        self.settings.set(key, value.clone());
                    }
                }
            }
            topics::RENDER_SELECT => {
                if let Some(id) = e.str("renderer") {
                    self.renderer = Some(id.to_string());
                    self.settings.set("frame.renderer", json!(id));
                }
            }
            topics::DEVICE_SELECT => {
                if let (Some(slot), Some(id)) = (e.str("slot"), e.str("device")) {
                    if self.manifest.devices.iter().any(|s| s.slot == slot) {
                        self.devices.insert(slot.to_string(), id.to_string());
                        self.settings.set(&device_key(slot), json!(id));
                    }
                }
            }
            topics::GAME_STATUS => {
                if let Some(state) = e.str("state") {
                    self.state = state.to_string();
                    match state {
                        "running" if self.hide_while_running => self.shown = false,
                        "stopped" if !self.stopping && self.exit_to_desktop() => self.exit = true,
                        "stopped" | "failed" => self.shown = true,
                        _ => {}
                    }
                    if matches!(state, "stopped" | "failed") {
                        self.stopping = false;
                    }
                }
            }
            topics::GAME_START => self.stopping = false,
            topics::GAME_STOP => self.stopping = true,
            topics::FRAME_EXIT => self.exit = true,
            topics::FRAME_OPEN => self.shown = true,
            topics::FRAME_CLOSE => self.shown = false,
            topics::CONFIG_OPTIONS => self.fill_options(e),
            topics::REPORT_NEW => {
                if let Ok(r) = serde_json::from_value::<gameframe_report::Report>(e.data.clone()) {
                    match self.reports.iter_mut().find(|old| old.id == r.id) {
                        Some(old) => *old = r,
                        None => self.reports.push(r),
                    }
                }
            }
            _ => {}
        }
    }

    fn exit_to_desktop(&self) -> bool {
        self.settings.get(ON_EXIT).and_then(Value::as_str) == Some("desktop")
    }

    /// A line in the console from the frame itself.
    fn note(&mut self, level: &str, text: &str) {
        let e = Event::new(topics::LOG, Origin::Frame, json!({ "level": level, "text": text }));
        let _ = self.record(e);
    }
}

/// The saved setting that holds a slot's device.
fn device_key(slot: &str) -> String {
    format!("frame.device.{slot}")
}
