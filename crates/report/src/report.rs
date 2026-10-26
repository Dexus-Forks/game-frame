//! One report: what happened, where in the port and in the original, the game's state, and how
//! to start the game again the same way. It reads and writes as JSON, and renders as the
//! Markdown body and the link of a GitHub issue.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// What a report is about.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The game reached a path the port has not ported (`unported!`).
    #[default]
    Unported,
    /// The port panicked.
    Panic,
    /// The port checked itself against the original and they differ (`mismatch!`).
    Mismatch,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Unported => "unported",
            Kind::Panic => "panic",
            Kind::Mismatch => "mismatch",
        }
    }
}

/// Where reports go. Dev keeps them on this machine; Production also gives each one a link to
/// GitHub's new-issue form, filled in, for the player to send.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// A developer's run: the reports folder and a line on standard error, no links.
    #[default]
    Dev,
    /// A player's build: a link to a filled-in issue, with the home folder written as `~`.
    Production,
}

/// The game a report comes from. `issues` is its repository on GitHub, `owner/name`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    pub game: String,
    pub version: String,
    /// The commit the program was built from, when the build knows it.
    pub commit: String,
    /// `owner/name` on GitHub, where the issue goes.
    pub issues: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// The same for every hit at one place: the kind, the file, the line and the original's
    /// address. A tool finds an issue already open for it by this id.
    pub id: String,
    pub kind: Kind,
    /// One line: what the port does not do, or the panic's message.
    pub what: String,
    /// The original's function or address this place stands for, such as `0x4315e8`.
    pub original: Option<String>,
    pub file: String,
    pub line: u32,
    pub column: u32,
    /// Values at the place itself, given to `unported!`.
    pub context: BTreeMap<String, Value>,
    /// The game's state the program keeps current with `set_state`: the frame, the track, the
    /// save.
    pub state: BTreeMap<String, Value>,
    /// The launch description, to start the game again the same way.
    pub launch: Value,
    /// The devices chosen in the frame, such as `{"video": "glide", "sound": "none"}`.
    pub devices: Value,
    pub meta: Meta,
    /// How many times the game came here.
    pub count: u64,
    /// Seconds since 1970 of the first and the last time.
    pub first: u64,
    pub last: u64,
    pub platform: String,
    /// The stack at the first time, when the build has symbols.
    pub backtrace: String,
    pub mode: Mode,
    /// In Production: the link to GitHub's new-issue form, filled in ([`Report::issue_url`]).
    pub issue: Option<String>,
}

/// The longest issue body a link carries; browsers and GitHub refuse longer addresses.
const LINK_BODY: usize = 6000;

impl Report {
    /// The report's id: the kind and an FNV-1a hash of the place, `unported-3f2a19c0`.
    pub fn id_for(kind: Kind, file: &str, line: u32, original: Option<&str>) -> String {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let place = format!("{}:{file}:{line}:{}", kind.name(), original.unwrap_or(""));
        for b in place.bytes() {
            h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
        format!("{}-{:08x}", kind.name(), h as u32 ^ (h >> 32) as u32)
    }

    pub fn from_json(text: &str) -> Result<Report, String> {
        serde_json::from_str(text).map_err(|e| format!("report: {e}"))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default() + "\n"
    }

    /// The issue's title: `[unported] the sky's fog at crates/render/src/sky.rs:88 (0x4a5e00)`.
    pub fn title(&self) -> String {
        let original = self.original.as_ref().map(|o| format!(" ({o})")).unwrap_or_default();
        let what: String = self.what.lines().next().unwrap_or("").chars().take(90).collect();
        format!("[{}] {what} at {}:{}{original}", self.kind.name(), self.file, self.line)
    }

    /// The issue's body in Markdown, laid out for an agent to work from: the place, the
    /// original's address, the state, the launch to reproduce it, the stack.
    pub fn markdown(&self) -> String {
        self.body(usize::MAX)
    }

    fn body(&self, limit: usize) -> String {
        let mut s = String::new();
        let game = if self.meta.game.is_empty() { "the game".to_string() } else { self.meta.game.clone() };
        s += &format!("<!-- gameframe-report: {} -->\n", self.id);
        s += &format!("**{}** in {game} {}", self.kind.name(), self.meta.version);
        if !self.meta.commit.is_empty() {
            s += &format!(" (`{}`)", self.meta.commit);
        }
        s += &format!(", {} time{}, on {}.\n\n", self.count, if self.count == 1 { "" } else { "s" }, self.platform);
        s += &format!("> {}\n\n", self.what.replace('\n', "\n> "));
        s += "| Where | |\n|---|---|\n";
        s += &format!("| Port | `{}:{}:{}` |\n", self.file, self.line, self.column);
        if let Some(o) = &self.original {
            s += &format!("| Original | `{o}` |\n");
        }
        s += &format!("| Report | `{}` |\n\n", self.id);
        let table = |title: &str, map: &BTreeMap<String, Value>| {
            if map.is_empty() {
                return String::new();
            }
            let rows: String = map.iter().map(|(k, v)| format!("| `{k}` | `{}` |\n", short(v))).collect();
            format!("### {title}\n\n| Key | Value |\n|---|---|\n{rows}\n")
        };
        s += &table("At the place", &self.context);
        s += &table("The game's state", &self.state);
        if !self.devices.is_null() {
            s += &format!("### Devices\n\n`{}`\n\n", self.devices);
        }
        if !self.launch.is_null() {
            s += "### Start it again\n\nThe launch description (`--launch FILE` for a port that takes one):\n\n```json\n";
            s += &serde_json::to_string_pretty(&self.launch).unwrap_or_default();
            s += "\n```\n\n";
        }
        s += "### For the agent\n\n";
        s += "1. Read the port at the place above and its `not ported` or `Deviation:` comment.\n";
        if let Some(o) = &self.original {
            s += &format!("2. Trace the original at `{o}` (`re-trace`): its inputs, what it writes, its callers.\n");
        } else {
            s += "2. Find the original's function for this place from the port's `Original:` citations (`re-trace`).\n";
        }
        s += "3. Start the game with the launch above, or write a test with the values at the place, and measure the original (`re-emulate`).\n";
        s += "4. Port it (`re-port`), check it (`re-verify`), and land it with this issue's number (`re-land`).\n\n";
        let tail = if self.backtrace.is_empty() {
            String::new()
        } else {
            format!("<details><summary>Stack</summary>\n\n```text\n{}\n```\n</details>\n", self.backtrace.trim_end())
        };
        if s.len() + tail.len() <= limit {
            s += &tail;
        } else if !tail.is_empty() {
            s += "The stack is in the report's JSON file.\n";
        }
        if s.len() > limit {
            let mut cut = limit;
            while !s.is_char_boundary(cut) {
                cut -= 1;
            }
            s.truncate(cut);
            s += "\n\n(Cut short: the whole report is in its JSON file.)\n";
        }
        s
    }

    /// Write the home folder as `~` everywhere in the report, before it leaves the machine.
    pub fn without_home(&mut self, home: &str) {
        if home.len() < 2 {
            return;
        }
        let fix = |s: &mut String| *s = s.replace(home, "~");
        fn fix_value(v: &mut Value, home: &str) {
            match v {
                Value::String(s) => *s = s.replace(home, "~"),
                Value::Array(a) => a.iter_mut().for_each(|v| fix_value(v, home)),
                Value::Object(o) => o.values_mut().for_each(|v| fix_value(v, home)),
                _ => {}
            }
        }
        fix(&mut self.what);
        fix(&mut self.file);
        fix(&mut self.backtrace);
        for map in [&mut self.context, &mut self.state] {
            map.values_mut().for_each(|v| fix_value(v, home));
        }
        fix_value(&mut self.launch, home);
    }

    /// A link that opens GitHub's new-issue form for this report, filled in, with the labels
    /// `gameframe-report` and the kind. `None` when the game names no repository. Long bodies
    /// lose the stack first; attach the JSON file then.
    pub fn issue_url(&self) -> Option<String> {
        let repo = self.meta.issues.trim().trim_start_matches("https://github.com/").trim_end_matches('/');
        if repo.split('/').filter(|p| !p.is_empty()).count() != 2 {
            return None;
        }
        Some(format!(
            "https://github.com/{repo}/issues/new?title={}&labels={}&body={}",
            encode(&self.title()),
            encode(&format!("gameframe-report,{}", self.kind.name())),
            encode(&self.body(LINK_BODY)),
        ))
    }
}

/// A value on one line, at most 120 characters.
fn short(v: &Value) -> String {
    let text = match v {
        Value::String(s) => s.clone(),
        v => v.to_string(),
    };
    let text = text.replace('|', "\\|").replace('\n', " ");
    if text.chars().count() > 120 {
        text.chars().take(117).collect::<String>() + "..."
    } else {
        text
    }
}

/// Percent-encoding for a query value.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out += &format!("%{b:02X}"),
        }
    }
    out
}
