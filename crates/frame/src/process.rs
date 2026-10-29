//! A game outside Rust: another program that speaks the contract over JSON lines.
//!
//! [`Process`] starts the game's program. The program's first line on standard
//! output is its manifest. After that, the frame writes each event to the program's standard
//! input and reads the program's events from its standard output, one JSON object per line, as
//! [`crate::Event`] serialises them. Anything the program writes to standard error goes to the
//! console as a log line. A program in any language can do this with its standard library.
//!
//! When the program ends, the frame hears it once, after the program's last lines: `stopped` for
//! a program that exited successfully, `failed` with its exit status and its last line on
//! standard error for one that did not. A program that said `stopped` or `failed` itself as its
//! last status and then exited successfully gets no second status.

use crate::event::{topics, Event, Origin};
use crate::game::{Game, Outbox};
use crate::manifest::Manifest;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::{Duration, Instant};

/// How long to wait for the last lines of a program that has exited, when something else still
/// holds its output open (a program it started).
const LAST_LINES: Duration = Duration::from_secs(1);

pub struct Process {
    manifest: Manifest,
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<(bool, String)>,
    /// The game's last `game.status` state, from its own events.
    last_state: Option<String>,
    /// The last line on standard error, for the reason a program failed.
    last_error: Option<String>,
    /// When the program was seen to have exited, and whether that was reported.
    exited: Option<(std::process::ExitStatus, Instant)>,
    reported: bool,
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
        Ok(Process { manifest, child, stdin, lines, last_state: None, last_error: None, exited: None, reported: false })
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
        let drained = loop {
            match self.lines.try_recv() {
                Ok((true, line)) => {
                    out.log("info", &line);
                    self.last_error = Some(line);
                }
                Ok((false, line)) => match serde_json::from_str::<Event>(&line) {
                    Ok(mut e) => {
                        e.origin = Origin::Game;
                        if e.topic == topics::GAME_STATUS {
                            self.last_state = e.str("state").map(str::to_string);
                        }
                        out.events.push(e);
                    }
                    Err(_) => out.log("warn", &format!("the game's program wrote a line that is not an event: {line}")),
                },
                Err(TryRecvError::Empty) => break false,
                Err(TryRecvError::Disconnected) => break true,
            }
        };
        if self.reported {
            return;
        }
        if self.exited.is_none() {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.exited = Some((status, Instant::now()));
            }
        }
        let Some((status, at)) = self.exited else { return };
        // Its last lines first, so its own last status comes before ours.
        if !drained && at.elapsed() < LAST_LINES {
            return;
        }
        self.reported = true;
        let said_end = matches!(self.last_state.as_deref(), Some("stopped" | "failed"));
        if status.success() {
            if !said_end {
                out.status("stopped", &format!("the game's program ended ({status})"));
            }
        } else {
            let why = self.last_error.as_deref().map_or(String::new(), |l| format!(": {l}"));
            out.status("failed", &format!("the game's program ended ({status}){why}"));
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
