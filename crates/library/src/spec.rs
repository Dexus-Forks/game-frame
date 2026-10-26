//! What the game declares: its sections and how the page shows them. It is read from the
//! project's `library.json`, the same file the HTML page reads, so the page and the game agree
//! on every name.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The whole `library.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Spec {
    /// The storage name: the folder name natively, the IndexedDB database in the browser.
    /// Lower case letters, digits and hyphens. Change it and the player's files are left behind.
    pub id: String,
    pub title: String,
    /// One line under the title.
    pub tagline: String,
    /// The logo picture, a path or URL the page loads.
    pub logo: Option<String>,
    /// A picture behind the page.
    pub backdrop: Option<String>,
    /// Design tokens: a token's name without `--lib-` to its CSS value, such as
    /// `"accent": "#4fb3ff"`. The names are in the template's `tokens.css`.
    pub theme: BTreeMap<String, String>,
    /// `dark` (the default), `light`, or `auto` to follow the player's system.
    pub scheme: Option<String>,
    /// A ready-made theme from the template's `themes/` folder, by name, such as
    /// `win98` (the template's README lists them). It is drawn after the components; `theme` tokens still win.
    pub preset: Option<String>,
    /// The themes the player may choose between, by name, in the frame's Settings tab. Empty
    /// (the default) offers no choice. `"none"` in the list offers the frame's own look.
    pub presets: Vec<String>,
    /// Icons replaced by pictures: an icon's name (`add`, `download`, `delete`, `up`, `down`,
    /// `start`, `file`, `missing`, `game`, `saves`, `mods`, `screenshots`, `recordings`) to a
    /// picture URL. The picture is used as a mask, so it takes the text colour.
    pub icons: BTreeMap<String, String>,
    /// The words on the start button.
    pub start: Option<String>,
    /// Links under the panel: help, source, issues, a Mastodon or GitHub page. Each may name
    /// an icon and a place in the frame.
    pub links: Vec<Link>,
    /// Where the frame offers to send a report from the running game (`gameframe-report`), one
    /// button each: the game's repository, GameFrame's, a fork's. Empty: the report's own.
    pub issues: Vec<IssueTarget>,
    /// The small print at the bottom: what the player needs, who made what.
    pub fine: Option<String>,
    /// Encrypt the kept files with a key this browser makes and never lets out (the
    /// default). A copy of the database is unreadable without that browser. Off, the files are
    /// kept as they are. Changing it leaves files kept the other way unreadable.
    pub encrypt: Option<bool>,
    /// Content around the files, drawn by the page's `library-panels.js`: Markdown, a
    /// changelog, messages, a video, releases, or a type the game adds. The crate keeps them for
    /// the page and does not read them.
    pub panels: Vec<Panel>,
    /// Markdown pages beside the library (help, guides), drawn by the page's
    /// `library-pages.js` at hash routes such as `#/help`. The crate does not read them.
    pub pages: Vec<Page>,
    /// The nav bar's name for the library itself (the home page). The default is "Library".
    pub home: Option<String>,
    /// In the order the page shows them. The game-data section is always first.
    pub sections: Vec<Section>,
}

/// A page: its route (`help` is `#/help`), its title, and its Markdown in a file or inline.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Page {
    pub path: String,
    pub title: String,
    pub src: Option<String>,
    pub text: Option<String>,
    pub icon: Option<String>,
}

/// A panel: its type, and the options that type reads (`src`, `title`, `place`, `limit`...).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub options: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub text: String,
    pub href: String,
    /// An icon by name, as in `icons` (`github`, `mastodon`, `home`), drawn before the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Where the frame draws it: `footer` (the default, under the panel), `header` (under the
    /// title) or `play` (under the start button). The HTML page draws every link in the footer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
}

impl Link {
    /// Whether the link belongs in `place`; one with no place is the footer's.
    pub fn is_in(&self, place: &str) -> bool {
        self.place.as_deref().unwrap_or("footer") == place
    }
}

/// A GitHub repository a report can go to, as one button.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueTarget {
    /// The button's words, such as "Report to Hardwar".
    pub text: String,
    /// `owner/name` on GitHub.
    pub repo: String,
    /// The report kinds it takes (`unported`, `panic`, `mismatch`); empty takes every kind.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<String>,
}

/// What a section holds. The kind sets the defaults; any field in [`Section`] overrides them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// The player's own copy of the game. Required: the game does not start without it.
    GameData,
    /// Saved games. Newest first, each can be downloaded.
    Saves,
    /// Mods: switched on and off, in the order they load.
    Mods,
    /// Pictures the game takes.
    Screenshots,
    /// Recordings or replays.
    Recordings,
    /// Anything else, under the section's own label.
    #[default]
    Custom,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Section {
    /// The key in storage: lower case letters, digits and hyphens.
    pub id: String,
    pub kind: Kind,
    pub label: String,
    /// The section's icon: a built-in icon's name or a picture URL. The default follows the kind.
    pub icon: Option<String>,
    /// Paragraphs of help. Text in `backticks` is shown as code.
    pub help: Vec<String>,
    /// Numbered steps, shown before the file chooser (the game-data section uses them).
    pub steps: Vec<String>,
    /// File name endings the section takes, such as ".sav". Empty takes any file.
    pub accept: Vec<String>,
    /// Files the game needs here. Names match case-blind.
    pub required: Vec<Required>,
    /// More than one file. The default is true.
    pub many: Option<bool>,
    /// The player sets the order. The default is true for mods.
    pub ordered: Option<bool>,
    /// The player switches each file on and off. The default is true for mods.
    pub toggles: Option<bool>,
    /// Each file can be downloaded. The default is true except for game data.
    pub download: Option<bool>,
    /// The player can add files from the page. The default is true.
    pub import: Option<bool>,
    /// Shown when the section is empty.
    pub empty: Option<String>,
}

/// A file the game needs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Required {
    pub name: String,
    /// The size in bytes, when every release has the same file.
    pub size: Option<u64>,
    /// Where the player finds it, shown when it is missing.
    pub hint: Option<String>,
}

impl Section {
    pub fn many(&self) -> bool {
        self.many.unwrap_or(true)
    }
    pub fn ordered(&self) -> bool {
        self.ordered.unwrap_or(self.kind == Kind::Mods)
    }
    pub fn toggles(&self) -> bool {
        self.toggles.unwrap_or(self.kind == Kind::Mods)
    }
    pub fn download(&self) -> bool {
        self.download.unwrap_or(self.kind != Kind::GameData)
    }
    pub fn import(&self) -> bool {
        self.import.unwrap_or(true)
    }
    /// Whether the section takes a file of this name.
    pub fn accepts(&self, name: &str) -> bool {
        let n = name.to_ascii_lowercase();
        self.accept.is_empty() || self.accept.iter().any(|e| n.ends_with(&e.to_ascii_lowercase()))
    }
}

impl Spec {
    pub fn from_json(text: &str) -> Result<Spec, String> {
        let spec: Spec = serde_json::from_str(text).map_err(|e| format!("library.json: {e}"))?;
        spec.check()?;
        Ok(spec)
    }

    pub fn encrypt(&self) -> bool {
        self.encrypt.unwrap_or(true)
    }

    pub fn section(&self, id: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// The first section of a kind, such as the saves.
    pub fn of_kind(&self, kind: Kind) -> Option<&Section> {
        self.sections.iter().find(|s| s.kind == kind)
    }

    /// The names and the shape the page and the stores rely on.
    pub fn check(&self) -> Result<(), String> {
        let name_ok = |s: &str| {
            !s.is_empty()
                && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if !name_ok(&self.id) {
            return Err(format!("library.json: id {:?} must be lower case letters, digits and hyphens", self.id));
        }
        let mut seen = Vec::new();
        for s in &self.sections {
            if !name_ok(&s.id) {
                return Err(format!("library.json: section id {:?} must be lower case letters, digits and hyphens", s.id));
            }
            if seen.contains(&&s.id) {
                return Err(format!("library.json: section {} is listed twice", s.id));
            }
            seen.push(&s.id);
        }
        let data = self.sections.iter().filter(|s| s.kind == Kind::GameData).count();
        if data != 1 {
            return Err(format!("library.json: needs one game-data section, has {data}"));
        }
        Ok(())
    }
}
