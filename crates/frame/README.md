# gameframe-frame

The contract between a port's core and the frame around it. The core is the exact
reimplementation of the game. The frame is everything the player sees before and around it: the
launcher, the settings, the renderer choice, the library of game files and saves, help pages
and an event console. This crate has no UI. It defines the events, the manifest and the session
that connects the two. [gameframe-frame-ui](../frame-ui/README.md) draws the frame, and
[examples/frame-demo](../../examples/frame-demo/README.md) runs it with no game.

```rust
use gameframe_frame::{Demo, Session, Settings};

let mut session = Session::new(Box::new(Demo::default()), Settings::new())?;
session.on("game.*", |e| println!("{} {}", e.topic, e.data));
session.ready();
session.select_renderer("glide")?;
session.set("video.fog", serde_json::json!(false))?;
session.start(Vec::new());
loop { session.tick(); /* every 50 ms */ }
```

## Events

Every message is an `Event`: a `topic`, JSON `data`, who sent it (`from`: `frame`, `game` or
`player`) and a sequential `id`. On the wire it is one JSON object:

```json
{"topic": "config.set", "data": {"key": "video.fog", "value": false}, "from": "frame", "id": 7}
```

A topic is two or more lower-case words joined by dots, `area.action`. A listener's pattern is
an exact topic, a prefix such as `game.*`, or `*` for every event. The frame shows every event in
its console, including topics it does not know, so a game can add its own (`demo.press`,
`pod.track.loaded`) without a change here.

| Topic | Direction | Data |
|---|---|---|
| `frame.ready` | frame to game | `{"renderer", "devices", "settings"}`, once, after the manifest is read |
| `frame.open` | either way | show the frame over a running game: the game's own menu asks for the launcher, or the frame opened |
| `frame.close` | either way | hide the frame and go back to the game ("Back to the game") |
| `frame.exit` | either way | leave the program for the desktop; the frame stops a running game first |
| `frame.preference` | frame | `{"key": "frame.show-at-start", "value": false}` |
| `config.set` | frame to game | `{"key", "value"}`: the player changed a setting |
| `config.changed` | game to frame | `{"key", "value"}`: the game changed one; the frame saves it |
| `config.options` | game to frame | `{"key", "options": [{"value", "label"}]}`: the list of a choice whose manifest has `from` |
| `render.select` | frame to game | `{"renderer": "glide"}` |
| `device.select` | frame to game | `{"slot": "sound", "device": "none"}`: a device in a slot other than graphics |
| `game.start` | frame to game | `{"renderer", "devices", "settings", "args"}` |
| `game.stop` | frame to game | none: stop, as the game's own quit does |
| `game.status` | game to frame | `{"state": "loading" or "running" or "paused" or "stopped" or "failed", "detail"}`. `Session::state()` is `idle` before the first start and `starting` from `game.start` until the game answers |
| `input.press` | player to game | `{"control", "down"}`, with the control names from the manifest |
| `library.changed` | frame to game | `{"section"}`: read that library section again |
| `report.new` | game to frame | one `gameframe_report::Report`: a place the game reached that is not ported, or a crash; `Session::reports()` keeps one per place, and Play lists them with their GitHub link (`Link::send_reports`) |
| `log.message` | either way | `{"level": "info" or "warn" or "error", "text"}` |

The constants are in `gameframe_frame::topics`.

## The manifest

The game describes itself once, when the frame starts. The frame draws only what the manifest
names, so a game opts into each feature by listing it.

```json
{
  "contract": "1.1",
  "name": "Example Port",
  "version": "0.3.0",
  "capabilities": ["start", "renderers", "devices", "settings", "input", "library"],
  "renderers": [
    {"id": "software", "label": "Software", "grade": "exact", "default": true},
    {"id": "glide", "label": "3dfx Glide", "grade": "partial", "done": 131, "total": 156,
     "notes": ["Fog is drawn without the card's table."]}
  ],
  "devices": [
    {"slot": "sound", "label": "Sound card", "options": [
      {"id": "directsound", "label": "DirectSound", "grade": "exact", "default": true},
      {"id": "none", "label": "No sound", "grade": "exact"}]}
  ],
  "settings": [
    {"key": "video.scale", "label": "Scaling", "group": "Video", "type": "choice",
     "options": [{"value": "nearest", "label": "Sharp pixels"}], "default": "nearest"},
    {"key": "sound.volume", "label": "Volume", "type": "range", "min": 0, "max": 100, "step": 5, "default": 80}
  ],
  "controls": [{"name": "fire", "label": "Fire", "key": "Space"}]
}
```

- **Capabilities.** `start`, `renderers`, `devices`, `settings`, `input` and `library`
  (`gameframe_frame::capabilities`). A frame hides what a game does not list.
- **Grades.** Each renderer has a grade, `exact`, `good`, `partial`, `experimental` or
  `missing`, and may give `done` of `total` (calls, features, tests: the game says which) and
  notes. The player sees how complete each renderer is before choosing it.
- **Settings.** `toggle`, `choice` (with `options`), `range` (`min`, `max`, `step`) and `text`.
  A choice whose options come from the game's data, such as its tracks or cars, has `from`
  instead of a fixed list: `{"files": "DATA/TRACKS/*.TRK"}` lists the matching files in the
  game's folder (a config-only game), and `{"game": true}` waits for the game to send
  `config.options`. A saved value that is no longer in the list moves to the first option.
  `group` puts settings under a heading. `restart: true` says the change waits for the next
  start. The frame refuses a value of the wrong kind.
- **Devices.** The renderers are the graphics slot. `devices` lists the other slots, as a
  1990s setup program did: `{"slot": "sound", "label": "Sound card", "options": [...]}`, each
  option a renderer's fields (id, label, detail, grade, done, total, notes, default). The slots
  the frame knows are `video`, `sound`, `music` and `input` (`device::slots`). The same card has
  the same id in every game (`device::ids`: `glide`, `powervr`, `directsound`, `sound-blaster`,
  `adlib` and the rest), and `device::known(id)` names the GameFrame crate or wrapper that models
  it. The frame saves each choice as `frame.device.SLOT` and sends both in `game.start` as
  `"devices": {"video": "glide", "sound": "none"}`. A device not ported yet may name a
  `stand_in`, another device in its slot (`"stand_in": "software"`). The game then starts on
  that one and reports the gap, so every listed device starts end to end. A kernel keeps a
  `Held` and passes it every event (`held.note(&e)`), so it knows the frame's latest picks. On
  `game.start` it calls `manifest.resolve(&data, &held)`. That gives a `Setup`: per slot the
  device asked for and the device to run (`setup.used("video")`), and the gaps. `Err` is the
  reason for `failed`: an id the manifest lacks, or a `missing` device with no stand-in. Then
  `link.send_reports(setup.asked_json())` and `setup.report_gaps(&manifest)`, which writes one
  `unported` report per gap at the kernel's own place. `chosen`, `Manifest::playable` and
  `Manifest::stand_in` are the parts it is made of. Play shows one drop-down per
  slot, and beside it a card for the chosen device: its grade, its meter, its notes.
- **Versions.** `CONTRACT` is `1.1` (1.1 added `devices`). The frame refuses a larger major number. A minor version
  only adds fields, and a frame ignores fields it does not know.

## Three ways to join

**Rust.** Implement `Game`: `manifest()`, `on_event(&Event, &mut Outbox)` and, if the game sends
events no one asked for, `poll(&mut Outbox)`. `Outbox` has `send`, `status` and `log`.

**Any language.** `Process::spawn(program, args)` starts a program that speaks JSON lines. Its
first line on standard output is its manifest. Then the frame writes each event to its standard
input and reads its events from standard output, one object per line. Standard error goes to the
console as log lines. A standard library is enough to write such a program.

A Rust port's own program takes the other end with `Link::wait(&mut game, start)`. It prints
the game's manifest, hands each event to the game's `Game` until `start` takes a launch from it
(usually after `game.start`), and returns that launch with a `Link`. While the game runs, a
thread reads the frame's events: `game.stop`, `frame.exit` or the frame closing its end sets
`Link::stopping()`, which the game's loop checks as it checks its own quit. The game reports
with `Link::status` and `Link::send`, and prints nothing else on standard output.

A port that opens its own window with winit can make the window only once in a process, and on
macOS the frame and the game each need the main thread. Such a port runs one game per program:
`Respawn::spawn(program, args)` keeps one program answering the frame, gives it the first
`game.start`, and starts a new program for each start after it, and for a start whose
arguments changed (a disc added in the Library tab). `args` is a function, called again for
each start. POD's and Hardwar's launchers run their games this way.

**Config only.** `Configured::from_json` reads a `frame.json` with a manifest and the command that
starts the game:

```json
{
  "manifest": { "contract": "1.0", "name": "Example", "capabilities": ["start", "renderers", "settings"], "...": "..." },
  "start": { "command": "./example", "args": ["--renderer", "{renderer}", "--scale", "{setting:video.scale}"] }
}
```

On `game.start` it runs the command with `{renderer}`, `{device:SLOT}` and `{setting:KEY}` filled in. It reports
`running`, then `stopped` when the program ends. `Configured::load(path)` reads the file from
disk: `"folder"` names the game's folder relative to it, the command runs there, and each choice
with `"from": {"files": PATTERN}` lists the matching files there. The pattern's folders match
in any case, and its last part takes `*` and `?` (`list_files`). Such a game has no channel to
the frame while it runs, so the frame comes back when the program ends.
[`launcher/`](../../launcher/README.md) is a program that does all this from a `frame.json`.

## Back to the launcher

`Session::shown()` says whether the frame should be on screen. It starts shown. A frame whose
game opens its own window calls `hide_while_running(true)`, and then a game that reports
`running` hides it. `frame.close` hides it ("Back to the game", `back_to_game()`), and
`frame.open` shows it: the game sends it from its own menu, or the frame from `open_frame()`.
A game that stops or fails shows it again.

When the game ends by itself, the saved `frame.on-exit` decides: `"launcher"`, the default,
shows the frame, and `"desktop"` asks the program to leave (`wants_exit()`). A stop the player
asked for in the frame, and a game that failed, always show the frame. The player's "Exit to
desktop" is `exit()`: it stops a running game, so the game can save, then sends `frame.exit`.

## Settings and the first run

`Settings` holds the saved values: the game's, keyed as its manifest says, and the frame's own,
whose keys start with `frame.` (`frame.renderer`, `frame.show-at-start`, `frame.preset`,
`frame.on-exit`). It saves to one JSON
object (`settings_file::load` and `save` on the desktop). `show_at_start(args)` decides whether
the frame opens: `--frame` or `--launcher` always opens it, `--no-frame` never does, and otherwise
it opens until the player turns "Open this window when the game starts" off.

## The demo game

`Demo` is the frame's stand-in when there is no port. It lists every capability, three
renderers with different grades, five settings and six controls. Its track list comes from
the game: it sends `config.options` when the frame is ready. It answers each event with a
console line, takes 30 ticks to "load", and presses its own controls every
`DEMO_PRESS_TICKS` ticks as `demo.press` events, so the console shows traffic with no one at the
keys.

## What it does not do

It draws nothing, runs no game loop and reads no game files. The game owns its loop and its
window, and the frame only sends and receives events. The library of files is
[gameframe-library](../library/README.md).
