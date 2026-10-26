//! A game whose program runs one game and ends: [`Respawn`] starts it again for each start.
//!
//! A port that opens its own window with winit can make the window only once in a process, so
//! its program runs one game and exits. The frame still needs a program to answer it before the
//! first start (the manifest, the choices it fills). [`Respawn`] keeps one program running for
//! that, gives it the first `game.start`, and starts a new program for each start after it, and
//! for a start whose arguments changed, such as a disc added in the Library tab. Everything else
//! passes through to the program as [`Process`] does. The ports of POD and Hardwar run their
//! games this way.

use crate::event::{topics, Event};
use crate::game::{Game, Outbox};
use crate::manifest::Manifest;
use crate::process::Process;

/// The program's arguments, made again for each start, so they can name files the player
/// added since.
pub type Args = Box<dyn Fn() -> Vec<String>>;

pub struct Respawn {
    program: String,
    args: Args,
    /// The arguments the running program was started with.
    started_with: Vec<String>,
    process: Process,
    manifest: Manifest,
    /// The running program has been given a `game.start`.
    used: bool,
}

impl Respawn {
    /// Start the first program, which answers the frame until the player starts a game.
    pub fn spawn(program: &str, args: Args) -> Result<Respawn, String> {
        let started_with = args();
        let process = Process::spawn(program, &started_with)?;
        let manifest = process.manifest();
        Ok(Respawn { program: program.into(), args, started_with, process, manifest, used: false })
    }
}

impl Game for Respawn {
    fn manifest(&self) -> Manifest {
        self.manifest.clone()
    }

    fn on_event(&mut self, e: &Event, out: &mut Outbox) {
        if e.topic == topics::GAME_START {
            let args = (self.args)();
            if self.used || args != self.started_with {
                // The start carries every setting, so the new program needs no `frame.ready`.
                match Process::spawn(&self.program, &args) {
                    Ok(p) => self.process = p,
                    Err(err) => return out.status("failed", &err),
                }
                self.started_with = args;
            }
            self.used = true;
        }
        self.process.on_event(e, out);
    }

    fn poll(&mut self, out: &mut Outbox) {
        self.process.poll(out);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::event::Origin;
    use crate::{Session, Settings};
    use serde_json::json;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    /// A program in a shell script: its manifest, then one game, then it ends. It names its
    /// process (`$$`) in its status.
    const SCRIPT: &str = r#"echo '{"contract":"1.0","name":"sh","capabilities":["start"]}'
while read -r line; do
  case "$line" in
    *'"game.start"'*) echo "{\"topic\":\"game.status\",\"data\":{\"state\":\"stopped\",\"detail\":\"$$\"},\"from\":\"game\"}"; exit 0 ;;
  esac
done"#;

    fn script(extra: Option<&str>) -> Vec<String> {
        let mut a = vec!["-c".to_string(), SCRIPT.to_string()];
        a.extend(extra.map(String::from));
        a
    }

    #[test]
    fn each_start_runs_a_new_program() {
        let game = Respawn::spawn("sh", Box::new(|| script(None))).unwrap();
        let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
        let ends = Rc::new(RefCell::new(Vec::new()));
        let keep = ends.clone();
        s.on("game.status", move |e| keep.borrow_mut().push(e.str("detail").unwrap_or("").to_string()));
        for game in 1..=2 {
            s.start(vec![]);
            for _ in 0..400 {
                s.tick();
                if ends.borrow().len() == game {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let ends = ends.borrow();
        assert_eq!(ends.len(), 2, "{ends:?}");
        assert_ne!(ends[0], ends[1], "the second game ran in a new program");
    }

    #[test]
    fn a_start_with_new_arguments_runs_a_new_program() {
        let disc = Rc::new(RefCell::new(None));
        let added = disc.clone();
        let mut g = Respawn::spawn("sh", Box::new(move || script(*added.borrow()))).unwrap();
        *disc.borrow_mut() = Some("disc");
        let mut out = Outbox::default();
        g.on_event(&Event::new(topics::GAME_START, Origin::Frame, json!({})), &mut out);
        assert_eq!(g.started_with.last().map(String::as_str), Some("disc"));
    }
}
