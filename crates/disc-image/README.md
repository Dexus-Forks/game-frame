# gameframe-disc-image

Read CD images in place: a CUE sheet and its BIN files, a raw BIN alone (Mode 1, Mode 2 Form 1)
or a cooked ISO. It lists the tracks, opens the ISO 9660 volume of the data track (with Joliet
names when the disc has them), lists the files with their size, sector and date, and reads any
range of a file without loading the whole image.

Every read goes through the `ReadAt` trait. The same code runs on a file, on memory, or on a
`ChunkCache` that a source fills on demand. A read of bytes that are not there yet fails with a
`Missing` error that names the chunks; the caller fetches them and reads again. The
[game-source](../game-source/README.md) crate does that for HTTP range requests and for files
dropped into a browser.

```rust
let disc = disc_image::Disc::open_path("target/cd/GAME.CUE")?;
let volume = disc.volume()?;
for e in volume.files() {
    println!("{} {} {}", e.path, e.size, e.lba);
}
let head = volume.read_range(volume.find("DATA/README.TXT").unwrap(), 0, 64)?;
```

Add it with `disc-image = { package = "gameframe-disc-image", git = "https://github.com/tanepiper/game-frame" }`,
so the code keeps the name `disc_image`.

## What it does not do

- It reads data tracks only. Audio tracks are listed with their position, not decoded.
- It does not read Mode 2 Form 2 sectors (video CDs), UDF, HFS or Rock Ridge names.
- It does not check sector EDC/ECC.
- It never writes an image.

## Where it comes from

POD: Planet of Death (1997), whose port runs from the player's own CD image, natively and in the
browser. The tests build a small image in every layout the crate reads (cooked ISO, raw Mode 1,
raw Mode 2 Form 1, CUE sheets with an audio track after the data), so they use no game's files.
To check it on your game's disc, list the files with it and compare the list and a few files'
bytes with a copy of the disc mounted by the operating system.

| | |
|---|---|
| Build | `cargo build --release -p gameframe-disc-image`; it has no dependencies and builds for `wasm32-unknown-unknown` |
| Test | `cargo test --release -p gameframe-disc-image` |

Licence: Apache 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE)).
