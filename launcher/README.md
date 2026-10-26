# gameframe-launcher

The frame in front of a game that a `frame.json` file describes: a port's own program, or any
other command. The player picks a renderer and the settings,
starts the game, and comes back to the launcher when it ends. It is the general launcher on
[gameframe-frame](../crates/frame/README.md) and
[gameframe-frame-ui](../crates/frame-ui/README.md), for games that do not speak the contract
themselves. Desktop only.

## Build and run

It is its own Cargo workspace, so the desktop's dependencies stay out of `crates/`. It needs
Rust 1.85 or newer, and on Linux the WebKitGTK packages that Dioxus lists for the desktop.

```bash
cd launcher
cargo run -- example
```

```text
gameframe-launcher [PATH] [--frame | --no-frame] [--data DIR] [--preset NAME]
```

- `PATH` is a `frame.json` file or the folder that holds one. The default is the current folder.
- `--data DIR` keeps the settings and the library there. The default is `launcher-data/` next
  to `frame.json`.
- `--frame` opens the window even when the player turned it off at start. `--no-frame` starts
  the game at once with the window hidden. Without either, the saved choice decides.
- `--preset NAME` starts with one of the ready-made themes, such as `win98`, until the player
  picks one in Settings.

## `frame.json`

```json
{
  "manifest": {
    "contract": "1.0", "name": "Example", "capabilities": ["start", "settings", "renderers"],
    "renderers": [{ "id": "software", "label": "Software", "grade": "exact", "default": true }],
    "settings": [{ "key": "race.track", "label": "Track", "type": "choice",
                   "from": { "files": "TRACKS/*.TRK" }, "default": "" }]
  },
  "start": { "command": "./game", "args": ["--track", "{setting:race.track}"] },
  "folder": "game"
}
```

- `manifest` is the frame's manifest, as in the frame's README.
- `start` is the command. `{renderer}` and `{setting:KEY}` in its arguments are filled in.
- `folder` is the game's folder, relative to `frame.json`. The command runs there. `{folder}`
  in the arguments becomes its full path.
- `image` names a disc image instead, such as `"image": "media/GAME.CUE"` in a GameFrame
  project. File lists are read from inside the image, and `{image}` in the arguments becomes
  its full path, for an emulator that mounts it (`imgmount d {image}` in DOSBox).
- A choice with `"from": {"files": PATTERN}` lists the files that match in the game's folder,
  or in the image, by name without the extension: the tracks, the cars or the levels. The folders in the
  pattern match in any case, and its last part takes `*` and `?`.

`example/` has a `frame.json` whose game is a shell command that prints its arguments and
waits three seconds, and two empty track files.

A `library.json` next to `frame.json` gives the title, the look, the help pages and the
player's library, as for the [library page](../templates/library-page/README.md). Without one
the window takes the game's name and offers every ready-made theme.

## While the game runs

The window hides while the game runs. When the game ends by itself, the window comes back. With
"When the game ends: Exit to desktop" in Play, the launcher leaves instead. A stop from the
window always comes back to it, and a game that failed always shows the window with the reason.
"Exit to desktop" in Play stops a running game, then leaves.

## What it does not do

It does not install, find or check the game: the command must work from the game's folder. A
game started from `frame.json` has no channel to the frame, so it cannot open the window while
it runs. A port that speaks the contract can, with `frame.open`: over JSON lines through
`gameframe_frame::Process`, or with the frame in its own program.
