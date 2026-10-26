//! Library: the player's sections from `library.json`, with the same rules as the HTML page
//! (accepted names, order, switches), through `gameframe-library`. Each change sends
//! `library.changed`, so the game reads the section again.

use crate::state::use_frame;
use crate::Icon;
use dioxus::prelude::*;
use gameframe_frame::topics;
use gameframe_library::{Kind, Section};
use serde_json::json;

fn size(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1} MB", n as f64 / 1e6)
    } else {
        format!("{} KB", (n / 1000).max(1))
    }
}

#[component]
pub fn LibraryView() -> Element {
    let frame = use_frame();
    let spec = frame.spec();
    let mut chosen = use_signal(|| spec.sections.first().map(|s| s.id.clone()).unwrap_or_default());
    let Some(section) = spec.section(&chosen()).cloned() else {
        return rsx! { p { "library.json has no sections." } };
    };
    rsx! {
        section { class: "lib-section",
            div { class: "lib-tabs", role: "tablist",
                for s in spec.sections.iter().cloned() {
                    button {
                        r#type: "button",
                        role: "tab",
                        "aria-selected": "{s.id == chosen()}",
                        onclick: move |_| chosen.set(s.id.clone()),
                        Icon { name: s.icon.clone().unwrap_or_else(|| match s.kind { Kind::GameData => "game".into(), k => format!("{k:?}").to_lowercase() }) }
                        "{s.label}"
                    }
                }
            }
            for line in section.help.iter() {
                p { "{line}" }
            }
            Files { key: "{section.id}", section }
        }
    }
}

#[component]
fn Files(section: Section) -> Element {
    let frame = use_frame();
    let mut problem = use_signal(String::new);
    let id = section.id.clone();
    let entries = use_resource({
        let id = id.clone();
        move || {
            let id = id.clone();
            async move {
                let _ = frame.revision();
                match frame.library() {
                    Some(lib) => lib.entries(&id).await.map_err(|e| e.to_string()),
                    None => Ok(Vec::new()),
                }
            }
        }
    });
    let changed = move |id: String| frame.update(move |s| s.send(topics::LIBRARY_CHANGED, json!({ "section": id })));
    let list = entries.read().clone().unwrap_or(Ok(Vec::new()));
    let items = match list {
        Ok(items) => items,
        Err(e) => return rsx! { p { class: "lib-status is-bad", "{e}" } },
    };
    let count = items.len();
    let accept = section.accept.join(",");
    let input_id = format!("frame-add-{id}");

    rsx! {
        ul { class: "lib-rows",
            if items.is_empty() {
                li { class: "lib-row is-empty", {section.empty.clone().unwrap_or_else(|| format!("No {} yet.", section.label.to_lowercase()))} }
            }
            for (i, l) in items.into_iter().enumerate() {
                {
                    let name = l.entry.name.clone();
                    let title = l.entry.card.as_ref().map(|c| c.title.clone()).unwrap_or_else(|| name.clone());
                    let detail = l.entry.card.as_ref().and_then(|c| c.detail.clone());
                    let meta = [detail, Some(size(l.entry.size))].into_iter().flatten().collect::<Vec<_>>().join(" · ");
                    let (n1, n2, n3, n4) = (name.clone(), name.clone(), name.clone(), name.clone());
                    let (s1, s2, s3, s4) = (id.clone(), id.clone(), id.clone(), id.clone());
                    rsx! {
                        li { class: if l.on { "lib-row" } else { "lib-row is-off" },
                            if section.toggles() {
                                input { r#type: "checkbox", checked: l.on, "aria-label": "Use {name}",
                                    onchange: move |e| { let (s, n) = (s1.clone(), n1.clone()); spawn(async move {
                                        if let Some(lib) = frame.library() { let _ = lib.set_on(&s, &n, e.checked()).await; }
                                        changed(s);
                                    }); } }
                            }
                            if let Some(t) = l.entry.card.as_ref().and_then(|c| c.thumbnail.clone()) {
                                img { class: "lib-thumb", src: "{t}", alt: "" }
                            }
                            div { class: "lib-row-text",
                                span { class: "lib-row-name", "{title}" }
                                span { class: "lib-row-meta", "{meta}" }
                            }
                            if section.ordered() {
                                button { r#type: "button", class: "lib-button lib-button--icon", title: "Load earlier", disabled: i == 0,
                                    onclick: move |_| { let (s, n) = (s2.clone(), n2.clone()); spawn(async move {
                                        if let Some(lib) = frame.library() { let _ = lib.move_to(&s, &n, i.saturating_sub(1)).await; }
                                        changed(s);
                                    }); },
                                    Icon { name: "up" } }
                                button { r#type: "button", class: "lib-button lib-button--icon", title: "Load later", disabled: i + 1 == count,
                                    onclick: move |_| { let (s, n) = (s3.clone(), n3.clone()); spawn(async move {
                                        if let Some(lib) = frame.library() { let _ = lib.move_to(&s, &n, i + 1).await; }
                                        changed(s);
                                    }); },
                                    Icon { name: "down" } }
                            }
                            button { r#type: "button", class: "lib-button lib-button--icon lib-button--danger", title: "Delete {name}",
                                onclick: move |_| { let (s, n) = (s4.clone(), n4.clone()); spawn(async move {
                                    if let Some(lib) = frame.library() { let _ = lib.remove(&s, &n).await; }
                                    changed(s);
                                }); },
                                Icon { name: "delete" } }
                        }
                    }
                }
            }
        }
        if section.import() {
            div { class: "lib-actions",
                label { class: "lib-button lib-button--quiet", r#for: "{input_id}",
                    Icon { name: "add" } "Add {section.label.to_lowercase()}" }
                input { id: "{input_id}", class: "lib-hidden", r#type: "file", multiple: section.many(), accept: "{accept}",
                    onchange: move |e| {
                        let files = e.files();
                        let s = id.clone();
                        spawn(async move {
                            let Some(lib) = frame.library() else { return };
                            for f in files {
                                let bytes = match f.read_bytes().await { Ok(b) => b.to_vec(), Err(err) => { problem.set(err.to_string()); continue } };
                                match lib.add(&s, &f.name(), &bytes).await {
                                    Ok(()) => problem.set(String::new()),
                                    Err(err) => problem.set(err.to_string()),
                                }
                            }
                            changed(s);
                        });
                    } }
            }
        }
        if !problem().is_empty() {
            p { class: "lib-status is-bad", "{problem}" }
        }
    }
}
