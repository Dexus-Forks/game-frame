# The library page

A plain HTML page for a browser port. It shows the player's game files, saves and other
sections, keeps them in the browser, and starts the game. It also holds content around the game:
news, a changelog, messages, a video, help pages. It has no framework and no build step. Copy
the folder into the port's web folder, then change `library.json`.

| File | What it is | Edit it? |
|---|---|---|
| `library.json` | The words, pictures, tokens, icons, sections, panels and pages | Yes |
| `index.html` | The page: a canvas for the game, `#library`, the scripts | Add the game's own script |
| `tokens.css` | The design tokens every component uses | No: set tokens in `library.json` or `theme.css` |
| `library.css` | The components | No: restyle with tokens, or your own CSS |
| `library.js` | The library view, the icons and `window.gameframeLibrary` | No |
| `library-store.js` | IndexedDB and encryption, in the layout of the Rust crate | No |
| `library-panels.js` | The panels and the Markdown | No: add panel types from your own script |
| `library-pages.js` | The nav bar and the pages at `#/path` | No |
| `themes/` | Ready-made themes, each one stylesheet | No: pick one with `"preset"` |
| `news.md`, `help.md` | Example content | Replace |

The Rust side is [gameframe-library](../../crates/library/README.md). It reads the same
`library.json` and the same database, so the game sees what the player added here.

## Sections

`sections` lists what the library holds. Exactly one has `"kind": "game-data"`. It is shown first,
with its `steps` and a list of the `required` files that are missing. The Start button waits for
them. Every other section gets a tab. Fields: `id`, `kind` (`game-data`, `saves`, `mods`,
`screenshots`, `recordings`, `custom`), `label`, `icon`, `help`, `steps`, `accept`, `required`
(`name`, `size`, `hint`), `many`, `ordered`, `toggles`, `download`, `import`, `empty`. Text in
`backticks` is shown as code.

Files dropped anywhere on the page go to the first section whose `accept` list takes them.

## The design system

**Tokens.** `tokens.css` defines every colour, font, size, space, radius, shadow and picture as a
`--lib-*` variable, and the components use nothing else. There are three ways to change them:

1. In `library.json`, `"theme": {"accent": "#4fb3ff", "font-display": "Georgia, serif"}` sets
   `--lib-accent` and `--lib-font-display`.
2. In a `theme.css` loaded after `library.css`, set the variables on `#library`.
3. Any rule of your own: every class starts with `lib-`.

`"scheme"` is `dark` (the default), `light`, or `auto` to follow the player's system.

**Ready-made themes.** `"preset": "win98"` in `library.json` loads `themes/win98.css` after
`library.css`. Each theme sets the tokens and adds the borders, bars and buttons of its era.
`"theme"` tokens still win over it, so a game can keep a theme and change its accent.

| Preset | The look |
|---|---|
| `win98` | Windows 95 and 98: grey bevels, a navy title bar, a teal desktop |
| `xp` | Windows XP: a blue title bar with round corners, beige faces, green progress blocks |
| `vista` | Windows Vista and 7: a glass frame, glossy buttons with a blue glow |
| `system7` | Macintosh System 7: black lines on white, a striped title bar, a grey pattern |
| `aqua` | Mac OS X 10.0 to 10.4: pinstripes, three lights, pill buttons in blue gel |
| `beos` | BeOS: a yellow tab that holds the window's name, soft grey bevels, a blue desktop |
| `nextstep` | NeXTSTEP: four greys, a black title bar, square bevels |
| `cde` | CDE on Sun Solaris, HP-UX and AIX: lavender-grey Motif bevels, a maroon title bar |
| `gnome2` | GNOME 2 with Clearlooks on Linux: a blue title bar, rounded buttons, a striped progress bar |
| `amiga` | Amiga Workbench 1.3: white, black and orange on blue, a striped drag bar, Topaz |
| `atari` | Atari ST GEM: a green desktop, a dotted title bar, the ST's system font |

`"presets": ["win98", "xp", "none"]` lets the player choose between those themes in the frame's
Settings tab (`"none"` is the frame's own look). The choice is saved as `frame.preset`. The HTML
page reads only `"preset"`.

A theme is only CSS, so a game can copy one as the start of its own. The themes use fonts
the player may have (Tahoma, Segoe UI, Geneva, Lucida Grande, Topaz, the ST's system font), with
common fonts after them, and ship no fonts or pictures.
The frame (`gameframe-frame-ui`) builds the same files in and reads the same `"preset"`.

| Group | Tokens |
|---|---|
| Colour | `bg`, `surface`, `surface-2`, `line`, `text`, `muted`, `accent`, `accent-text`, `danger`, `ok`, `scrim` |
| Type | `font`, `font-display`, `font-mono`, `text-s`, `text-m`, `text-l`, `text-xl`, `text-title`, `weight-strong` |
| Space and shape | `space-1` to `space-6`, `radius-s`, `radius-m`, `radius-l`, `radius-pill`, `shadow`, `blur`, `width` |
| Pictures | `backdrop`, `backdrop-opacity`, `logo-width`, `icon-size`, `thumb-width`, `thumb-height` |

**Pictures.** `logo` replaces the title with a picture. `backdrop` sets the picture behind the
card. A save's or mod's card can carry a thumbnail.

**Icons.** The built-in icons are `add`, `download`, `delete`, `up`, `down`, `start`, `file`,
`missing`, `game`, `saves`, `mods`, `screenshots` and `recordings`, drawn in the text colour.
`"icons": {"delete": "icons/trash.svg"}` replaces one with a picture. The picture is used as a
mask, so it takes the colour of the text around it. A section's or page's `icon` is a built-in
name or a picture URL. Scripts get an icon with `gameframeLibrary.icon(name)`.

**Components.** `lib-card`, `lib-header`, `lib-nav`, `lib-section`, `lib-steps`, `lib-notice`,
`lib-rows` and `lib-row`, `lib-thumb`, `lib-tabs`, `lib-button` (with `--primary`, `--quiet`,
`--icon` and `--danger`), `lib-icon`, `lib-callout`, `lib-messages`, `lib-video`, `lib-md`,
`lib-status`, `lib-links`, `lib-fine`.

## Panels

`panels` puts content around the files. Each has a `type` and a `title`. `"place"` is `tab` (the
default, a tab beside the sections), `top` (above the game files) or `none` (only where a page
places it). A `src` is a file next to the page, or any URL, fetched when the page opens.

| Type | Options | Shows |
|---|---|---|
| `markdown` | `src` or `text` | Markdown |
| `changelog` | `src` (default `CHANGELOG.md`), `limit` (default 3) | The newest `## ` sections of a changelog |
| `messages` | `src` (a JSON file) or `items`: `{date, title, text, level}` | Notices; `level` is `warning` or `error` |
| `youtube` | `id`, `label` | A video, loaded from YouTube only when the player clicks |
| `releases` | `repo` (`owner/name`), `limit` | GitHub releases, fetched live |

A port adds its own type from its own script:

```js
gameframeLibrary.panels.scores = async (panel) => {
  const list = document.createElement('ol');
  // fill it
  return list;
};
```

## Pages

`pages` adds Markdown pages, such as help and guides, with a nav bar. Each page has a `path`
(`help` is shown at `#/help`), a `title`, an `icon`, and a `src` file or inline `text`. The
library itself is the home page at `#/`; `"home"` names it in the nav bar. Hash routes work on
any static host with no server set-up. A Markdown link to `#/help` opens that page.

Markdown takes headings, paragraphs, lists, quotes, code, links and pictures, and these
containers:

```markdown
::: tip Where are my saves?
Saves are kept in this browser.
:::

::: panel trailer
:::
```

`tip`, `info`, `warning` and `danger` draw a box. `panel NAME` places the panel whose `name` is
`NAME`. The Markdown is built as page elements, never as HTML, so a file cannot run script in the
page.

## The game's side

- `window.gameframeLibrary.ready` resolves once the page has opened the database. Open the Rust
  store after that.
- Start sends `gameframe-library:start` to `window` and hides the page. `gameframeLibrary.show()`
  brings it back, for example from the game's menu.
- `list(section)`, `read(section, name)` and `add(section, file)` work from any script.
- The game may set `describe(section, name, bytes)` and `accept(section, name, bytes)`. The crate's
  `web::register_hooks` does this.

## Storage

IndexedDB, in a database named after `id`, version 1, with three object stores. `files` holds
each file's bytes. `entries` holds each file's name, size, time and card as JSON text. Both are
keyed `SECTION/NAME`. `state` holds each section's order and switched-off files, keyed `SECTION`,
and the encryption key, under `#key`. With `"encrypt": false` the files are kept as they are.
Turning encryption on or off later leaves the files kept the other way unreadable.

Licence: Apache 2.0, like the rest of GameFrame's code.
