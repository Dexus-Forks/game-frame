//! The frame with no game. The demo game from `gameframe-frame` stands in for a port: it has
//! three renderers with different grades, five settings (a track list the game fills) and a pad of controls, it answers each
//! event on the console, and it presses its own controls every two seconds.
//!
//! The sections come from `templates/library-page/library.json`. The look is the ready-made
//! Windows 98 theme (`"preset": "win98"` in `library.json`). Try the others with
//! `--preset xp` on the desktop or `?preset=xp` in the browser (the library page's README lists
//! them), or `none` for the frame's own look. The Settings tab switches between them, and a
//! saved choice wins over the starting one.
//!
//! Desktop: `cargo run`. The frame opens on the first run until "Show this at start" is turned
//! off; `cargo run -- --frame` opens it again and `--no-frame` skips it. Settings and the
//! library are kept in `frame-demo-data/` in the current folder.
//!
//! Browser: `trunk serve --no-default-features --features web`, or
//! `trunk build --public-url ./` for a folder to serve. Settings are kept in local storage and
//! the library in IndexedDB, encrypted.
//!
//! It does not start a real game: "Start" only runs the demo game's loading steps.

use dioxus::prelude::*;
use gameframe_frame::{Demo, Session, Settings};
use gameframe_frame_ui::{Frame, FrameSetup};
use gameframe_library::{Hooks, Library, NoHooks, Spec, Store};

/// The template's sections and pages, with the ready-made Windows 98 theme.
const SPEC: &str = include_str!("../library.json");
const HELP: &str = include_str!("../../../templates/library-page/help.md");

/// The spec, with its help page's text filled in and the preset the demo was asked for.
fn spec() -> Spec {
    let mut spec = Spec::from_json(SPEC).expect("the demo's library.json is valid");
    if let Some(name) = asked_preset() {
        spec.preset = (name != "none").then_some(name);
    }
    for p in &mut spec.pages {
        if p.src.as_deref() == Some("help.md") {
            p.text = Some(HELP.to_string());
        }
    }
    spec
}

/// `--preset NAME` on the desktop.
#[cfg(not(target_arch = "wasm32"))]
fn asked_preset() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == "--preset").and_then(|i| args.get(i + 1).cloned())
}

/// `?preset=NAME` in the browser.
#[cfg(target_arch = "wasm32")]
fn asked_preset() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|p| p.strip_prefix("preset=").map(String::from))
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let settings = gameframe_frame::settings_file::load(&desktop::settings_path());
        if !settings.show_at_start(&args) {
            println!("The frame is turned off, so a port would start its game now.");
            println!("Run with --frame to open the frame again.");
            return;
        }
    }
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        let window = WindowBuilder::new().with_title(spec().title);
        dioxus::LaunchBuilder::desktop().with_cfg(Config::new().with_window(window)).launch(app);
    }
    #[cfg(target_arch = "wasm32")]
    dioxus::launch(app);
}

fn app() -> Element {
    let setup = use_resource(make_setup);
    let ready = setup.read().clone();
    match ready {
        Some(Ok(setup)) => rsx! { Frame { setup } },
        Some(Err(e)) => rsx! { p { "The frame could not start: {e}" } },
        None => rsx! {},
    }
}

fn library(spec: &Spec, store: Box<dyn Store>) -> gameframe_library::Library<Box<dyn Store>, Box<dyn Hooks>> {
    Library::new(spec.clone(), store, Box::new(NoHooks) as Box<dyn Hooks>)
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use std::path::PathBuf;

    pub fn data_dir() -> PathBuf {
        PathBuf::from("frame-demo-data")
    }

    pub fn settings_path() -> PathBuf {
        data_dir().join("settings.json")
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn make_setup() -> Result<FrameSetup, String> {
    use gameframe_frame::settings_file;
    let spec = spec();
    let settings = settings_file::load(&desktop::settings_path());
    let session = Session::new(Box::new(Demo::default()), settings)?;
    let store = gameframe_library::Folder::new(desktop::data_dir().join("library"));
    Ok(FrameSetup::new(session, spec.clone())
        .with_library(library(&spec, Box::new(store)))
        .with_save(|s: &Settings| {
            if let Err(e) = settings_file::save(&desktop::settings_path(), s) {
                eprintln!("could not save the settings: {e}");
            }
        }))
}

#[cfg(target_arch = "wasm32")]
async fn make_setup() -> Result<FrameSetup, String> {
    const KEY: &str = "gameframe-frame-demo.settings";
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    let saved = storage.as_ref().and_then(|s| s.get_item(KEY).ok().flatten());
    let settings = saved.map(|t| Settings::from_json(&t)).unwrap_or_default();
    let spec = spec();
    let session = Session::new(Box::new(Demo::default()), settings)?;
    let store = gameframe_library::web::IndexedDb::open(&spec).await.map_err(|e| e.to_string())?;
    Ok(FrameSetup::new(session, spec.clone())
        .with_library(library(&spec, Box::new(store)))
        .with_save(move |s: &Settings| {
            if let Some(storage) = &storage {
                let _ = storage.set_item(KEY, &s.to_text());
            }
        }))
}
