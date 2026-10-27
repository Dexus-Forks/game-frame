//! Reports from a running port, for the paths it has not ported yet and for its crashes.
//!
//! A port marks each piece it leaves out with a `not ported` comment (the re-port skill). Where
//! the game can reach such a piece while it runs, the port calls [`unported!`] there too. The
//! first time the game gets there, this crate writes a report: the place in the port, the
//! original's address, the values at the place, the game's state, the launch description and
//! the devices, so the run can be repeated. Later times only count. A panic becomes a report
//! the same way ([`install`] sets the hook).
//!
//! ```no_run
//! use gameframe_report as report;
//! report::install(report::Meta { game: "Hardwar".into(), version: "0.4.0".into(),
//!     issues: "tanepiper/hardwar".into(), ..Default::default() });
//! report::set_dir("reports");            // <id>.json and <id>.md for each place
//! report::set_state("frame", 1234);      // kept current by the game loop
//! let material = 7;
//! report::unported!("0x5898b4", "the material of a hit's other side", material = material);
//! ```
//!
//! Each report is one JSON file and one Markdown file in the folder, named by its id. The
//! Markdown is a GitHub issue's body, laid out for an agent to debug from; [`Report::issue_url`]
//! opens GitHub's form filled in, and `tools/triage/report_issue.py` files it with `gh`, once
//! per id. [`on_report`] passes each new report on, such as to the frame as `report.new`.
//!
//! Two modes ([`Mode`]). **Dev**, the default in a debug build, keeps the reports on this
//! machine: the folder and one line on standard error. **Production**, the default in a release
//! build, also writes the home folder as `~` and gives each report a link to GitHub's
//! new-issue form, filled in ([`Report::issue`]); the frame shows it as a button, and the
//! player reads the issue and sends it from the browser. `GAMEFRAME_REPORT=dev` or
//! `production`, or [`set_mode`], chooses.
//!
//! It does not stop the game: an unported path goes on as the port wrote it. With
//! `GAMEFRAME_STRICT=1` in the environment, or [`set_strict`], it panics instead, for tests.
//! It reads no files and sends nothing over the network: a link only opens the form.

mod report;

pub use report::{Kind, Meta, Mode, Report};
#[doc(hidden)]
pub use serde_json;

use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

type Sink = Box<dyn Fn(&Report) + Send>;

#[derive(Default)]
struct State {
    meta: Meta,
    dir: Option<PathBuf>,
    state: BTreeMap<String, Value>,
    launch: Value,
    devices: Value,
    seen: HashMap<String, Report>,
    sinks: Vec<Sink>,
    strict: Option<bool>,
    mode: Option<Mode>,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(Default::default)
}

fn with<T>(f: impl FnOnce(&mut State) -> T) -> T {
    let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
    f(&mut s)
}

/// Name the game and turn panics into reports. Call it once, early in `main`.
pub fn install(meta: Meta) {
    with(|s| s.meta = meta);
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a panic".into());
        if let Some(at) = info.location() {
            // A panic inside the reporter itself must not wait for its own lock.
            if let Ok(mut s) = state().try_lock() {
                record(&mut s, Kind::Panic, &what, None, (at.file(), at.line(), at.column()), BTreeMap::new());
            }
        }
        previous(info);
    }));
}

/// Where the reports go: `<id>.json` and `<id>.md` for each place. Without a folder they are
/// only kept in memory ([`reports`]) and passed to [`on_report`].
pub fn set_dir(dir: impl Into<PathBuf>) {
    with(|s| s.dir = Some(dir.into()));
}

/// One value of the game's state, written into every report from now on: the frame number,
/// the track, the save. A game updates it as it runs.
pub fn set_state(key: &str, value: impl serde::Serialize) {
    let value = serde_json::to_value(value).unwrap_or(Value::Null);
    with(|s| s.state.insert(key.to_string(), value));
}

/// The launch description the game started from, to start it again the same way.
pub fn set_launch(launch: impl serde::Serialize) {
    let launch = serde_json::to_value(launch).unwrap_or(Value::Null);
    with(|s| s.launch = launch);
}

/// The devices chosen in the frame (`game.start`'s `devices`).
pub fn set_devices(devices: Value) {
    with(|s| s.devices = devices);
}

/// Call `f` with each new report, the first time the game reaches its place.
pub fn on_report(f: impl Fn(&Report) + Send + 'static) {
    with(|s| s.sinks.push(Box::new(f)));
}

/// Dev or Production. The default is `GAMEFRAME_REPORT=dev` or `production` from the
/// environment, else Dev in a debug build and Production in a release build.
pub fn set_mode(mode: Mode) {
    with(|s| s.mode = Some(mode));
}

/// The mode reports are made in now.
pub fn mode() -> Mode {
    with(|s| current_mode(s))
}

fn current_mode(s: &State) -> Mode {
    s.mode.unwrap_or_else(|| match std::env::var("GAMEFRAME_REPORT").as_deref() {
        Ok("dev") => Mode::Dev,
        Ok("production") => Mode::Production,
        _ if cfg!(debug_assertions) => Mode::Dev,
        _ => Mode::Production,
    })
}

/// Panic at an unported path instead of going on. The default is `GAMEFRAME_STRICT=1`.
pub fn set_strict(on: bool) {
    with(|s| s.strict = Some(on));
}

/// Every report so far, the most frequent first.
pub fn reports() -> Vec<Report> {
    let mut all: Vec<Report> = with(|s| s.seen.values().cloned().collect());
    all.sort_by(|a, b| b.count.cmp(&a.count).then(a.id.cmp(&b.id)));
    all
}

/// Write every report's file again with its last count: call it when the game ends.
pub fn flush() {
    with(|s| {
        for r in s.seen.values() {
            write(&s.dir, r);
        }
    });
}

/// What [`unported!`] and [`mismatch!`] call. `at` is the file, line and column.
pub fn hit(kind: Kind, what: &str, original: Option<&str>, at: (&str, u32, u32), context: Vec<(&str, Value)>) {
    let context = context.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    let strict = with(|s| {
        record(s, kind, what, original, at, context);
        s.strict.unwrap_or_else(|| std::env::var("GAMEFRAME_STRICT").is_ok_and(|v| v == "1"))
    });
    if strict && kind != Kind::Panic {
        panic!("{}: {what} ({}:{})", kind.name(), at.0, at.1);
    }
}

fn record(s: &mut State, kind: Kind, what: &str, original: Option<&str>, at: (&str, u32, u32), context: BTreeMap<String, Value>) {
    let now = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let id = Report::id_for(kind, at.0, at.1, original);
    if let Some(r) = s.seen.get_mut(&id) {
        r.count += 1;
        r.last = now;
        r.context = context;
        // The file again at 2, 4, 8 ... hits, so a long run does not write on every frame.
        if r.count.is_power_of_two() {
            write(&s.dir, r);
        }
        return;
    }
    let mode = current_mode(s);
    let mut r = Report {
        id: id.clone(),
        kind,
        what: what.to_string(),
        original: original.map(str::to_string),
        file: at.0.to_string(),
        line: at.1,
        column: at.2,
        context,
        state: s.state.clone(),
        launch: s.launch.clone(),
        devices: s.devices.clone(),
        meta: s.meta.clone(),
        count: 1,
        first: now,
        last: now,
        platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        backtrace: std::backtrace::Backtrace::force_capture().to_string(),
        mode,
        issue: None,
    };
    match mode {
        Mode::Dev => {
            let at = s.dir.as_ref().map(|d| format!(", {}", d.join(format!("{}.md", r.id)).display())).unwrap_or_default();
            eprintln!("report: {}{at}", r.title());
        }
        Mode::Production => {
            let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
            r.without_home(&home);
            r.issue = r.issue_url();
        }
    }
    write(&s.dir, &r);
    for f in &s.sinks {
        f(&r);
    }
    s.seen.insert(id, r);
}

fn write(dir: &Option<PathBuf>, r: &Report) {
    let Some(dir) = dir else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let _ = std::fs::write(dir.join(format!("{}.json", r.id)), r.to_json());
    let _ = std::fs::write(dir.join(format!("{}.md", r.id)), format!("# {}\n\n{}", r.title(), r.markdown()));
}

/// The game reached a path the port has not ported. It goes on as the port wrote it; the
/// first time at each place a report is written.
///
/// `unported!("0x4315e8", "the weather's thunder", kind = k)`: the original's address or
/// function, one line saying what is missing, then any values at the place as `name = value`
/// (anything serde can write).
#[macro_export]
macro_rules! unported {
    ($original:expr, $what:expr $(, $key:ident = $value:expr)* $(,)?) => {
        $crate::hit(
            $crate::Kind::Unported,
            ::core::convert::AsRef::<str>::as_ref(&$what),
            Some(::core::convert::AsRef::<str>::as_ref(&$original)),
            (file!(), line!(), column!()),
            vec![$((stringify!($key), $crate::serde_json::to_value(&$value).unwrap_or_default())),*],
        )
    };
}

/// The port checked a value against the original's and they differ, as [`unported!`] takes
/// them: `mismatch!("0x489c1c", "fire control's stream A", ours = a, original = b)`.
#[macro_export]
macro_rules! mismatch {
    ($original:expr, $what:expr $(, $key:ident = $value:expr)* $(,)?) => {
        $crate::hit(
            $crate::Kind::Mismatch,
            ::core::convert::AsRef::<str>::as_ref(&$what),
            Some(::core::convert::AsRef::<str>::as_ref(&$original)),
            (file!(), line!(), column!()),
            vec![$((stringify!($key), $crate::serde_json::to_value(&$value).unwrap_or_default())),*],
        )
    };
}
