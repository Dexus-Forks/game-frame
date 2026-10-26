//! A session: one game, its settings and the event log, which every view of the frame reads.
//!
//! Each event goes through the session, which keeps the frame's own state (the settings, the
//! renderer, the game's status), passes the event to the game and to every listener, then
//! does the same for the events the game sends back.

use crate::event::{matches, topics, valid_topic, Event, Origin};
use crate::game::{Game, Outbox};
use crate::manifest::Manifest;
use crate::settings::Settings;
use serde_json::{json, Value};
use std::collections::VecDeque;

/// How many events the log keeps for the console.
pub const LOG_LENGTH: usize = 500;

type Listener = Box<dyn FnMut(&Event)>;

pub struct Session {
    game: Box<dyn Game>,
    manifest: Manifest,
    settings: Settings,
    renderer: Option<String>,
    state: String,
    log: VecDeque<Event>,
    next_id: u64,
    listeners: Vec<(String, Listener)>,
}

impl Session {
    /// Read the game's manifest and fill in the settings it does not have yet with their
    /// defaults. Send `frame.ready` with [`Session::ready`] once the views are listening.
    pub fn new(game: Box<dyn Game>, mut settings: Settings) -> Result<Session, String> {
        let manifest = game.manifest();
        manifest.check()?;
        for s in &manifest.settings {
            if !settings.get(&s.key).is_some_and(|v| s.accepts(v)) {
                settings.set(&s.key, s.default.clone());
            }
        }
        let renderer = settings
            .get("frame.renderer")
            .and_then(Value::as_str)
            .filter(|id| manifest.renderers.iter().any(|r| r.id == *id))
            .map(str::to_string)
            .or_else(|| manifest.default_renderer().map(|r| r.id.clone()));
        Ok(Session {
            game,
            manifest,
            settings,
            renderer,
            state: "stopped".into(),
            log: VecDeque::new(),
            next_id: 1,
            listeners: Vec::new(),
        })
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
        self.send(topics::FRAME_READY, json!({ "settings": settings, "renderer": self.renderer }));
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

    pub fn start(&mut self, args: Vec<String>) {
        let data = json!({ "renderer": self.renderer, "settings": self.settings.to_json(), "args": args });
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
            topics::GAME_STATUS => {
                if let Some(state) = e.str("state") {
                    self.state = state.to_string();
                }
            }
            _ => {}
        }
    }

    /// A line in the console from the frame itself.
    fn note(&mut self, level: &str, text: &str) {
        let e = Event::new(topics::LOG, Origin::Frame, json!({ "level": level, "text": text }));
        let _ = self.record(e);
    }
}
