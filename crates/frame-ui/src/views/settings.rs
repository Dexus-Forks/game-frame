//! Settings: the theme, when `library.json` offers a choice, then the game's settings from its
//! manifest, grouped, each with the input its kind needs.

use crate::state::use_frame;
use dioxus::prelude::*;
use gameframe_frame::{topics, Kind, Setting, PRESET};
use serde_json::{json, Value};

#[component]
pub fn SettingsView() -> Element {
    let frame = use_frame();
    let settings = frame.with(|s| {
        if s.manifest().can(gameframe_frame::capabilities::SETTINGS) { s.manifest().settings.clone() } else { Vec::new() }
    });
    let mut groups: Vec<(String, Vec<Setting>)> = Vec::new();
    for s in settings {
        let g = s.group.clone().unwrap_or_else(|| "Game".into());
        match groups.iter_mut().find(|(name, _)| *name == g) {
            Some((_, list)) => list.push(s),
            None => groups.push((g, vec![s])),
        }
    }
    rsx! {
        section { class: "lib-section lib-settings",
            ThemeChoice {}
            for (group, list) in groups {
                h2 { "{group}" }
                div { class: "lib-rows",
                    for s in list {
                        Field { setting: s }
                    }
                }
            }
        }
    }
}

#[component]
fn Field(setting: Setting) -> Element {
    let frame = use_frame();
    let key = setting.key.clone();
    let value = frame.setting(&key).unwrap_or(setting.default.clone());
    let id = format!("set-{}", key.replace('.', "-"));
    let set = move |v: Value| {
        let key = key.clone();
        frame.update(move |s| {
            if let Err(e) = s.set(&key, v) {
                s.send("log.message", json!({ "level": "warn", "text": e }));
            }
        });
    };
    let input = match &setting.kind {
        Kind::Toggle => rsx! {
            input { id: "{id}", r#type: "checkbox", checked: value.as_bool().unwrap_or(false),
                onchange: move |e| set(json!(e.checked())) }
        },
        Kind::Choice { options, from: Some(_) } if options.is_empty() => rsx! {
            span { class: "lib-row-meta", "The game lists these when it is ready." }
        },
        Kind::Choice { options, .. } => rsx! {
            select { id: "{id}", class: "lib-input", value: value.as_str().unwrap_or("").to_string(),
                onchange: move |e| set(json!(e.value())),
                for o in options.iter() {
                    option { value: "{o.value}", selected: value.as_str() == Some(o.value.as_str()), "{o.label}" }
                }
            }
        },
        Kind::Range { min, max, step } => {
            let shown = value.as_f64().unwrap_or(*min);
            rsx! {
                input { id: "{id}", class: "lib-range", r#type: "range", min: "{min}", max: "{max}", step: "{step}", value: "{shown}",
                    onchange: move |e| { if let Ok(n) = e.value().parse::<f64>() { set(json!(n)) } } }
                span { class: "lib-row-meta lib-range-value", "{shown}" }
            }
        }
        Kind::Text => rsx! {
            input { id: "{id}", class: "lib-input", r#type: "text", value: value.as_str().unwrap_or("").to_string(),
                onchange: move |e| set(json!(e.value())) }
        },
    };
    rsx! {
        div { class: "lib-row lib-setting",
            div { class: "lib-row-text",
                label { class: "lib-row-name", r#for: "{id}", "{setting.label}" }
                if let Some(help) = &setting.help {
                    span { class: "lib-row-meta", "{help}" }
                }
                if setting.restart {
                    span { class: "lib-row-meta", "Takes effect when the game starts again." }
                }
            }
            div { class: "lib-setting-input", {input} }
        }
    }
}

/// The theme picker, shown when `library.json` lists `presets`. The choice is the frame's own
/// preference, so it goes out as `frame.preference` and is saved with the settings.
#[component]
fn ThemeChoice() -> Element {
    let frame = use_frame();
    let spec = frame.spec();
    if spec.presets.is_empty() {
        return rsx! {};
    }
    let chosen = frame.setting(PRESET);
    let current = crate::active_theme(&spec, chosen.as_ref().and_then(|v| v.as_str())).unwrap_or_else(|| "none".into());
    rsx! {
        h2 { "Look" }
        div { class: "lib-rows",
            div { class: "lib-row lib-setting",
                div { class: "lib-row-text",
                    label { class: "lib-row-name", r#for: "set-frame-preset", "Theme" }
                    span { class: "lib-row-meta", "How this window looks. The game is not changed." }
                }
                div { class: "lib-setting-input",
                    select { id: "set-frame-preset", class: "lib-input",
                        onchange: move |e| frame.update(|s| s.send(topics::FRAME_PREFERENCE, json!({ "key": PRESET, "value": e.value() }))),
                        for name in spec.presets.iter() {
                            option { value: "{name}", selected: *name == current, {frame.theme_label(name)} }
                        }
                    }
                }
            }
        }
    }
}
