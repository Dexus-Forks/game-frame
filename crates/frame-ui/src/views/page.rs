//! A help page from `library.json`'s `pages`, rendered from its `text`. A page with only a
//! `src` needs its text read first: `crate::load_pages` does that on the desktop.

use crate::state::use_frame;
use dioxus::prelude::*;

#[component]
pub fn Page(path: String) -> Element {
    let frame = use_frame();
    let spec = frame.spec();
    let Some(page) = spec.pages.iter().find(|p| p.path == path) else {
        return rsx! { p { "No page {path}." } };
    };
    let html = match &page.text {
        Some(text) => crate::markdown::to_html(text),
        None => format!("<p>This page's file, {}, was not read.</p>", page.src.as_deref().unwrap_or("?")),
    };
    rsx! {
        article { class: "lib-section",
            h2 { "{page.title}" }
            div { class: "lib-md", dangerous_inner_html: "{html}" }
        }
    }
}
