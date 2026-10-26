//! The saved settings: the game's (keyed as its manifest says) and the frame's own, whose keys
//! start with `frame.` (`frame.renderer`, `frame.show-at-start`, `frame.on-exit`). Saved as one JSON object.

use serde_json::{Map, Value};

/// Whether the frame opens when the program starts: the default on the first run.
pub const SHOW_AT_START: &str = "frame.show-at-start";
/// The ready-made theme the player chose, by name, or `"none"` for the frame's own look. The
/// frame offers the choice only when `library.json` lists `presets`.
pub const PRESET: &str = "frame.preset";

/// What happens when the game ends by itself (the player quit it, or its program ended):
/// `"launcher"`, the default, shows the frame again; `"desktop"` leaves the program. A stop the
/// player asked for in the frame, and a game that failed, always show the frame.
pub const ON_EXIT: &str = "frame.on-exit";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Settings {
    values: Map<String, Value>,
    changed: bool,
}

impl Settings {
    pub fn new() -> Settings {
        Settings::default()
    }

    /// Settings saved before, from their JSON text. Text that does not read gives no settings.
    pub fn from_json(text: &str) -> Settings {
        let values = serde_json::from_str::<Map<String, Value>>(text).unwrap_or_default();
        Settings { values, changed: false }
    }

    pub fn to_json(&self) -> Value {
        Value::Object(self.values.clone())
    }

    pub fn to_text(&self) -> String {
        serde_json::to_string_pretty(&self.values).unwrap_or_default() + "\n"
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        if self.values.get(key) != Some(&value) {
            self.values.insert(key.to_string(), value);
            self.changed = true;
        }
    }

    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// Whether to open the frame when the program starts. `--frame` (or `--launcher`) on the
    /// command line always opens it and `--no-frame` never does. Otherwise the saved
    /// `frame.show-at-start` decides, and it is on until the player turns it off.
    pub fn show_at_start(&self, args: &[String]) -> bool {
        if args.iter().any(|a| a == "--frame" || a == "--launcher") {
            return true;
        }
        if args.iter().any(|a| a == "--no-frame") {
            return false;
        }
        self.get(SHOW_AT_START).and_then(Value::as_bool).unwrap_or(true)
    }
}

/// Settings in a JSON file on the desktop, such as `CONFIG_DIR/my-port/settings.json`.
#[cfg(not(target_arch = "wasm32"))]
pub mod file {
    use super::Settings;
    use std::path::Path;

    /// The saved settings, or none on the first run.
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path).map(|t| Settings::from_json(&t)).unwrap_or_default()
    }

    pub fn save(path: &Path, settings: &Settings) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, settings.to_text())?;
        std::fs::rename(tmp, path)
    }
}
