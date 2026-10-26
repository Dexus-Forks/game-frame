//! Console: every event that crossed between the frame, the game and the player, newest first,
//! with a filter; a pad of the game's controls that sends `input.press`; and a line to send
//! any event by hand. With no game, the demo game answers here, so the whole contract can be
//! tried without a port.

use crate::state::use_frame;
use dioxus::prelude::*;
use gameframe_frame::{capabilities, Origin};
use serde_json::Value;

#[component]
pub fn Console() -> Element {
    let frame = use_frame();
    let mut filter = use_signal(|| "*".to_string());
    let mut topic = use_signal(|| "demo.ping".to_string());
    let mut data = use_signal(|| "{}".to_string());
    let mut problem = use_signal(String::new);
    let (controls, can_input) = frame.with(|s| (s.manifest().controls.clone(), s.manifest().can(capabilities::INPUT)));
    let pattern = filter();
    let events: Vec<_> = frame.with(|s| s.log().iter().rev().filter(|e| e.matches(pattern.trim())).take(200).cloned().collect());

    rsx! {
        section { class: "lib-section lib-console",
            if can_input && !controls.is_empty() {
                h2 { "Controls" }
                div { class: "lib-pad",
                    for c in controls {
                        {
                            let (down, up) = (c.name.clone(), c.name.clone());
                            rsx! {
                                button { r#type: "button", class: "lib-button lib-pad-key",
                                    onmousedown: move |_| frame.update(|s| s.press(&down, true)),
                                    onmouseup: move |_| frame.update(|s| s.press(&up, false)),
                                    span { "{c.label}" }
                                    if let Some(k) = &c.key { span { class: "lib-row-meta", "{k}" } }
                                }
                            }
                        }
                    }
                }
            }
            h2 { "Send an event" }
            form { class: "lib-send",
                onsubmit: move |e| {
                    e.prevent_default();
                    match serde_json::from_str::<Value>(&data()) {
                        Ok(v) if gameframe_frame::valid_topic(topic().trim()) => {
                            problem.set(String::new());
                            frame.update(|s| s.send(topic().trim(), v));
                        }
                        Ok(_) => problem.set("A topic is two or more lower-case words joined by dots.".into()),
                        Err(err) => problem.set(format!("The data is not JSON: {err}")),
                    }
                },
                input { class: "lib-input", "aria-label": "Topic", value: "{topic}", oninput: move |e| topic.set(e.value()) }
                input { class: "lib-input lib-mono", "aria-label": "Data (JSON)", value: "{data}", oninput: move |e| data.set(e.value()) }
                button { r#type: "submit", class: "lib-button lib-button--quiet", "Send" }
            }
            if !problem().is_empty() { p { class: "lib-status is-bad", "{problem}" } }
            div { class: "lib-console-head",
                h2 { "Events" }
                input { class: "lib-input lib-mono", "aria-label": "Show topics", value: "{filter}", title: "A topic, a prefix such as game.*, or * for all",
                    oninput: move |e| filter.set(e.value()) }
            }
            ol { class: "lib-log",
                for e in events {
                    li { class: "lib-log-line is-{origin(e.origin)}",
                        span { class: "lib-log-id", "{e.id}" }
                        span { class: "lib-log-from", "{origin(e.origin)}" }
                        span { class: "lib-log-topic", "{e.topic}" }
                        span { class: "lib-log-data", {summary(&e.data)} }
                    }
                }
            }
        }
    }
}

fn origin(o: Origin) -> &'static str {
    match o {
        Origin::Frame => "frame",
        Origin::Game => "game",
        Origin::Player => "player",
    }
}

/// A log line's text as itself; any other data as compact JSON.
fn summary(v: &Value) -> String {
    match (v.get("text").and_then(Value::as_str), v) {
        (Some(t), _) => t.to_string(),
        (None, Value::Null) => String::new(),
        _ => v.to_string(),
    }
}
