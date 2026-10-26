# gameframe-library

The player's library for a port. It holds the game data, the saves and the sections a game opts
into (mods, screenshots, recordings, or its own), and keeps them in folders on the desktop or in
the browser. GameFrame's [library page](../../templates/library-page/README.md) is the HTML page
that lets the player manage them. The game declares its sections once, in `library.json`, and
the page and the crate both read that file.

```rust
use gameframe_library::{now, Folder, Library, NoHooks, Spec};

let spec = Spec::from_json(include_str!("../web/library.json"))?;
let lib = Library::new(spec, Folder::new(dirs::data_dir().unwrap().join("my-port")), NoHooks);
if !now(lib.missing_game_data())?.is_empty() {
    // Ask for the game files.
}
let data = now(lib.game_data())?; // a game_source::Source
let mods = now(lib.enabled("mods"))?; // the mods that are on, in load order
now(lib.add("saves", "town.sav", &bytes))?;
```

In the browser the same calls are awaited, after the page is ready:

```rust
let store = gameframe_library::web::IndexedDb::open(&spec).await?;
let lib = Library::new(spec, store, MyHooks);
let saves = lib.entries("saves").await?;
```

## What it holds

- **Sections.** `Kind` is `game-data` (required, one per game), `saves`, `mods`, `screenshots`,
  `recordings` or `custom`. The kind sets the defaults: mods are ordered and switched on and off,
  saves are newest first, and everything except game data can be downloaded. Each field in
  `library.json` overrides a default: `accept`, `many`, `ordered`, `toggles`, `download`,
  `import`, `required`.
- **Hooks.** `Hooks::describe` gives the page a card for a file: a save's town and date, a
  thumbnail made with `data_url`. `Hooks::accept` refuses a file with a sentence the player sees.
  In the browser, `web::register_hooks` hands both to the page.
- **Stores.** `Folder` keeps `ROOT/SECTION/NAME`, so players can copy saves by hand.
  `Folder::link_in(section, from, keep)` fills a section from a folder the player already has,
  such as the game's install folder, by hard links (nothing is copied; the folder must be on
  the same volume). It links the files the section accepts and `keep` takes, and leaves a
  section alone once it holds every required file, or with none required, any file.
  `web::IndexedDb` uses the page's own layout. `Memory` is for tests. A game can write its own
  `Store`.
- **Big files in the browser.** `Library::add_file` keeps a `web_sys::File` the player chose.
  A file of at least `web::BIG` bytes (64 MiB), such as a CD image, is streamed into the origin
  private file system (OPFS) 8 MiB at a time, as `gameframe-library-<id>/SECTION/NAME`, and is
  never read whole. Its record still goes into IndexedDB, so it lists like any other file. The
  hooks see only its first 1 MiB. `Library::file` gives a kept file back as a `File`, to read in
  slices, for example with `game_source::web::open_files`.
- **Game data as a source.** `Library::game_data` gives every kept game file as a
  [`game_source::Source`](../game-source/README.md), the same trait a folder or a CD image gives.
  On the desktop, `Folder::section_dir` opens the game-data folder directly instead.

## Encryption in the browser

With `encrypt` on (the default), each file kept in the browser is encrypted with AES-GCM. The
key is a WebCrypto key made by the browser and marked non-extractable. Scripts can use it but
cannot read it, so a copy of the database's files cannot be read without that browser. It
protects files that are copied off the disk. It does not protect against code that runs in the
page. Browsers give no fixed machine identifier, by design, so the key lives in the browser
profile.

A big file (at least `web::BIG` bytes) is kept plain in OPFS. It is read back in slices, and
AES-GCM over the whole file would have to be read whole to be checked.

Open the Rust store after the page's `gameframeLibrary.ready` promise resolves. Then both sides
use the one key the page made.

## What it does not do

- It does not draw the in-game menus. The game reads the same library from its own load and
  save screens.
- It does not move files from another layout. A port that kept files before (in its own
  IndexedDB database, say) copies them in once.
- It does not read a big file in ranges itself. `Library::file` gives the `File`, and
  `game-source` reads it. `Library::game_data` and `Store::read` still read each file whole.
- The HTML page lists a big file but cannot download it, and a file added on the page goes
  whole into IndexedDB whatever its size. Add a CD image from the game's own Library tab.
- The page's panels and pages are kept in `Spec` for the page; the crate does not read them.

## Where it comes from

Un-Banished's browser start page and its save store, POD's game sources, and Hardwar's file and
save crates. The tests cover the rules on the memory and folder stores, and the template's
`library.json`. The OPFS path for big files builds for the browser but has not been run in one
yet. The browser store was checked by hand against the page in a browser: each side
read and changed the files and the switches the other side wrote.

| | |
|---|---|
| Build | `cargo build --release -p gameframe-library`; for the browser, add `--target wasm32-unknown-unknown` |
| Test | `cargo test --release -p gameframe-library` |

Licence: Apache 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE)).
