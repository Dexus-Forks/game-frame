//! A game set up from `frame.json` alone: a manifest and the command that starts the game, such
//! as a port's own program.
//!
//! On `game.start` [`Configured`] runs the command, with `{renderer}`, `{device:SLOT}` and `{setting:KEY}` in the
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
//! folder). The command runs there, and file lists are read from there. `image` names a disc
//! image instead (`"image": "media/GAME.CUE"`): file lists are read from inside it, and
//! `{image}` in the arguments becomes its full path, for an emulator that mounts it. `{folder}`
//! becomes the game's folder. The game cannot open the frame again while it runs: the frame
//! comes back when the program ends.

use crate::event::{topics, Event};
use crate::files::{list_files, list_image_files};
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
    /// A disc image to read file lists from, relative to `frame.json`.
    #[serde(default)]
    pub image: Option<PathBuf>,
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
    image: Option<PathBuf>,
}

impl Configured {
    /// Read `frame.json` text. The command runs in the current folder and no file lists are
    /// filled; [`Configured::load`] does both.
    pub fn from_json(text: &str) -> Result<Configured, String> {
        let file: FrameFile = serde_json::from_str(text).map_err(|e| format!("frame.json: {e}"))?;
        file.manifest.check()?;
        Ok(Configured { file, child: None, folder: None, image: None })
    }

    /// Read a `frame.json` file, find the game's folder, and fill each choice that lists files.
    pub fn load(path: &Path) -> Result<Configured, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut game = Configured::from_json(&text)?;
        let base = path.parent().unwrap_or(Path::new("."));
        let folder = base.join(game.file.folder.clone().unwrap_or_default());
        let image = game.file.image.as_ref().map(|i| base.join(i));
        if let Some(i) = &image {
            if !i.is_file() {
                return Err(format!("frame.json: the image {} does not exist", i.display()));
            }
        }
        for s in &mut game.file.manifest.settings {
            let pattern = match &s.kind {
                Kind::Choice { from: Some(from), .. } => from.files.clone(),
                _ => None,
            };
            if let Some(pattern) = pattern {
                let names = match &image {
                    Some(i) => list_image_files(i, &pattern).map_err(|e| format!("{}: {e}", i.display()))?,
                    None => list_files(&folder, &pattern),
                };
                let list = names.into_iter().map(|v| Choice { label: v.clone(), value: v }).collect();
                s.fill(list);
            }
        }
        game.folder = Some(folder);
        game.image = image;
        Ok(game)
    }

    /// The game's folder, when it was loaded from a path.
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    /// The arguments with `{renderer}`, `{device:SLOT}` and `{setting:KEY}` filled in from a
    /// `game.start`.
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
                while let Some(i) = a.find("{device:") {
                    let Some(end) = a[i..].find('}') else { break };
                    let value = crate::device::chosen(&start.data, &a[i + 8..i + end]).unwrap_or("").to_string();
                    a.replace_range(i..i + end + 1, &value);
                }
                if let Some(i) = &self.image {
                    a = a.replace("{image}", &absolute(i));
                }
                if let Some(f) = &self.folder {
                    a = a.replace("{folder}", &absolute(f));
                }
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

fn absolute(path: &Path) -> String {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()).display().to_string()
}
