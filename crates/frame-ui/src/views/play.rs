//! Play: the game's setup, as the Manager or SETUP.EXE of a 1990s game showed it: one choice
//! per hardware slot (graphics, sound card, music) with how complete each device is. Then the
//! start and stop buttons, "Back to the game" while it runs, the game's status, and whether the
//! frame opens at start.

use crate::state::use_frame;
use crate::Icon;
use dioxus::prelude::*;
use gameframe_frame::{capabilities, device, topics, Grade, ON_EXIT, SHOW_AT_START};
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
    let reports = frame.with(|s| s.reports().to_vec());
    let (manifest, slots, chosen, state, detail) = frame.with(|s| {
        let detail = s
            .log()
            .iter()
            .rev()
            .find(|e| e.topic == "game.status")
            .and_then(|e| e.str("detail").map(String::from))
            .unwrap_or_default();
        // The slots the manifest opts into: graphics with `renderers`, the rest with `devices`.
        let m = s.manifest();
        let slots: Vec<_> = m
            .slots()
            .into_iter()
            .filter(|slot| !slot.options.is_empty())
            .filter(|slot| m.can(if slot.slot == device::slots::VIDEO { capabilities::RENDERERS } else { capabilities::DEVICES }))
            .collect();
        let chosen: Vec<_> = slots.iter().map(|slot| (slot.slot.clone(), s.device(&slot.slot).map(String::from))).collect();
        (m.clone(), slots, chosen, s.state().to_string(), detail)
    });
    let show_at_start = frame.setting(SHOW_AT_START).and_then(|v| v.as_bool()).unwrap_or(true);
    let on_exit = frame.setting(ON_EXIT).and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| "launcher".into());
    let can_exit = frame.has_exit();
    let preference = move |key: &'static str, value: serde_json::Value| frame.update(|s| s.send(topics::FRAME_PREFERENCE, json!({ "key": key, "value": value })));
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
            for slot in slots.iter().cloned() {
                {
                    let chosen = chosen.iter().find(|(k, _)| *k == slot.slot).and_then(|(_, v)| v.clone());
                    let prompt = if slot.slot == device::slots::VIDEO { "Choose how the game draws.".to_string() } else { format!("Choose the {}.", slot.label.to_lowercase()) };
                    rsx! {
                        h3 { class: "lib-slot", "{slot.label}" }
                        p { "{prompt}" }
                        div { class: "lib-renderers", role: "radiogroup", "aria-label": "{slot.label}",
                            for r in slot.options.iter().cloned() {
                                {
                                    let selected = chosen.as_deref() == Some(r.id.as_str());
                                    let id = r.id.clone();
                                    let which = slot.slot.clone();
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
                                            onclick: move |_| { let _ = frame.update(|s| s.select_device(&which, &id)); },
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
                }
            }
            if manifest.can(capabilities::START) {
                if running {
                    div { class: "lib-actions",
                        button { r#type: "button", class: "lib-button lib-button--primary lib-start",
                            onclick: move |_| frame.update(|s| s.back_to_game()),
                            Icon { name: "start" } "Back to the game"
                        }
                        button { r#type: "button", class: "lib-button lib-start",
                            onclick: move |_| frame.update(|s| s.stop()),
                            Icon { name: "stop" } "Stop"
                        }
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
            // What the game reached that the port has not done, and its crashes (`report.new`).
            if !reports.is_empty() {
                div { class: "lib-reports", role: "region", "aria-label": "Reports",
                    h3 { class: "lib-slot", "Reports" }
                    p { class: "lib-row-meta",
                        "The game reached places this port has not done yet. Each is written down with what the game was doing."
                    }
                    ul { class: "lib-notice",
                        for r in reports.iter() {
                            li {
                                Icon { name: "missing" }
                                span { class: "lib-report",
                                    span { class: "lib-row-name", "{r.what}" }
                                    span { class: "lib-row-meta",
                                        "{r.kind.name()}, {r.count}×"
                                        if let Some(o) = &r.original { ", original " code { "{o}" } }
                                    }
                                    if let Some(url) = &r.issue {
                                        a { class: "lib-button lib-button--quiet", href: "{url}", target: "_blank", rel: "noopener",
                                            "Open a GitHub issue"
                                        }
                                    } else {
                                        span { class: "lib-row-meta", "Saved as " code { "{r.id}.md" } " in the reports folder." }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            label { class: "lib-check",
                input {
                    r#type: "checkbox",
                    checked: show_at_start,
                    onchange: move |e| preference(SHOW_AT_START, json!(e.checked())),
                }
                "Open this window when the game starts. "
                span { class: "lib-row-meta", "Start the game with --frame to open it again." }
            }
            if can_exit {
                div { class: "lib-row lib-setting",
                    div { class: "lib-row-text",
                        label { class: "lib-row-name", r#for: "set-frame-on-exit", "When the game ends" }
                        span { class: "lib-row-meta", "Stopping it here always comes back to this window." }
                    }
                    div { class: "lib-setting-input",
                        select { id: "set-frame-on-exit", class: "lib-input",
                            onchange: move |e| preference(ON_EXIT, json!(e.value())),
                            option { value: "launcher", selected: on_exit != "desktop", "Come back here" }
                            option { value: "desktop", selected: on_exit == "desktop", "Exit to desktop" }
                        }
                    }
                }
                button { r#type: "button", class: "lib-button lib-exit",
                    onclick: move |_| frame.update(|s| s.exit()),
                    Icon { name: "stop" } "Exit to desktop"
                }
            }
        }
    }
}
