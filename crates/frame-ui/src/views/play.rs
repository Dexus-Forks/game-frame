//! Play: the renderer choice with how complete each one is, the start and stop button, the
//! game's status, and whether the frame opens at start.

use crate::state::use_frame;
use crate::Icon;
use dioxus::prelude::*;
use gameframe_frame::{capabilities, topics, Grade, SHOW_AT_START};
use serde_json::json;

fn grade_label(g: Grade) -> &'static str {
    match g {
        Grade::Exact => "Exact",
        Grade::Good => "Good",
        Grade::Partial => "Partial",
        Grade::Experimental => "Experimental",
        Grade::Missing => "Not started",
    }
}

#[component]
pub fn Play() -> Element {
    let frame = use_frame();
    let (manifest, chosen, state, detail) = frame.with(|s| {
        let detail = s
            .log()
            .iter()
            .rev()
            .find(|e| e.topic == "game.status")
            .and_then(|e| e.str("detail").map(String::from))
            .unwrap_or_default();
        (s.manifest().clone(), s.renderer().map(String::from), s.state().to_string(), detail)
    });
    let show_at_start = frame.setting(SHOW_AT_START).and_then(|v| v.as_bool()).unwrap_or(true);
    let running = matches!(state.as_str(), "loading" | "running" | "paused");
    let missing = use_resource(move || async move {
        let _ = frame.revision();
        match frame.library() {
            Some(lib) => lib.missing_game_data().await.map(|m| m.into_iter().map(|m| m.name).collect::<Vec<_>>()).unwrap_or_default(),
            None => Vec::new(),
        }
    });
    let missing = missing.read().clone().unwrap_or_default();

    rsx! {
        section { class: "lib-section",
            h2 { Icon { name: "game" } "{manifest.name}" }
            if !missing.is_empty() {
                ul { class: "lib-notice",
                    for name in missing.iter() {
                        li { Icon { name: "missing" } span { "Missing " code { "{name}" } ". Add it in the Library tab." } }
                    }
                }
            }
            if manifest.can(capabilities::RENDERERS) && !manifest.renderers.is_empty() {
                p { "Choose how the game draws." }
                div { class: "lib-renderers", role: "radiogroup", "aria-label": "Renderer",
                    for r in manifest.renderers.iter().cloned() {
                        {
                            let selected = chosen.as_deref() == Some(r.id.as_str());
                            let id = r.id.clone();
                            let share = match (r.done, r.total) {
                                (Some(d), Some(t)) if t > 0 => Some((d, t, d as f64 * 100.0 / t as f64)),
                                _ => None,
                            };
                            rsx! {
                                button {
                                    r#type: "button",
                                    class: "lib-renderer",
                                    role: "radio",
                                    "aria-checked": "{selected}",
                                    disabled: running,
                                    onclick: move |_| { let _ = frame.update(|s| s.select_renderer(&id)); },
                                    div { class: "lib-renderer-head",
                                        span { class: "lib-row-name", "{r.label}" }
                                        span { class: "lib-grade is-{grade_label(r.grade).to_lowercase().replace(' ', \"-\")}", "{grade_label(r.grade)}" }
                                    }
                                    if let Some(detail) = &r.detail {
                                        span { class: "lib-row-meta", "{detail}" }
                                    }
                                    if let Some((d, t, pct)) = share {
                                        div { class: "lib-meter", title: "{d} of {t}",
                                            div { class: "lib-meter-fill", style: "width: {pct:.0}%" }
                                        }
                                        span { class: "lib-row-meta", "{d} of {t} calls" }
                                    }
                                    for n in r.notes.iter() {
                                        span { class: "lib-row-meta", "{n}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if manifest.can(capabilities::START) {
                if running {
                    button { r#type: "button", class: "lib-button lib-button--primary lib-start",
                        onclick: move |_| frame.update(|s| s.stop()),
                        Icon { name: "stop" } "Stop"
                    }
                } else {
                    button { r#type: "button", class: "lib-button lib-button--primary lib-start",
                        disabled: !missing.is_empty(),
                        onclick: move |_| frame.update(|s| s.start(Vec::new())),
                        Icon { name: "start" } "Start the game"
                    }
                }
            }
            // A stopped game needs no line; the live region stays so a screen reader hears changes.
            div { class: "lib-status", role: "status", "aria-live": "polite",
                if state != "stopped" {
                    "The game is {state}."
                    if !detail.is_empty() { " {detail}" }
                }
            }
            label { class: "lib-check",
                input {
                    r#type: "checkbox",
                    checked: show_at_start,
                    onchange: move |e| frame.update(|s| s.send(topics::FRAME_PREFERENCE, json!({ "key": SHOW_AT_START, "value": e.checked() }))),
                }
                "Open this window when the game starts. "
                span { class: "lib-row-meta", "Start the game with --frame to open it again." }
            }
        }
    }
}
