---
name: re-browse
description: Build and grow a resource viewer for a game - first the catalogue (tools/formats/catalog.py, a static HTML page and JSON manifest of every file, its kind, entropy, hex and what the Python decoders make of it, with pictures), then the project's own viewer app on the port's readers (pictures, palettes, sound, 3D models, text, hex; native and browser; a file output mode for agents). Use at the start of a project, right after the inventory, and after every new decoder, so the person and the agent can check decoded content by looking at it.
---

# Build a resource viewer

A resource viewer opens the player's copy of the game and shows every file: what it is, what
the decoders make of it, and what is not understood yet. Build it on the first day and grow it
with every format. The guide is `docs/working/viewer.md`. This skill gives the steps.

## 1. The catalogue, on day one

Run it after `re-inventory`, before any decoder exists. It lists every file with its kind (from
its first bytes), its size, its entropy and a hex view.

```bash
python3 tools/formats/catalog.py target/game derived/catalog
open derived/catalog/index.html          # for the person
cat derived/catalog/catalog.json         # for the agent: one entry per file, and the summary
```

`derived/` is ignored by git. The catalogue holds game material: never commit it.

As decoders are written (`re-format`), pass each one with the files it reads. The decoder is the
same `decode(reader)` module `validate.py` runs:

```bash
python3 tools/formats/catalog.py target/game derived/catalog \
    --decoder '*.dat=decoders/units.py' \
    --decoder 'graphics/*.pic=decoders/picture.py' \
    --palette target/game/GAME.PAL --palette-offset 0
```

To see pictures, add `images(result)` to the decoder. It returns a list of
`(label, width, height, pixels)`, with one palette index per byte, row by row. For true colour,
add a fifth item `'rgb'` and give three bytes per pixel.

```python
def images(result):
    return [(f'sprite {i}', s['w'], s['h'], s['pixels']) for i, s in enumerate(result['sprites'])]
```

The summary line is the project's format coverage, for example "12 of 47 files decode
completely (31% of 2,345,678 bytes); 1 fails; 34 have no decoder". Copy it into
`docs/CHECKLIST.md` whenever it changes. The exit status is 1 when a decoder fails on a file.

## 2. Look at every result

Open each new decoder's pictures and tables in the catalogue before you port it.

- A picture with slanted stripes has the wrong width. A picture made of noise is still packed,
  or starts at the wrong offset.
- Wrong colours with the right shapes mean the wrong palette. The game may set its palette at
  run time; take it from a capture or a screenshot file (`re-observe`), and label it *assumed*.
- A table with absurd values (a speed of 3e38, a count of 60,000) is a misread type or a
  missed field.

Ask the person to check what only a player knows: which icon is which, whether a face, a ship
or a map looks the way it does in the game. Record what they confirm as *observed* with the
file and entry number.

## 3. The project viewer

When the port has its first reader, build the viewer as an app in the port's language. A
Rust port on wgpu and winit can use egui (`egui`, `egui-wgpu`, `egui-winit` of the same
version as its wgpu; enable egui's `default_fonts` feature, or no text is drawn).

Lay it out like this:

| Part | Where | Rule |
|---|---|---|
| Readers | the port's format crate or module | The viewer has no decoder of its own |
| Host services | one shared crate: window set-up, audio output, loading the player's files, browser glue | The game and the viewer both use it. Move code there from the game; do not copy it |
| The viewer | `apps/viewer` (or the port's equivalent) | A file list by kind, one view per kind of file, a hex view for everything |

Give it these views, as the game's files need them:

- **Executables and overlays:** headers, segments or sections, relocations, export and import
  tables, the overlay or module tables the program uses, the strings.
- **Pictures:** every entry, its header fields, zoom, transparency.
- **Palettes:** swatches.
- **Sound:** events or samples, and playback through the port's own sound code. When the port
  has the original's driver, play the music through it.
- **3D models:** the model drawn and turned with the mouse, its header, vertices and drawing
  commands, and where a walk of the commands stopped.
- **Text:** strings with offsets and visible control codes; message tables when they are known.
- **Unknown files:** hex, until they are decoded.

Each view names the original routine that reads the format and marks *inferred* and *assumed*
choices. Mark the viewer's own presentation choices (a camera, flat shading, a song played on a
different synthesiser) with `Deviation:` comments.

## 4. Native, browser and agents

- Natively, read the game folder given on the command line, defaulting to the project's ignored
  copy (`game-data/original` or `target/`).
- In the browser, read files from a file picker, or from a local development server that
  serves the ignored data folder on `127.0.0.1` only (the port's dev-server script with an
  `--app viewer` choice). Never bundle or upload the files.
- For agents, add an export mode from the start: `viewer --export DIR` opens no window and
  writes every view to files. Write one PNG per picture, palette and font (`NAME.png`), one
  frame of each model or level from a fixed camera (`NAME.3d.png`), and a `manifest.json` with
  every file's kind, its reader, whether every byte was consumed and the coverage line. Print
  the coverage line too. An agent reads the manifest and looks at the pictures to check a
  decoder, compares them with captures of the original (`re-verify`, `tools/verify/`), and
  takes the diary's figures from them (`re-site`). The export holds the game's material:
  write it into `derived/` only.

```bash
cargo run --release -p GAME-viewer -- --export derived/viewer        # no window
jq '.coverage' derived/viewer/manifest.json
```

- Add each new view to the export in the same change that adds it to the window, with a test
  that runs the export on files built in the test.

## 5. Grow it into an editor

When modding is a goal, the viewer becomes the editor, after its readers are proved and before
the game itself. Build it in this order ([Writers, editors and mods](../../docs/working/mods.md)):

1. **Readers that record offsets.** Each reader keeps, for every field a person may change,
   the offset it read it from. A writer then patches those bytes. It never re-serialises the
   decoded view.
2. **One edit type.** Every change is one value of one `Edit` enum: move this vertex, set this
   texture, change this price. The window and a command-line mode both make those values, so an
   agent, a test or a script can do anything a person can
   ([one edit type](../../docs/knowhow/context-editor-one-edit-type.md)).
3. **Exact undo.** Each applied edit keeps the bytes it replaced. Undo writes them back. After
   each edit, decode again from the bytes, so the view shows what will be saved.
4. **Derived data written again.** Bounds, normals and planes stored next to geometry follow
   the vertices. Find them first with `tools/formats/derived.py`.
5. **Behind a build option**, when the project keeps its exact core apart from its additions.

The tests that make it trustworthy: a save with no edits equals the original file on every
file of the kind (`validate.py --write`); each edit kind saves, reads back and finds exactly its
change; an edit and its undo give the original bytes; one edit of each kind loads in the
original game and shows there.

## 6. Test it

- The readers' tests run over every file of the player's copy and skip when it is missing.
- The viewer builds natively and for the browser in CI or before each commit.
- Load the viewer once with the real files and open one entry of each kind. Report what you
  opened and what you saw. "It builds" is not a check.
- When the viewer plays sound you cannot hear, test the sound path headlessly: run the player
  for a few seconds and check that the samples are not silent.

## 7. Land it

Write each format into the project's `docs/FORMATS.md` (or one document per format) with its
evidence, add a README section on running the viewer, and update `docs/CHECKLIST.md` with the
coverage line (`re-land`).

## Pitfalls

- A viewer with its own decoder drifts from the port's. Keep one reader.
- A picture can look right while the decoder skips bytes. Show whether every byte was consumed.
- The viewer's drawing is not the game's renderer. Check the port's frames against the
  original's separately.
- Text renders as empty boxes in egui when the `default_fonts` feature is off.
- In the browser, a hidden tab stops drawing; a viewer that seems frozen may only be hidden.
