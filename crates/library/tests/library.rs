//! The library's rules on the memory and folder stores, and the template's library.json.

use gameframe_library::game_source::Source;
use gameframe_library::{data_url, now, Card, Folder, Hooks, Kind, Library, Memory, NoHooks, Section, Spec, Store};

const TEMPLATE: &str = include_str!("../../../templates/library-page/library.json");

fn spec() -> Spec {
    Spec::from_json(TEMPLATE).unwrap()
}

#[test]
fn the_template_reads() {
    let s = spec();
    assert_eq!(s.id, "example-port");
    assert!(s.encrypt());
    let data = s.of_kind(Kind::GameData).unwrap();
    assert!(!data.download() && data.many() && !data.ordered());
    let mods = s.section("mods").unwrap();
    assert!(mods.ordered() && mods.toggles() && mods.download());
    assert!(s.section("saves").unwrap().accepts("TOWN.SAV"));
    assert_eq!(s.pages[0].path, "help");
    assert_eq!(s.panels[0].kind, "messages");
}

#[test]
fn bad_specs_are_refused() {
    assert!(Spec::from_json(r#"{"id":"x","sections":[]}"#).unwrap_err().contains("game-data"));
    assert!(Spec::from_json(r#"{"id":"X Y","sections":[{"id":"g","kind":"game-data"}]}"#).is_err());
    assert!(Spec::from_json(r#"{"id":"x","colour":1,"sections":[{"id":"g","kind":"game-data"}]}"#).is_err());
    let twice = r#"{"id":"x","sections":[{"id":"g","kind":"game-data"},{"id":"g","kind":"saves"}]}"#;
    assert!(Spec::from_json(twice).unwrap_err().contains("twice"));
}

#[test]
fn game_data_is_checked_and_loaded() {
    let lib = Library::new(spec(), Memory::new(), NoHooks);
    let missing = now(lib.missing_game_data()).unwrap();
    assert_eq!(missing.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["GAME.DAT", "LEVELS.PAK"]);
    now(lib.add("game", "game.dat", b"abc")).unwrap();
    now(lib.add("game", "LEVELS.PAK", b"defg")).unwrap();
    assert!(now(lib.missing_game_data()).unwrap().is_empty());
    let source = now(lib.game_data()).unwrap();
    assert_eq!(source.read("GAME.DAT").unwrap(), b"abc");
    assert_eq!(source.read_range("LEVELS.PAK", 1, 2).unwrap(), b"ef");
    assert!(source.read_range("LEVELS.PAK", 3, 2).is_err());
}

#[test]
fn files_are_refused_by_name_and_by_the_game() {
    struct Picky;
    impl Hooks for Picky {
        fn accept(&self, _: &Section, _: &str, bytes: &[u8]) -> Result<(), String> {
            if bytes.starts_with(b"SAVE") { Ok(()) } else { Err("not a save".into()) }
        }
        fn describe(&self, _: &Section, name: &str, _: &[u8]) -> Option<Card> {
            Some(Card { title: name.trim_end_matches(".sav").into(), ..Card::default() })
        }
    }
    let lib = Library::new(spec(), Memory::new(), Picky);
    assert!(now(lib.add("saves", "town.txt", b"SAVE")).is_err());
    assert_eq!(now(lib.add("saves", "town.sav", b"JUNK")).unwrap_err().to_string(), "not a save");
    assert!(now(lib.add("saves", "../town.sav", b"SAVE")).is_err());
    assert!(now(lib.add("nowhere", "a", b"")).is_err());
    now(lib.add("saves", "town.sav", b"SAVE1")).unwrap();
    now(lib.add("saves", "farm.sav", b"SAVE2")).unwrap();
    let saves = now(lib.entries("saves")).unwrap();
    // Newest first, with the game's card.
    assert_eq!(saves[0].entry.name, "farm.sav");
    assert_eq!(saves[1].entry.card.as_ref().unwrap().title, "town");
}

#[test]
fn mods_keep_their_order_and_switches() {
    let lib = Library::new(spec(), Memory::new(), NoHooks);
    for name in ["c.mod", "a.mod", "b.mod"] {
        now(lib.add("mods", name, b"m")).unwrap();
    }
    assert_eq!(now(lib.enabled("mods")).unwrap(), ["c.mod", "a.mod", "b.mod"]);
    now(lib.move_to("mods", "b.mod", 0)).unwrap();
    now(lib.set_on("mods", "a.mod", false)).unwrap();
    assert_eq!(now(lib.enabled("mods")).unwrap(), ["b.mod", "c.mod"]);
    now(lib.remove("mods", "c.mod")).unwrap();
    let state = now(lib.store().state("mods")).unwrap();
    assert_eq!(state.order, ["b.mod", "a.mod"]);
    assert_eq!(state.off, ["a.mod"]);
}

#[test]
fn a_one_file_section_replaces_its_file() {
    let text = r#"{"id":"x","sections":[{"id":"g","kind":"game-data"},{"id":"cfg","label":"Settings","many":false}]}"#;
    let lib = Library::new(Spec::from_json(text).unwrap(), Memory::new(), NoHooks);
    now(lib.add("cfg", "one.ini", b"1")).unwrap();
    now(lib.add("cfg", "two.ini", b"2")).unwrap();
    let names: Vec<_> = now(lib.entries("cfg")).unwrap().into_iter().map(|l| l.entry.name).collect();
    assert_eq!(names, ["two.ini"]);
}

#[test]
fn the_folder_store_keeps_files_where_the_player_can_see_them() {
    let root = std::env::temp_dir().join(format!("gameframe-library-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let lib = Library::new(spec(), Folder::new(&root), NoHooks);
    now(lib.add("saves", "town.sav", b"SAVE")).unwrap();
    now(lib.add("mods", "a.mod", b"m")).unwrap();
    now(lib.store().write("saves", "farm.sav", b"x", Some(Card { title: "Farm".into(), ..Card::default() }))).unwrap();
    assert_eq!(std::fs::read(root.join("saves/town.sav")).unwrap(), b"SAVE");
    let saves = now(lib.entries("saves")).unwrap();
    assert_eq!(saves.len(), 2, "hidden files are not listed");
    assert!(saves.iter().any(|l| l.entry.card.as_ref().is_some_and(|c| c.title == "Farm")));
    // A second library on the same folder sees the same state.
    let again = Library::new(spec(), Folder::new(&root), NoHooks);
    now(again.set_on("mods", "a.mod", false)).unwrap();
    assert!(now(lib.enabled("mods")).unwrap().is_empty());
    now(lib.remove("saves", "farm.sav")).unwrap();
    assert!(!root.join("saves/.library/cards/farm.sav.json").exists());
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn thumbnails_are_data_urls() {
    assert_eq!(data_url("image/png", b"Man"), "data:image/png;base64,TWFu");
    assert_eq!(data_url("a/b", b"Ma"), "data:a/b;base64,TWE=");
    assert_eq!(data_url("a/b", b"M"), "data:a/b;base64,TQ==");
}
