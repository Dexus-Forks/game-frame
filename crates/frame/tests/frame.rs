//! The contract: events, the manifest's checks, a session driving the demo game, the first-run
//! rule, and the adapters for games outside Rust.

use gameframe_frame::{topics, Configured, Demo, Event, Game, Manifest, Origin, Outbox, Session, Settings};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

fn demo() -> Session {
    let mut s = Session::new(Box::new(Demo::default()), Settings::new()).unwrap();
    s.ready();
    s
}

#[test]
fn topics_match_patterns() {
    let e = Event::new("game.start", Origin::Frame, json!(null));
    assert!(e.matches("game.start") && e.matches("game.*") && e.matches("*"));
    assert!(!e.matches("game") && !e.matches("gamex.*") && !e.matches("game.stop"));
    assert!(gameframe_frame::valid_topic("pod.ghost.load"));
    assert!(!gameframe_frame::valid_topic("Game.Start") && !gameframe_frame::valid_topic("start"));
}

#[test]
fn events_have_one_wire_form() {
    let e = Event::new("config.set", Origin::Frame, json!({"key": "fog", "value": true}));
    let text = serde_json::to_string(&e).unwrap();
    assert_eq!(text, r#"{"topic":"config.set","data":{"key":"fog","value":true},"from":"frame","id":0}"#);
    assert_eq!(serde_json::from_str::<Event>(&text).unwrap(), e);
}

#[test]
fn a_session_fills_defaults_and_keeps_state() {
    let mut s = demo();
    assert_eq!(s.settings().get("video.fog"), Some(&json!(true)));
    assert_eq!(s.renderer(), Some("software"));
    assert!(s.set("video.fog", json!("yes")).is_err(), "a toggle takes a bool");
    assert!(s.set("nothing", json!(1)).is_err());
    s.set("sound.volume", json!(40)).unwrap();
    s.select_renderer("glide").unwrap();
    assert!(s.select_renderer("vulkan").is_err());
    assert!(s.settings_changed());
    assert_eq!(s.settings().get("frame.renderer"), Some(&json!("glide")));
    // Saved settings come back, with the chosen renderer.
    let again = Session::new(Box::new(Demo::default()), Settings::from_json(&s.settings().to_text())).unwrap();
    assert_eq!(again.renderer(), Some("glide"));
    assert_eq!(again.settings().get("sound.volume"), Some(&json!(40)));
}

#[test]
fn the_demo_answers_on_the_console() {
    let mut s = demo();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = seen.clone();
    s.on("game.*", move |e| log.borrow_mut().push(e.topic.clone()));
    s.press("fire", true);
    s.start(vec![]);
    assert_eq!(s.state(), "loading");
    for _ in 0..30 {
        s.tick();
    }
    assert_eq!(s.state(), "running");
    s.press("fire", true);
    s.stop();
    assert_eq!(s.state(), "stopped");
    let lines: Vec<String> = s.log().iter().filter(|e| e.topic == topics::LOG).filter_map(|e| e.str("text").map(String::from)).collect();
    assert!(lines.iter().any(|l| l == "pressed fire: the game is not running"));
    assert!(lines.iter().any(|l| l == "pressed fire: the game reacts"));
    assert_eq!(*seen.borrow(), ["game.start", "game.status", "game.status", "game.stop", "game.status"]);
    let ids: Vec<u64> = s.log().iter().map(|e| e.id).collect();
    assert!(ids.windows(2).all(|w| w[1] == w[0] + 1), "events are numbered in order");
}

#[test]
fn the_demo_presses_its_own_controls() {
    let mut s = demo();
    for _ in 0..gameframe_frame::DEMO_PRESS_TICKS * 2 {
        s.tick();
    }
    let presses: Vec<&str> = s.log().iter().filter(|e| e.topic == "demo.press").filter_map(|e| e.str("control")).collect();
    assert_eq!(presses, ["up", "right"]);
    assert!(s.log().iter().filter(|e| e.topic == "demo.press").all(|e| e.origin == Origin::Game));
}

#[test]
fn a_newer_minor_contract_still_reads() {
    let m: Manifest = serde_json::from_value(serde_json::json!({
        "contract": "1.7", "name": "Later", "viewer": { "decoders": [] },
        "renderers": [{ "id": "soft", "label": "Software", "grade": "exact", "shader": "crt" }]
    }))
    .unwrap();
    assert!(m.check().is_ok());
    assert_eq!(m.renderers[0].id, "soft");
}

#[test]
fn bad_topics_and_contracts_are_refused() {
    struct Loud;
    impl Game for Loud {
        fn manifest(&self) -> Manifest {
            Manifest { contract: "1.0".into(), ..Manifest::default() }
        }
        fn on_event(&mut self, _: &Event, out: &mut Outbox) {
            out.send("NOT A TOPIC", json!(null));
        }
    }
    let mut s = Session::new(Box::new(Loud), Settings::new()).unwrap();
    s.send("frame.ready", json!(null));
    assert!(s.log().iter().any(|e| e.str("text").is_some_and(|t| t.contains("dropped an event"))));
    assert!(Manifest::from_json(r#"{"contract": "2.0", "name": "x"}"#).is_err());
    let bad_default = r#"{"contract": "1.0", "settings": [{"key": "a", "label": "A", "type": "toggle", "default": 3}]}"#;
    assert!(Manifest::from_json(bad_default).unwrap_err().contains("default"));
}

#[test]
fn the_frame_opens_on_the_first_run_until_turned_off() {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut settings = Settings::new();
    assert!(settings.show_at_start(&args(&[])));
    settings.set(gameframe_frame::SHOW_AT_START, json!(false));
    assert!(!settings.show_at_start(&args(&[])));
    assert!(settings.show_at_start(&args(&["--frame"])) && settings.show_at_start(&args(&["--launcher"])));
    assert!(!Settings::new().show_at_start(&args(&["--no-frame"])));
}

#[test]
fn a_config_game_fills_its_arguments() {
    let text = r#"{
        "manifest": {"contract": "1.0", "name": "x", "capabilities": ["start"],
                     "settings": [{"key": "video.scale", "label": "Scale", "type": "text", "default": "2"}]},
        "start": {"command": "true", "args": ["--renderer={renderer}", "--scale", "{setting:video.scale}"]}
    }"#;
    let game = Configured::from_json(text).unwrap();
    let start = Event::new(topics::GAME_START, Origin::Frame, json!({"renderer": "glide", "settings": {"video.scale": "3"}, "args": ["--vanilla"]}));
    assert_eq!(game.arguments(&start), ["--renderer=glide", "--scale", "3", "--vanilla"]);
    let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
    s.start(vec![]);
    assert_eq!(s.state(), "running");
    for _ in 0..200 {
        s.tick();
        if s.state() == "stopped" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(s.state(), "stopped", "`true` ends at once and the frame sees it");
}

#[test]
fn a_game_in_another_language_speaks_json_lines() {
    // A whole game in a shell script: its manifest, then an answer to each event it reads.
    let script = r#"echo '{"contract":"1.0","name":"sh","capabilities":["start","input"]}'
while read -r line; do
  case "$line" in
    *'"game.start"'*) echo '{"topic":"game.status","data":{"state":"running"},"from":"game"}' ;;
    *'"input.press"'*) echo "pressed" >&2 ;;
  esac
done"#;
    let game = gameframe_frame::Process::spawn("sh", &["-c".into(), script.into()]).unwrap();
    let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
    assert_eq!(s.manifest().name, "sh");
    s.start(vec![]);
    s.press("fire", true);
    for _ in 0..200 {
        s.tick();
        if s.state() == "running" && s.log().iter().any(|e| e.str("text") == Some("pressed")) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(s.state(), "running");
    assert!(s.log().iter().any(|e| e.str("text") == Some("pressed")), "standard error goes to the console");
}

#[test]
fn the_game_fills_a_choice_from_its_data() {
    let s = demo();
    let track = s.manifest().setting("race.track").unwrap();
    let gameframe_frame::Kind::Choice { options, .. } = &track.kind else { panic!("a choice") };
    assert_eq!(options.iter().map(|o| o.value.as_str()).collect::<Vec<_>>(), ["CANYON", "HARBOUR", "SNOWLINE"]);
    assert_eq!(s.settings().get("race.track"), Some(&json!("CANYON")), "an empty saved value moves to the first track");

    let mut saved = Settings::new();
    saved.set("race.track", json!("SNOWLINE"));
    let mut s = Session::new(Box::new(Demo::default()), saved).unwrap();
    assert_eq!(s.settings().get("race.track"), Some(&json!("SNOWLINE")), "kept while the list is not filled");
    s.ready();
    assert_eq!(s.settings().get("race.track"), Some(&json!("SNOWLINE")), "kept, since it is in the list");
    assert!(s.set("race.track", json!("MOON")).is_err());
    assert!(s.set("race.track", json!("HARBOUR")).is_ok());
}

#[test]
fn the_frame_hides_while_the_game_runs_and_comes_back() {
    let mut s = demo();
    s.hide_while_running(true);
    assert!(s.shown());
    s.start(vec![]);
    for _ in 0..30 {
        s.tick();
    }
    assert_eq!(s.state(), "running");
    assert!(!s.shown(), "hidden once the game runs");
    s.open_frame();
    assert!(s.shown(), "the game or the player asked for the frame");
    s.back_to_game();
    assert!(!s.shown());
    s.stop();
    assert!(s.shown(), "a stopped game brings the frame back");
}

#[test]
fn a_running_game_asks_for_the_frame() {
    // The game's own menu has a "Launcher" item: it sends `frame.open`.
    let script = r#"echo '{"contract":"1.0","name":"sh","capabilities":["start","input"]}'
while read -r line; do
  case "$line" in
    *'"game.start"'*) echo '{"topic":"game.status","data":{"state":"running"},"from":"game"}' ;;
    *'"input.press"'*) echo '{"topic":"frame.open","data":null,"from":"game"}' ;;
  esac
done"#;
    let game = gameframe_frame::Process::spawn("sh", &["-c".into(), script.into()]).unwrap();
    let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
    s.hide_while_running(true);
    s.start(vec![]);
    for _ in 0..200 {
        s.tick();
        if s.state() == "running" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!s.shown(), "the game has the screen");
    s.press("menu", true);
    for _ in 0..200 {
        s.tick();
        if s.shown() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(s.shown(), "frame.open from the game shows the frame");
}

#[test]
fn a_config_game_lists_files_and_runs_in_its_folder() {
    let dir = std::env::temp_dir().join(format!("frame-load-{}", std::process::id()));
    let tracks = dir.join("game/Data/TRACKS");
    std::fs::create_dir_all(&tracks).unwrap();
    for f in ["canyon.trk", "HARBOUR.TRK", "notes.txt"] {
        std::fs::write(tracks.join(f), b"").unwrap();
    }
    std::fs::write(
        dir.join("frame.json"),
        r#"{
        "manifest": {"contract": "1.0", "name": "x", "capabilities": ["start", "settings"],
                     "settings": [{"key": "race.track", "label": "Track", "type": "choice",
                                   "from": {"files": "DATA/tracks/*.TRK"}, "default": ""}]},
        "start": {"command": "sh", "args": ["-c", "echo {setting:race.track} > chosen.txt"]},
        "folder": "game"
    }"#,
    )
    .unwrap();
    let game = Configured::load(&dir.join("frame.json")).unwrap();
    let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
    let gameframe_frame::Kind::Choice { options, .. } = &s.manifest().setting("race.track").unwrap().kind else { panic!() };
    assert_eq!(options.iter().map(|o| o.value.as_str()).collect::<Vec<_>>(), ["HARBOUR", "canyon"]);
    s.set("race.track", json!("canyon")).unwrap();
    s.start(vec![]);
    for _ in 0..200 {
        s.tick();
        if s.state() == "stopped" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(std::fs::read_to_string(dir.join("game/chosen.txt")).unwrap().trim(), "canyon");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A JSON-lines game that prints its manifest, runs `then`, and exits; the session's `game.status`
/// events once it has gone quiet.
fn ended_program(then: &str) -> Vec<Event> {
    let script = format!(r#"echo '{{"contract":"1.0","name":"sh","capabilities":["start"]}}'
{then}"#);
    let game = gameframe_frame::Process::spawn("sh", &["-c".into(), script]).unwrap();
    let mut s = Session::new(Box::new(game), Settings::new()).unwrap();
    for _ in 0..100 {
        s.tick();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    s.log().iter().filter(|e| e.topic == topics::GAME_STATUS).cloned().collect()
}

#[test]
fn a_program_that_ends_is_reported_once() {
    let ends = ended_program("exit 0");
    assert_eq!(ends.len(), 1, "one end, not one each tick: {ends:?}");
    assert_eq!(ends[0].str("state"), Some("stopped"));
}

#[test]
fn a_program_that_fails_says_why() {
    let ends = ended_program("echo 'no disc in the drive' >&2; exit 3");
    assert_eq!(ends.len(), 1, "{ends:?}");
    assert_eq!(ends[0].str("state"), Some("failed"));
    assert!(ends[0].str("detail").unwrap().ends_with(": no disc in the drive"), "{ends:?}");
}

#[test]
fn a_program_that_said_it_stopped_is_not_reported_again() {
    let said = r#"echo '{"topic":"game.status","data":{"state":"stopped","detail":"the race ended"},"from":"game"}'; exit 0"#;
    let ends = ended_program(said);
    assert_eq!(ends.len(), 1, "{ends:?}");
    assert_eq!(ends[0].str("detail"), Some("the race ended"));
    // An error after its own `stopped` is still a failure.
    let crashed = r#"echo '{"topic":"game.status","data":{"state":"stopped"},"from":"game"}'; exit 1"#;
    let ends = ended_program(crashed);
    assert_eq!(ends.iter().filter_map(|e| e.str("state")).collect::<Vec<_>>(), ["stopped", "failed"]);
}

#[test]
fn a_config_game_that_fails_says_failed() {
    let text = r#"{"manifest": {"contract": "1.0", "name": "x", "capabilities": ["start"]},
                   "start": {"command": "false"}}"#;
    let mut s = Session::new(Box::new(Configured::from_json(text).unwrap()), Settings::new()).unwrap();
    s.start(vec![]);
    for _ in 0..200 {
        s.tick();
        if s.state() == "failed" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(s.state(), "failed");
}

/// A game whose program ends at once, as if the player quit it from its own menu.
fn ending_game(on_exit: &str) -> Session {
    let text = r#"{"manifest": {"contract": "1.0", "name": "x", "capabilities": ["start"]},
                   "start": {"command": "true"}}"#;
    let mut saved = Settings::new();
    saved.set(gameframe_frame::ON_EXIT, json!(on_exit));
    let mut s = Session::new(Box::new(Configured::from_json(text).unwrap()), saved).unwrap();
    s.hide_while_running(true);
    s.start(vec![]);
    for _ in 0..200 {
        s.tick();
        if s.state() == "stopped" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(s.state(), "stopped");
    s
}

#[test]
fn a_game_that_ends_returns_to_the_launcher_or_the_desktop() {
    let s = ending_game("launcher");
    assert!(s.shown() && !s.wants_exit(), "the default brings the launcher back");
    let s = ending_game("desktop");
    assert!(s.wants_exit(), "frame.on-exit desktop leaves the program");
}

#[test]
fn the_player_stops_or_exits() {
    let mut s = demo();
    s.send(topics::FRAME_PREFERENCE, json!({"key": "frame.on-exit", "value": "desktop"}));
    s.start(vec![]);
    for _ in 0..30 {
        s.tick();
    }
    s.stop();
    for _ in 0..30 {
        s.tick();
    }
    assert_eq!(s.state(), "stopped");
    assert!(!s.wants_exit(), "a stop the player asked for keeps the launcher");
    s.start(vec![]);
    for _ in 0..30 {
        s.tick();
    }
    s.exit();
    assert!(s.wants_exit());
    assert!(s.log().iter().rev().any(|e| e.topic == topics::GAME_STOP), "the game is stopped first, so it can save");
}
