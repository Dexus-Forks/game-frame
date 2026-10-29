//! GameFrame's launcher: the frame in front of a game that `frame.json` describes, such as a
//! port's own program.
//!
//! ```text
//! gameframe-launcher [PATH] [--frame | --no-frame] [--data DIR] [--preset NAME]
//! ```
//!
//! - `PATH` is a `frame.json` file or the folder that holds one. The default is the current
//!   folder. The file's format is `gameframe_frame::Configured`'s: a manifest, the command that
//!   starts the game, and the game's folder. Choices with `"from": {"files": PATTERN}` list the
//!   matching files in that folder, such as the tracks or the cars.
//! - A `library.json` next to `frame.json` gives the look, the help pages and the player's
//!   library, as for the HTML library page. Without one the window takes the game's name and
//!   offers every ready-made theme.
//! - `--data DIR` keeps the settings and the library there. The default is `launcher-data/`
//!   next to `frame.json`.
//! - `--frame` opens the window even when the player turned it off at start, and `--no-frame`
//!   starts the game at once with the window hidden. Without either, the saved choice decides.
//! - `--preset NAME` starts with one of the ready-made themes, until the player picks one.
//!
//! The window hides while the game runs. When the game ends by itself, the window comes back,
//! or the launcher exits if the player chose "Exit to desktop" for that. A stop from the window
//! always comes back to it. A game started this way has no channel to the frame, so it cannot
//! open the window while it runs; a port that speaks the contract (`gameframe_frame::Process`,
//! or the frame in its own program) can, with `frame.open`.
//!
//! It does not install, find or check the game: the command in `frame.json` must work from the
//! game's folder.

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use gameframe_frame::{settings_file, Configured, Session, Settings};
use gameframe_frame_ui::{Frame, FrameSetup, THEMES};
use gameframe_library::{Folder, Hooks, Library, NoHooks, Spec, Store};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const USAGE: &str = "usage: gameframe-launcher [PATH] [--frame | --no-frame] [--data DIR] [--preset NAME]
PATH is frame.json or the folder that holds it (the default is the current folder).";

/// What the command line asked for.
struct Options {
    config: PathBuf,
    data: PathBuf,
    preset: Option<String>,
    args: Vec<String>,
}

static OPTIONS: OnceLock<Options> = OnceLock::new();

fn options(args: Vec<String>) -> Result<Options, String> {
    let mut path = None;
    let (mut data, mut preset) = (None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => return Err(String::new()),
            "--data" => data = Some(PathBuf::from(it.next().ok_or("--data needs a folder")?)),
            "--preset" => preset = Some(it.next().ok_or("--preset needs a name")?.clone()),
            "--frame" | "--launcher" | "--no-frame" => {}
            s if s.starts_with('-') => return Err(format!("unknown option {s}")),
            s => path = Some(PathBuf::from(s)),
        }
    }
    let path = path.unwrap_or_else(|| PathBuf::from("."));
    let config = if path.is_dir() { path.join("frame.json") } else { path };
    if !config.is_file() {
        return Err(format!("{} does not exist", config.display()));
    }
    let base = config.parent().unwrap_or(Path::new(".")).to_path_buf();
    Ok(Options { data: data.unwrap_or_else(|| base.join("launcher-data")), config, preset, args })
}

fn settings_path() -> PathBuf {
    OPTIONS.get().expect("options are read first").data.join("settings.json")
}

/// `library.json` next to `frame.json`, or a plain look with the game's name.
fn spec(options: &Options, name: &str) -> Result<(Spec, bool), String> {
    let base = options.config.parent().unwrap_or(Path::new("."));
    let path = base.join("library.json");
    let mut spec = if path.is_file() {
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut spec = Spec::from_json(&text)?;
        gameframe_frame_ui::load_pages(&mut spec, base);
        spec
    } else {
        let mut presets: Vec<String> = THEMES.iter().map(|(n, _, _)| n.to_string()).collect();
        presets.push("none".into());
        Spec { title: name.to_string(), presets, ..Spec::default() }
    };
    let has_library = path.is_file();
    if let Some(p) = &options.preset {
        spec.preset = (p != "none").then(|| p.clone());
    }
    Ok((spec, has_library))
}

fn main() {
    let options = match options(std::env::args().skip(1).collect()) {
        Ok(o) => o,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("gameframe-launcher: {e}");
            }
            eprintln!("{USAGE}");
            std::process::exit(if e.is_empty() { 0 } else { 2 });
        }
    };
    let shown = settings_file::load(&options.data.join("settings.json")).show_at_start(&options.args);
    let _ = OPTIONS.set(options);
    let window = WindowBuilder::new().with_title("GameFrame").with_visible(shown);
    dioxus::LaunchBuilder::desktop().with_cfg(Config::new().with_window(window)).launch(app);
}

fn app() -> Element {
    let setup = use_hook(make_setup);
    match setup {
        Ok(setup) => rsx! { Frame { setup } },
        Err(e) => rsx! { p { "The launcher could not start: {e}" } },
    }
}

fn make_setup() -> Result<FrameSetup, String> {
    let options = OPTIONS.get().expect("options are read first");
    let game = Configured::load(&options.config)?;
    let settings = settings_file::load(&settings_path());
    let shown = settings.show_at_start(&options.args);
    let mut session = Session::new(Box::new(game), settings)?;
    session.hide_while_running(true);
    let (spec, has_library) = spec(options, &session.manifest().name)?;
    dioxus::desktop::window().set_title(&spec.title);
    let mut setup = FrameSetup::new(session, spec.clone())
        .start_at_once(!shown)
        .with_window(|shown| {
            let window = dioxus::desktop::window();
            window.set_visible(shown);
            if shown {
                window.set_focus();
            }
        })
        .with_exit(|| std::process::exit(0))
        .with_save(|s: &Settings| {
            if let Err(e) = settings_file::save(&settings_path(), s) {
                eprintln!("could not save the settings: {e}");
            }
        });
    if has_library {
        let store = Box::new(Folder::new(options.data.join("library"))) as Box<dyn Store>;
        setup = setup.with_library(Library::new(spec, store, Box::new(NoHooks) as Box<dyn Hooks>));
    }
    Ok(setup)
}
