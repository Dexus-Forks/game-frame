//! A game set up from `frame.json` alone: a manifest and the command that starts the game, such
//! as a port's own program.
//!
//! On `game.start` [`Configured`] runs the command, with `{renderer}` and `{setting:KEY}` in the
//! arguments replaced, and reports `running`, then `stopped` when the program ends (`failed` when
//! it exits with an error). A choice
//! whose manifest says `"from": {"files": "DATA/TRACKS/*.TRK"}` is filled from the game's folder
//! when [`Configured::load`] reads the file.
//!
//! ```json
//! {
//!   "manifest": { "contract": "1.0", "name": "Example", "capabilities": ["start", "settings"],
//!     "settings": [{ "key": "race.track", "label": "Track", "type": "choice",
//!                    "from": { "files": "DATA/TRACKS/*.TRK" }, "default": "" }] },
//!   "start": { "command": "./game", "args": ["--track", "{setting:race.track}"] },
//!   "folder": "game"
//! }
//! ```
//!
//! `folder` is the game's folder, relative to `frame.json` (the default is the file's own
//! folder). The command runs there, and file lists are read from there. The game cannot open
//! the frame again while it runs: the frame comes back when the program ends.

use crate::event::{topics, Event};
use crate::files::list_files;
use crate::game::{Game, Outbox};
use crate::manifest::{Choice, Kind, Manifest};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameFile {
    pub manifest: Manifest,
    pub start: Option<StartCommand>,
    /// The game's folder, relative to `frame.json`.
    #[serde(default)]
    pub folder: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartCommand {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

pub struct Configured {
    file: FrameFile,
    child: Option<Child>,
    /// The folder the command runs in and file lists come from, when it was loaded from a path.
    folder: Option<PathBuf>,
}

impl Configured {
    /// Read `frame.json` text. The command runs in the current folder and no file lists are
    /// filled; [`Configured::load`] does both.
    pub fn from_json(text: &str) -> Result<Configured, String> {
        let file: FrameFile = serde_json::from_str(text).map_err(|e| format!("frame.json: {e}"))?;
        file.manifest.check()?;
        Ok(Configured { file, child: None, folder: None })
    }

    /// Read a `frame.json` file, find the game's folder, and fill each choice that lists files.
    pub fn load(path: &Path) -> Result<Configured, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut game = Configured::from_json(&text)?;
        let base = path.parent().unwrap_or(Path::new("."));
        let folder = base.join(game.file.folder.clone().unwrap_or_default());
        for s in &mut game.file.manifest.settings {
            let pattern = match &s.kind {
                Kind::Choice { from: Some(from), .. } => from.files.clone(),
                _ => None,
            };
            if let Some(pattern) = pattern {
                let list = list_files(&folder, &pattern).into_iter().map(|v| Choice { label: v.clone(), value: v }).collect();
                s.fill(list);
            }
        }
        game.folder = Some(folder);
        Ok(game)
    }

    /// The game's folder, when it was loaded from a path.
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    /// The arguments with `{renderer}` and `{setting:KEY}` filled in from a `game.start`.
    pub fn arguments(&self, start: &Event) -> Vec<String> {
        let renderer = start.str("renderer").unwrap_or("");
        let settings = &start.data["settings"];
        let mut args: Vec<String> = self
            .file
            .start
            .iter()
            .flat_map(|s| s.args.iter())
            .map(|a| {
                let mut a = a.replace("{renderer}", renderer);
                while let Some(i) = a.find("{setting:") {
                    let Some(end) = a[i..].find('}') else { break };
                    let key = &a[i + 9..i + end];
                    let value = match &settings[key] {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        v => v.to_string(),
                    };
                    a.replace_range(i..i + end + 1, &value);
                }
                a
            })
            .collect();
        if let Some(extra) = start.data["args"].as_array() {
            args.extend(extra.iter().filter_map(|v| v.as_str().map(String::from)));
        }
        args
    }
}

impl Game for Configured {
    fn manifest(&self) -> Manifest {
        self.file.manifest.clone()
    }

    fn on_event(&mut self, e: &Event, out: &mut Outbox) {
        match e.topic.as_str() {
            topics::GAME_START => {
                let Some(start) = &self.file.start else {
                    return out.status("failed", "frame.json has no start command");
                };
                let args = self.arguments(e);
                let mut command = Command::new(&start.command);
                command.args(&args);
                if let Some(folder) = &self.folder {
                    command.current_dir(folder);
                }
                match command.spawn() {
                    Ok(child) => {
                        self.child = Some(child);
                        out.status("running", &format!("{} {}", start.command, args.join(" ")));
                    }
                    Err(err) => out.status("failed", &format!("{}: {err}", start.command)),
                }
            }
            topics::GAME_STOP => {
                if let Some(mut c) = self.child.take() {
                    let _ = c.kill();
                    let _ = c.wait();
                }
                out.status("stopped", "the player stopped the game");
            }
            _ => {}
        }
    }

    fn poll(&mut self, out: &mut Outbox) {
        if let Some(c) = &mut self.child {
            if let Ok(Some(status)) = c.try_wait() {
                self.child = None;
                let state = if status.success() { "stopped" } else { "failed" };
                out.status(state, &format!("the game ended ({status})"));
            }
        }
    }
}
