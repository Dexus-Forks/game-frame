# gameframe-frame-ui

The frame's views as [Dioxus](https://dioxuslabs.com) 0.7 components, for the browser (wasm)
and the desktop. They talk to the game only through [gameframe-frame](../frame/README.md)'s
`Session`, and they keep the player's files through [gameframe-library](../library/README.md).

```rust
use gameframe_frame::{Demo, Session, Settings};
use gameframe_frame_ui::{Frame, FrameSetup};
use gameframe_library::Spec;

fn app() -> Element {
    let setup = use_hook(|| {
        let session = Session::new(Box::new(Demo::default()), Settings::new()).unwrap();
        FrameSetup::new(session, Spec::from_json(include_str!("library.json")).unwrap())
    });
    rsx! { Frame { setup } }
}
```

[examples/frame-demo](../../examples/frame-demo/README.md) is a whole program for both targets,
with the library in IndexedDB or a folder and the settings saved.

## The views

| Tab | Shown when | What it does |
|---|---|---|
| Play | always | Missing game files, the renderers with their grade and a meter, Start, then "Back to the game" and Stop while it runs, the game's status, and "Open this window when the game starts". With `with_exit`, also "When the game ends" (come back here, or exit to desktop) and "Exit to desktop" |
| Settings | the manifest lists `settings`, or `library.json` lists `presets` | The theme, when `presets` offers a choice, then every setting by group, with the input its kind needs |
| Library | the setup has a library | The sections from `library.json`: add, delete, switch on and off, reorder. Each change sends `library.changed` |
| A page per `pages` entry | always | The page's Markdown, with `::: tip`, `info`, `warning` and `danger` callouts |
| Console | always | The game's controls as buttons (`input.press`), a form that sends any event, and the event log with a topic filter |

Each view is also exported (`Play`, `SettingsView`, `LibraryView`, `Page`, `Console`), so a game
can lay out its own frame from them. Inside a view, `use_frame()` gives the `FrameContext`:
`with` reads the session, `update` changes it, and `spec`, `library` and `setting` read the rest.

## `FrameSetup`

- `FrameSetup::new(session, spec)`: the game's session and its `library.json`.
- `.with_library(library)`: the player's library. Without it the Library tab is hidden.
- `.with_save(|settings| ...)`: called when the settings change, to save them.
- `.with_css(text)`: the game's own stylesheet, drawn last.
- `.with_window(|shown| ...)`: show or hide the program's window when the session says (see
  "Back to the launcher" in the frame's README). Without it, a hidden frame leaves one "Open the
  frame" button in the corner of the page, for a game in the same page.
- `.with_exit(|| ...)`: leave the program after `frame.exit`. It adds the exit controls to Play.
- `.start_at_once(true)`: start the game as soon as the frame is ready, for a program whose
  frame is turned off at start and whose window opens hidden.
- `tick_ms`: how often the game is polled, 50 ms by default.

On the desktop, `load_pages(&mut spec, dir)` reads each page's `src` file into its text.

## The look

The frame uses the [library page](../../templates/library-page/README.md)'s design tokens and
components, built in, so one `library.json` styles both. It draws these in order:

1. `tokens.css` and `library.css` from the library page, then `frame.css`;
2. the ready-made theme named by `"preset"` (the
   [list](../../templates/library-page/README.md#the-design-system); `THEMES` has them all and
   `theme(name)` returns one). When `library.json` lists `"presets"`, the player's choice from
   the Settings tab replaces it (`active_theme`);
3. the game's stylesheet from `with_css`;
4. the `"theme"` tokens from `library.json`, on the root element, which win over all of them.

Every class starts with `lib-`. The frame's own are `lib-frame`, `lib-renderers`, `lib-renderer`,
`lib-grade`, `lib-meter`, `lib-settings`, `lib-pad`, `lib-send`, `lib-log`, `lib-exit`,
`lib-frame-closed` and `lib-frame-open`.

## What it does not do

It does not open a window for the game or draw the game: the game owns its window. The panels of
the HTML library page (news, messages, video, releases) are not drawn yet. Only pages are.
