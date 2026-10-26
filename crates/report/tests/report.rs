//! The reporter: one report per place with its context and the game's state, counts, files,
//! the sink, strict mode and panics; and a report's issue title, body and link.

use gameframe_report::{self as report, Kind, Meta, Report};
use serde_json::json;
use std::sync::{Arc, Mutex};

fn reach(material: u32) {
    report::unported!("0x5898b4", "the material of a hit's other side", material = material, side = "left");
}

// One test, since the reporter is one per program.
#[test]
fn the_reporter_writes_once_per_place_and_counts() {
    let dir = std::env::temp_dir().join(format!("gameframe-report-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    report::install(Meta { game: "Test".into(), version: "1.0".into(), issues: "owner/game".into(), ..Meta::default() });
    report::set_mode(report::Mode::Dev);
    report::set_dir(&dir);
    report::set_state("frame", 1234);
    report::set_launch(json!({ "start": { "kind": "new", "setup": 3 } }));
    report::set_devices(json!({ "video": "software", "sound": "none" }));
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keep = heard.clone();
    report::on_report(move |r| keep.lock().unwrap().push(r.id.clone()));

    for m in 0..5 {
        reach(m);
    }
    report::mismatch!("0x489c1c", String::from("stream A"), ours = 1, original = 2);
    let all = report::reports();
    assert_eq!(all.len(), 2);
    let r = &all[0];
    assert_eq!((r.kind, r.count, r.original.as_deref()), (Kind::Unported, 5, Some("0x5898b4")));
    assert!(r.file.ends_with("report.rs") && r.line > 0);
    assert_eq!(r.context["material"], json!(4), "the last hit's values");
    assert_eq!(r.context["side"], json!("left"));
    assert_eq!(r.state["frame"], json!(1234));
    assert_eq!(r.devices["sound"], json!("none"));
    assert_eq!(heard.lock().unwrap().len(), 2, "the sink hears each place once");

    report::flush();
    let saved = Report::from_json(&std::fs::read_to_string(dir.join(format!("{}.json", r.id))).unwrap()).unwrap();
    assert_eq!(saved.count, 5);
    let md = std::fs::read_to_string(dir.join(format!("{}.md", r.id))).unwrap();
    assert!(md.starts_with("# [unported] the material of a hit's other side at "));
    assert!(md.contains("<!-- gameframe-report: unported-") && md.contains("`0x5898b4`") && md.contains("\"setup\": 3"));

    // A panic becomes a report, then the old hook runs.
    let caught = std::panic::catch_unwind(|| panic!("the sky has no mode 9"));
    assert!(caught.is_err());
    assert!(report::reports().iter().any(|r| r.kind == Kind::Panic && r.what == "the sky has no mode 9"));

    assert!(report::reports().iter().all(|r| r.issue.is_none()), "Dev makes no links");
    report::set_mode(report::Mode::Production);
    let home = std::env::var("HOME").unwrap();
    report::set_launch(json!({ "game": format!("{home}/Games/Test") }));
    report::unported!("0x1000", "a production place");
    let p = report::reports().into_iter().find(|r| r.what == "a production place").unwrap();
    assert!(p.issue.as_deref().unwrap().starts_with("https://github.com/owner/game/issues/new?"));
    assert_eq!(p.launch["game"], json!("~/Games/Test"), "the home folder stays on the machine");

    report::set_strict(true);
    assert!(std::panic::catch_unwind(|| reach(9)).is_err(), "strict mode stops at the place");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_report_makes_an_issue() {
    let mut r = Report {
        kind: Kind::Unported,
        what: "the sky's fog | table".into(),
        original: Some("FUN_004a5e00".into()),
        file: "crates/render/src/sky.rs".into(),
        line: 88,
        count: 3,
        meta: Meta { game: "POD".into(), issues: "https://github.com/tanepiper/unpod/".into(), ..Meta::default() },
        backtrace: "x".repeat(20_000),
        ..Report::default()
    };
    r.id = Report::id_for(r.kind, &r.file, r.line, r.original.as_deref());
    assert_eq!(r.id, Report::id_for(Kind::Unported, "crates/render/src/sky.rs", 88, Some("FUN_004a5e00")));
    assert_ne!(r.id, Report::id_for(Kind::Unported, "crates/render/src/sky.rs", 89, Some("FUN_004a5e00")));
    assert_eq!(r.title(), "[unported] the sky's fog | table at crates/render/src/sky.rs:88 (FUN_004a5e00)");
    assert!(r.markdown().contains("<summary>Stack</summary>"), "the file keeps the whole stack");
    let url = r.issue_url().unwrap();
    assert!(url.starts_with("https://github.com/tanepiper/unpod/issues/new?title=%5Bunported%5D"));
    assert!(url.contains("labels=gameframe-report%2Cunported") && url.len() < 20_000, "{}", url.len());
    assert!(!url.contains("Stack"), "a long stack stays out of the link");
    r.meta.issues = "not a repo".into();
    assert!(r.issue_url().is_none());
}
