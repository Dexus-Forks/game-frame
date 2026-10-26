//! Games outside Rust: another program that speaks the contract over JSON lines, and a game set
//! up from `frame.json` alone.
//!
//! **JSON lines.** [`Process`] starts the game's program. The program's first line on standard
//! output is its manifest. After that, the frame writes each event to the program's standard
//! input and reads the program's events from its standard output, one JSON object per line, as
//! [`crate::Event`] serialises them. Anything the program writes to standard error goes to the
//! console as a log line. A program in any language can do this with its standard library.
//!
//! **Config only.** [`Configured`] reads `frame.json`: a manifest and the command that starts
//! the game. On `game.start` it runs the command, with `{renderer}` and `{setting:KEY}` in the
//! arguments replaced, and reports `running` and then `stopped` when the program ends.

use crate::event::{topics, Event, Origin};
use crate::game::{Game, Outbox};
use crate::manifest::Manifest;
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};

pub struct Process {
    manifest: Manifest,
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<(bool, String)>,
}

impl Process {
    /// Start `program` with `args` and read its manifest from its first line.
    pub fn spawn(program: &str, args: &[String]) -> Result<Process, String> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{program}: {e}"))?;
        let stdin = child.stdin.take().ok_or("no standard input")?;
        let mut stdout = BufReader::new(child.stdout.take().ok_or("no standard output")?);
        let mut first = String::new();
        stdout.read_line(&mut first).map_err(|e| format!("{program}: {e}"))?;
        let manifest = Manifest::from_json(&first).map_err(|e| format!("{program}'s first line: {e}"))?;
        let (tx, lines) = channel();
        let out_tx = tx.clone();
        std::thread::spawn(move || {
            for line in stdout.lines().map_while(Result::ok) {
                if out_tx.send((false, line)).is_err() {
                    break;
                }
            }
        });
        let stderr = BufReader::new(child.stderr.take().ok_or("no standard error")?);
        std::thread::spawn(move || {
            for line in stderr.lines().map_while(Result::ok) {
                if tx.send((true, line)).is_err() {
                    break;
                }
            }
        });
        Ok(Process { manifest, child, stdin, lines })
    }
}

impl Game for Process {
    fn manifest(&self) -> Manifest {
        self.manifest.clone()
    }

    fn on_event(&mut self, e: &Event, out: &mut Outbox) {
        let line = serde_json::to_string(e).unwrap_or_default();
        if writeln!(self.stdin, "{line}").and_then(|_| self.stdin.flush()).is_err() {
            out.log("error", "the game's program is not reading its input");
        }
    }

    fn poll(&mut self, out: &mut Outbox) {
        while let Ok((stderr, line)) = self.lines.try_recv() {
            if stderr {
                out.log("info", &line);
                continue;
            }
            match serde_json::from_str::<Event>(&line) {
                Ok(mut e) => {
                    e.origin = Origin::Game;
                    out.events.push(e);
                }
                Err(_) => out.log("warn", &format!("the game's program wrote a line that is not an event: {line}")),
            }
        }
        if let Ok(Some(status)) = self.child.try_wait() {
            out.status("stopped", &format!("the game's program ended ({status})"));
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameFile {
    pub manifest: Manifest,
    pub start: Option<StartCommand>,
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
}

impl Configured {
    pub fn from_json(text: &str) -> Result<Configured, String> {
        let file: FrameFile = serde_json::from_str(text).map_err(|e| format!("frame.json: {e}"))?;
        file.manifest.check()?;
        Ok(Configured { file, child: None })
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
                match Command::new(&start.command).args(&args).spawn() {
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
                out.status("stopped", &format!("the game ended ({status})"));
            }
        }
    }
}
