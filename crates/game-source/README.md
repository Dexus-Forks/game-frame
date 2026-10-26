# gameframe-game-source

Where a game's files come from, behind one `Source` trait: a folder, a CD image read in place
([disc-image](../disc-image/README.md)), or a URL the player gives. The crate holds no URL.

- **A folder**: `Folder::new(path)`. Paths are relative, with `/` between names.
- **A CD image**: `Image` over a `disc_image::Volume`. `open(path, Some("DATA"))` opens a
  folder or an image, and makes a subfolder of the disc the root, so the paths match a copy of
  that folder.
- **Layers**: `Layered` reads a top folder (patches, added levels) before the base, with
  names matched case-blind as on Windows.
- **A URL, natively**: `download::fetch` downloads the image once into a cache folder, resumes
  when the connection breaks, and checks the MD5s the player gives.
- **A URL or dropped files, in the browser**: `web::open_url` and `web::open_files` read the
  image in 1 MiB chunks, by HTTP range requests or file slices (`remote`). `web::open_store`
  keeps the chunks in the browser's private file system (OPFS), or in IndexedDB where OPFS is
  missing, so a second visit fetches nothing.

```rust
let source = game_source::open(std::path::Path::new("target/cd/GAME.CUE"), Some("DATA"))?;
for f in source.list() {
    println!("{} {}", f.path, f.size);
}
let bytes = source.read("LEVELS/LEVEL1.DAT")?;
```

Add it with `game-source = { package = "gameframe-game-source", git = "https://github.com/tanepiper/game-frame" }`.
[gameframe-library](../library/README.md) uses it for a game's data section.

## What it does not do

- It does not unpack installers or archives: a game's own format crate does that over the
  files it gives.
- It does not decide where a URL comes from. The player gives it; a project never ships one
  that points at game files.
- The browser reads are asynchronous; the native `Source` reads are blocking.

## Where it comes from

POD: Planet of Death (1997), whose port reads the player's disc from a folder, a CUE/BIN image,
or an image on the player's own server. The tests run a local HTTP server to check resume, range
reads and the MD5 check, and use no game's files.

| | |
|---|---|
| Build | `cargo build --release -p gameframe-game-source`; for the browser, add `--target wasm32-unknown-unknown` |
| Test | `cargo test --release -p gameframe-game-source` |

Licence: Apache 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE)).
