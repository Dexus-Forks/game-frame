//! The frame's outside: the styles, the header, the tabs, the game's tick, and the view of the
//! chosen tab.

use crate::state::{sleep, FrameContext, FrameSetup};
use crate::views::{Console, LibraryView, Page, Play, SettingsView};
use dioxus::prelude::*;
use gameframe_frame::capabilities;

/// The design tokens and components shared with GameFrame's HTML library page, then the
/// frame's own parts.
pub const TOKENS_CSS: &str = include_str!("../../../templates/library-page/tokens.css");
pub const LIBRARY_CSS: &str = include_str!("../../../templates/library-page/library.css");
pub const FRAME_CSS: &str = include_str!("frame.css");

#[derive(Clone, PartialEq)]
enum Tab {
    Play,
    Settings,
    Library,
    Console,
    Page(String),
}

/// The whole frame. Give it a [`FrameSetup`] made in `main`.
#[component]
pub fn Frame(setup: FrameSetup) -> Element {
    let frame = FrameContext::provide(setup);
    let mut tab = use_signal(|| Tab::Play);

    // The game's tick: poll it, and draw again only when it sent something.
    use_future(move || async move {
        loop {
            sleep(frame.tick_ms()).await;
            frame.tick();
        }
    });
    use_hook(move || {
        frame.update(|s| s.ready());
        if frame.starts_at_once() {
            frame.update(|s| s.start(Vec::new()));
        }
    });

    let spec = frame.spec();
    let manifest = frame.with(|s| s.manifest().clone());
    let chosen = frame.setting(gameframe_frame::PRESET);
    let active = crate::active_theme(&spec, chosen.as_ref().and_then(|v| v.as_str()));
    let theme: String = spec.theme.iter().map(|(k, v)| format!("--lib-{k}: {v};")).collect();
    let scheme = spec.scheme.clone().unwrap_or_else(|| "dark".into());
    // The theme drawn, on the page as `data-theme`, so a game's own styles can apply to one
    // theme only (a font built from the player's files for its own theme).
    let theme_name = active.clone().unwrap_or_default();
    let backdrop = spec.backdrop.as_ref().map(|b| format!("--lib-backdrop: url(\"{b}\");")).unwrap_or_default();

    let mut tabs = vec![(Tab::Play, "Play".to_string(), "start")];
    if (manifest.can(capabilities::SETTINGS) && !manifest.settings.is_empty()) || !spec.presets.is_empty() {
        tabs.push((Tab::Settings, "Settings".into(), "file"));
    }
    if frame.library().is_some() {
        tabs.push((Tab::Library, "Library".into(), "saves"));
    }
    for p in &spec.pages {
        tabs.push((Tab::Page(p.path.clone()), p.title.clone(), "file"));
    }
    tabs.push((Tab::Console, "Console".into(), "game"));
    let current = tab();
    let shown = frame.with(|s| s.shown());

    // Hidden with no window to hide (the browser): the game has the page, and one button
    // brings the frame back.
    if !shown && !frame.has_window() {
        return rsx! {
            style { "{TOKENS_CSS}" }
            style { "{LIBRARY_CSS}" }
            style { "{FRAME_CSS}" }
            div { id: "library", class: "lib-frame-closed", "data-scheme": "{scheme}", "data-theme": "{theme_name}", style: "{theme}",
                button { r#type: "button", class: "lib-button lib-frame-open",
                    onclick: move |_| frame.update(|s| s.open_frame()),
                    crate::Icon { name: "game" } "Open the frame"
                }
            }
        };
    }

    rsx! {
        style { "{TOKENS_CSS}" }
        style { "{LIBRARY_CSS}" }
        style { "{FRAME_CSS}" }
        if let Some(css) = active.as_deref().and_then(|n| frame.theme_css(n)) {
            style { "{css}" }
        }
        if let Some(css) = frame.css() {
            style { "{css}" }
        }
        div { id: "library", class: "lib-frame", "data-scheme": "{scheme}", "data-theme": "{theme_name}", style: "{theme}{backdrop}",
            div { class: "lib-backdrop" }
            main { class: "lib-card lib-frame-card",
                header { class: "lib-header",
                    if let Some(logo) = &spec.logo {
                        img { class: "lib-logo", src: "{logo}", alt: "{spec.title}" }
                    } else {
                        h1 { class: "lib-title", "{spec.title}" }
                    }
                    if !spec.tagline.is_empty() {
                        p { class: "lib-tagline", "{spec.tagline}" }
                    }
                }
                nav { class: "lib-nav", "aria-label": "Frame",
                    for (t, label, icon) in tabs {
                        a {
                            href: "#",
                            "aria-current": if t == current { "page" } else { "false" },
                            onclick: move |e| { e.prevent_default(); tab.set(t.clone()); },
                            crate::Icon { name: icon }
                            "{label}"
                        }
                    }
                }
                match current {
                    Tab::Play => rsx! { Play {} },
                    Tab::Settings => rsx! { SettingsView {} },
                    Tab::Library => rsx! { LibraryView {} },
                    Tab::Console => rsx! { Console {} },
                    Tab::Page(path) => rsx! { Page { path } },
                }
                if !spec.links.is_empty() {
                    nav { class: "lib-links",
                        for l in spec.links.iter() {
                            a { href: "{l.href}", target: "_blank", rel: "noopener", "{l.text}" }
                        }
                    }
                }
                if let Some(fine) = &spec.fine {
                    p { class: "lib-fine", "{fine}" }
                }
            }
        }
    }
}
