# The frame demo

The frame with no game. The demo game from [gameframe-frame](../../crates/frame/README.md)
stands in for a port: three renderers with different grades, five settings (a track list the game fills) and a pad of
controls. It answers each event on the console and presses its own controls every two seconds,
so the console shows traffic with no one at the keys. It uses the sections of the
[library page](../../templates/library-page/README.md) template. It starts in the ready-made
Windows 98 theme, and the Settings tab switches between all of them.

It is its own Cargo workspace, so the desktop's dependencies stay out of `crates/`.

## Desktop

```bash
cargo run
```

The frame opens on the first run and saves its settings and library in `frame-demo-data/` in the
current folder. If "Open this window when the game starts" is turned off, the program says so
and exits, as a port would start its game. `cargo run -- --frame` opens the frame again.
`cargo run -- --preset xp` starts in another theme from
[the list](../../templates/library-page/README.md#the-design-system), or `none` for the frame's
own look. A theme chosen in Settings is saved and wins over it.

## Browser

With [Trunk](https://trunkrs.dev) and the `wasm32-unknown-unknown` target:

```bash
trunk serve --no-default-features --features web
```

Or build a folder to serve from anywhere:

```bash
trunk build --no-default-features --features web --public-url ./
```

The settings are kept in local storage and the library in IndexedDB, encrypted with a key that
stays in this browser. `?preset=aqua` in the address starts in another theme.

## What it does not do

Start runs only the demo game's loading steps. No game is drawn.
