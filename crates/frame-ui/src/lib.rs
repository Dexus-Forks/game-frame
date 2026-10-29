//! The frame's views as Dioxus components, for the browser and the desktop.
//!
//! [`Frame`] is the whole frame: a header from `library.json`, then tabs for Play (renderers
//! with their grades, start and stop, the game's status), Settings (from the game's manifest),
//! Library (the player's files, through `gameframe-library`), each help page, and Console (every
//! event, the game's controls, and a line to send any event). The views talk to the game only
//! through `gameframe-frame`'s [`Session`](gameframe_frame::Session).
//!
//! The styles are the HTML library page's design tokens and components, plus `frame.css`, so the
//! same `library.json` theme styles both. Every class starts with `lib-`. `"preset"` in
//! `library.json` picks a ready-made theme by name ([`THEMES`] lists them), and [`FrameSetup::with_css`] adds the game's own stylesheet after it.
//!
//! ```ignore
//! let session = Session::new(Box::new(Demo::default()), Settings::new())?;
//! let setup = FrameSetup::new(session, Spec::from_json(include_str!("library.json"))?);
//! dioxus::LaunchBuilder::new().with_context(setup).launch(app);
//! fn app() -> Element { rsx! { Frame { setup: use_context::<FrameSetup>() } } }
//! ```

mod icon;
mod markdown;
mod shell;
mod state;
mod themes;
mod views;

pub use icon::Icon;
pub use markdown::to_html as markdown_to_html;
pub use shell::{Frame, FRAME_CSS, LIBRARY_CSS, TOKENS_CSS};
pub use state::{use_frame, AnyLibrary, FrameContext, FrameSetup, SaveSettings, ShowWindow};
pub use themes::{active_theme, theme, theme_label, THEMES};
pub use views::{Console, LibraryView, Page, Play, SettingsView};

/// Read each page's `src` file into its `text`, from the folder `library.json` is in. For the
/// desktop, where pages ship next to the program.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_pages(spec: &mut gameframe_library::Spec, dir: &std::path::Path) {
    for p in &mut spec.pages {
        if p.text.is_none() {
            if let Some(src) = &p.src {
                p.text = std::fs::read_to_string(dir.join(src)).ok();
            }
        }
    }
}
