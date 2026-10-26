//! The built-in icons, the same set and paths as the HTML library page's, drawn in the text
//! colour. A name with a `.` or `/` is a picture URL, used as a mask so it takes the colour too.

use dioxus::prelude::*;

pub fn path(name: &str) -> &'static str {
    match name {
        "add" => "M12 5v14M5 12h14",
        "download" => "M12 4v11m0 0-4-4m4 4 4-4M5 19h14",
        "delete" => "M6 6l12 12M18 6 6 18",
        "up" => "M12 19V5m0 0-6 6m6-6 6 6",
        "down" => "M12 5v14m0 0-6-6m6 6 6-6",
        "start" => "M8 5v14l11-7z",
        "stop" => "M7 7h10v10H7z",
        "missing" => "M12 8v5m0 3v.5M10.3 4.2 2.8 18a2 2 0 0 0 1.7 3h15a2 2 0 0 0 1.7-3L13.7 4.2a2 2 0 0 0-3.4 0z",
        "game" => "M6 9h4M8 7v4M15 10h.01M18 8h.01M7 5h10a5 5 0 0 1 5 5v1a5 5 0 0 1-9 3h-2a5 5 0 0 1-9-3v-1a5 5 0 0 1 5-5z",
        "saves" => "M5 3h11l3 3v15H5zM8 3v5h7V3M8 21v-7h8v7",
        "mods" => "M12 3 3 8l9 5 9-5zM3 13l9 5 9-5M3 17.5l9 4.5 9-4.5",
        "screenshots" => "M3 7h4l2-3h6l2 3h4v13H3zM12 17a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
        "recordings" => "M3 6h13v12H3zM16 10l5-3v10l-5-3",
        "settings" => "M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0M14 4v4M8 10v4M16 16v4",
        "link" => "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1",
        "home" => "M3 11l9-8 9 8M5 10v11h14V10",
        "github" => "M9 19c-4 1.5-4-2-6-2.5M15 22v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12 12 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V22",
        "mastodon" => "M21 8.5C21 4 17.7 3 12 3S3 4 3 8.5v4C3 18 7 20 12 20c1.5 0 3-.3 4-.8l-.2-1.7c-1.2.4-2.6.5-3.8.5-2.4 0-4-1-4.2-3 4 .9 8.2.8 10.4-.2 1.7-.8 2.8-2.2 2.8-4.3zM8 14V9a2 2 0 0 1 4 0v3m0 0V9a2 2 0 0 1 4 0v5",
        _ => "M7 3h7l5 5v13H7zM14 3v5h5",
    }
}

#[component]
pub fn Icon(name: String) -> Element {
    if name.contains(['.', '/']) {
        return rsx! { span { class: "lib-icon is-mask", "aria-hidden": "true", style: "--lib-icon-url: url(\"{name}\")" } };
    }
    let filled = matches!(name.as_str(), "start" | "stop");
    rsx! {
        span { class: "lib-icon", "aria-hidden": "true",
            svg { view_box: "0 0 24 24",
                path {
                    d: path(&name),
                    fill: if filled { "currentColor" } else { "none" },
                    stroke: if filled { "none" } else { "currentColor" },
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
            }
        }
    }
}
