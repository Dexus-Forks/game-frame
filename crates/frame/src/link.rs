//! The other end of [`crate::Process`]: a port's own program, started by the frame, speaking
//! JSON lines on its standard input and output.
//!
//! [`Link::wait`] prints the game's manifest, then hands each event from the frame to the
//! game's [`Game`] until `start` takes a launch from it (usually after `game.start`). It returns
//! that launch and a [`Link`]. A thread then reads the frame's events while the game runs:
//! `game.stop`, `frame.exit` or the frame closing its end sets [`Link::stopping`], which the
//! game's loop checks to end as its own quit does. The game reports with [`Link::status`] and
//! [`Link::send`]. Anything else the program prints must go to standard error.
//!
//! ```no_run
//! # use gameframe_frame::{Demo, Link};
//! let mut game = Demo::default(); // the port's own `Game`: manifest, settings, start
//! let Some((launch, link)) = Link::wait(&mut game, |_| Some("the launch")) else { return };
//! link.status("running", "in its window");
//! while !link.stopping() { /* one frame of the game */ break; }
//! link.status("stopped", "the game ended");
//! ```

use crate::event::{topics, Event, Origin};
use crate::game::{Game, Outbox};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

type Writer = Arc<Mutex<Box<dyn Write + Send>>>;

/// The running game's line to the frame. Clone it into the parts of the game that report.
#[derive(Clone)]
pub struct Link {
    out: Writer,
    stop: Arc<AtomicBool>,
}

fn write_line(out: &Writer, text: &str) {
    if let Ok(mut w) = out.lock() {
        let _ = writeln!(w, "{text}");
        let _ = w.flush();
    }
}

fn write_event(out: &Writer, e: &Event) {
    write_line(out, &serde_json::to_string(e).unwrap_or_default());
}

impl Link {
    /// On standard input and output: see [`Link::wait_on`].
    pub fn wait<G: Game, T>(game: &mut G, start: impl FnMut(&mut G) -> Option<T>) -> Option<(T, Link)> {
        Link::wait_on(game, start, std::io::BufReader::new(std::io::stdin()), std::io::stdout())
    }

    /// Print the game's manifest on `output`, then pass each event read from `input` to the
    /// game, writing what it sends, until `start` returns a launch. `None` when the frame sent
    /// `frame.exit` or closed `input` first. A line that is not an event is answered with a
    /// warning.
    pub fn wait_on<G: Game, T>(
        game: &mut G,
        mut start: impl FnMut(&mut G) -> Option<T>,
        mut input: impl BufRead + Send + 'static,
        output: impl Write + Send + 'static,
    ) -> Option<(T, Link)> {
        let out: Writer = Arc::new(Mutex::new(Box::new(output)));
        write_line(&out, &serde_json::to_string(&game.manifest()).unwrap_or_default());
        let mut outbox = Outbox::default();
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line).ok()? == 0 {
                return None;
            }
            match serde_json::from_str::<Event>(line.trim_end()) {
                Ok(e) if e.topic == topics::FRAME_EXIT => return None,
                Ok(e) => game.on_event(&e, &mut outbox),
                Err(_) => outbox.log("warn", &format!("a line that is not an event: {}", line.trim_end())),
            }
            for e in outbox.take() {
                write_event(&out, &e);
            }
            if let Some(launch) = start(game) {
                let link = Link { out, stop: Arc::new(AtomicBool::new(false)) };
                let stop = link.stop.clone();
                std::thread::spawn(move || listen(input, stop));
                return Some((launch, link));
            }
        }
    }

    /// Whether the frame asked the running game to stop, or went away.
    pub fn stopping(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    /// An event to the frame.
    pub fn send(&self, topic: &str, data: Value) {
        write_event(&self.out, &Event::new(topic, Origin::Game, data));
    }

    /// Send each new report (`gameframe_report`) to the frame as `report.new`, so Play lists it
    /// with its GitHub link. Also gives the reports the launch's devices.
    pub fn send_reports(&self, devices: Value) {
        gameframe_report::set_devices(devices);
        let link = self.clone();
        gameframe_report::on_report(move |r| link.send(topics::REPORT_NEW, serde_json::to_value(r).unwrap_or(Value::Null)));
    }

    /// `game.status`: `running`, `paused`, `stopped` or `failed`, with a line saying why.
    pub fn status(&self, state: &str, detail: &str) {
        self.send(topics::GAME_STATUS, json!({ "state": state, "detail": detail }));
    }
}

/// While the game runs: a stop or an exit from the frame, or the frame gone, stops it. The
/// frame's other events wait for the next start (a setting changed while the game runs takes
/// effect then).
fn listen(input: impl BufRead, stop: Arc<AtomicBool>) {
    for line in input.lines().map_while(Result::ok) {
        if let Ok(e) = serde_json::from_str::<Event>(&line) {
            if e.topic == topics::GAME_STOP || e.topic == topics::FRAME_EXIT {
                break;
            }
        }
    }
    stop.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, CONTRACT};
    use std::io::Cursor;
    use std::time::Duration;

    /// What the link wrote, readable after the writer is moved into it.
    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);

    impl Write for Shared {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Shared {
        fn lines(&self) -> Vec<String> {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap().lines().map(String::from).collect()
        }
    }

    fn line(topic: &str, data: Value) -> String {
        serde_json::to_string(&Event::new(topic, Origin::Frame, data)).unwrap() + "\n"
    }

    /// A game that takes its launch from `game.start`.
    #[derive(Default)]
    struct Kernel {
        launch: Option<Value>,
    }

    impl Game for Kernel {
        fn manifest(&self) -> Manifest {
            Manifest { contract: CONTRACT.into(), name: "kernel".into(), ..Manifest::default() }
        }
        fn on_event(&mut self, e: &Event, out: &mut Outbox) {
            if e.topic == topics::GAME_START {
                self.launch = Some(e.data.clone());
                out.status("loading", "");
            }
        }
    }

    fn started(game: &mut Kernel) -> Option<Value> {
        game.launch.take()
    }

    #[test]
    fn the_manifest_first_then_events_until_the_start() {
        let input = line(topics::FRAME_READY, json!({"renderer": "", "settings": {}}))
            + "not an event\n"
            + &line(topics::GAME_START, json!({"renderer": "", "settings": {}, "args": []}))
            + &line(topics::GAME_STOP, Value::Null);
        let out = Shared::default();
        let mut game = Kernel::default();
        let (launch, link) = Link::wait_on(&mut game, started, Cursor::new(input), out.clone()).unwrap();
        assert_eq!(launch["args"], json!([]));
        let lines = out.lines();
        assert!(lines[0].contains("\"contract\""), "{}", lines[0]);
        assert!(lines.iter().any(|l| l.contains("not an event")));
        for _ in 0..200 {
            if link.stopping() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(link.stopping(), "game.stop stops the game");
        link.status("stopped", "done");
        assert!(out.lines().last().unwrap().contains("\"stopped\""));
    }

    #[test]
    fn an_exit_or_the_frame_gone_before_a_start_is_none() {
        let exit = line(topics::FRAME_EXIT, Value::Null);
        assert!(Link::wait_on(&mut Kernel::default(), started, Cursor::new(exit), Shared::default()).is_none());
        assert!(Link::wait_on(&mut Kernel::default(), started, Cursor::new(""), Shared::default()).is_none());
    }

    #[test]
    fn a_report_reaches_the_frame() {
        let input = line(topics::GAME_START, json!({"renderer": "software", "settings": {}, "args": []}));
        let out = Shared::default();
        let mut game = Kernel::default();
        let (launch, link) = Link::wait_on(&mut game, started, Cursor::new(input), out.clone()).unwrap();
        gameframe_report::set_mode(gameframe_report::Mode::Dev);
        link.send_reports(json!({"video": launch["renderer"]}));
        gameframe_report::unported!("0x4315e8", "the weather's thunder");
        let sent = out.lines().into_iter().find(|l| l.contains(topics::REPORT_NEW)).expect("report.new");
        let e: Event = serde_json::from_str(&sent).unwrap();
        assert_eq!((e.str("original"), e.data["devices"]["video"].as_str()), (Some("0x4315e8"), Some("software")));
    }
}
