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
