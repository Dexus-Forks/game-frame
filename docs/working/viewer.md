# A resource viewer from the first day

A resource viewer is a small program that opens the player's copy of the game and shows every
file: what it is, what the project's decoders make of it, and the bytes nobody understands yet.
Build it at the start of a project, right after the inventory, and grow it with every format.
It gives the person directing the work a way to check a decoder by looking at its output. It
gives an agent the same output as files it can read: a manifest, pictures and a coverage count.

Two worked examples exist. Un-Banished has a viewer for models, textures, sound and dialogs
(`apps/viewer` in https://github.com/tanepiper/unbanished). The Frontier: Elite II project has
a viewer for MZ overlays, packed pictures, MIDI songs, the original sound driver's music and
the 3D model tables (`apps/viewer` in https://github.com/tanepiper/frontier-decompile).

## Why it pays

- **A picture shows a wrong decoder at once.** A width one byte too small slants the image. A
  wrong palette turns faces green. A wrong vertex layout explodes a model. A table of numbers
  hides all three.
- **The person directing the work can verify content.** They need not read the decoder. They
  open the viewer and say "the 40th icon is the scanner" or "that ship is inside out".
- **An agent can verify content.** A viewer that writes a manifest and pictures to files lets
  an agent compare a decoder's output with a capture of the original (`re-verify`), count what
  is decoded, and find the files it has not looked at.
- **It keeps the unknown in view.** Every file is listed, decoded or not. Files that show only
  as hex are the open questions, and their count goes down as the project goes on.
- **The decoders are proved again.** The viewer calls the same readers the port uses, on every
  file of the player's copy, every time it starts.

## Two stages

| Stage | When | What | Tool |
|---|---|---|---|
| Catalogue | Day one, after `re-inventory` | A static HTML page and a JSON manifest: every file, its kind and entropy, a hex view, the result of every Python decoder, and the pictures a decoder returns | `tools/formats/catalog.py` |
| Project viewer | When the port has its first reader, usually within the first week | An app in the port's language that uses the port's own readers: pictures, sound, 3D, text, hex, natively and in the browser | the `re-browse` skill |

The catalogue needs no code beyond the decoders written for `validate.py`:

```bash
python3 tools/formats/catalog.py game derived/catalog \
    --decoder '*.dat=decoders/units.py' --decoder '*.pic=decoders/picture.py' \
    --palette game/GAME.PAL
open derived/catalog/index.html
```

A decoder shows pictures when it defines `images(result)`, returning `(label, width, height,
pixels)` with one palette index per byte. The summary line ("12 of 47 files decode completely
(31% of 2,345,678 bytes); 1 fails; 34 have no decoder") is the format coverage of the project.
Write it into `docs/CHECKLIST.md` when it changes.

## What the project viewer shows

| Kind of file | View | How it is checked |
|---|---|---|
| Executables, overlays, libraries | Headers, sections or segments, relocations, imports and exports, the overlay or module tables the program uses, the strings | Against `peinfo.py`, `dosinfo.py` and the Ghidra export |
| Pictures, sprites, fonts | Every entry drawn with its header fields; zoom; transparency shown | Against a capture of the original's screen (`re-observe`) |
| Palettes | The 256 colours as swatches | Against a dump of the DAC or a screenshot |
| Sound and music | Events, instruments, waveforms; playback | Against a recording of the original (`re-verify`) |
| 3D models | The model drawn, turned with the mouse; the header, the vertex list and the drawing commands; where a walk of the commands stopped | Against a capture of the model in the game |
| Text and scripts | Strings with their offsets; message tables; control codes made visible | Against the screen where the game shows them |
| Maps, levels, saves | A drawing of the map or a tree of the save's fields | Against the game's own display of the same data |
| Anything not understood | Hex, and an entropy strip (`binview.py`) | It stays listed until it is decoded |

Show the evidence in the viewer. Name the original routine that reads each format, and mark
what is *inferred* or *assumed* (a palette chosen because the game sets its own at run time,
derived vertices placed without the game's camera). A viewer that hides its guesses misleads
the person who checks it.

## How to build it

- **Keep the readers in the port.** The viewer calls the same reader crate or module as the
  port. It never has a decoder of its own. A fix to a reader then fixes the game and the
  viewer together.
- **Share the host code.** Window, audio output, loading the player's files and browser glue
  belong in one shared crate or module that the game and the viewer both use. Do not copy them
  into the viewer.
- **Read the player's own copy, locally.** Natively from the game folder. In the browser from
  a file picker, or from a local development server that serves the ignored `game-data/`
  folder to `127.0.0.1` only. Nothing is uploaded and nothing is bundled.
- **Run where the port runs.** If the port builds for the browser, the viewer does too. People
  can then check content without a build environment.
- **Give agents a way in.** A command-line mode that writes what a view shows to files: a
  picture per entry, a manifest with each file's decode status, a frame of a model. An agent
  reads those. It cannot read a window.
- **Mark the viewer's own choices.** Playback of a song on a different synthesiser, a camera,
  flat shading: each is a `Deviation:` comment, so the ledger shows the viewer is not the game.
- **Fail visibly.** A reader error is shown on the file's page with the offset where it
  stopped. The viewer never stops because one file is wrong.

## When to update it

- After each new decoder (`re-format`): add the view, then re-run the catalogue for coverage.
- Before porting a system that draws or plays something: look at its data in the viewer first.
- When a check against the original fails: open the data in the viewer before reading code.

## Pitfalls

- **The game's palette is often set at run time.** A viewer that guesses a palette must say
  so. Take it from a capture, a screenshot file the game wrote, or the code that sets it.
- **Drawing is not the game's renderer.** A viewer that draws a model with its own camera and
  shading shows the geometry. It does not prove the port's renderer. Compare the port's frames
  with the original's (`re-verify`).
- **A viewer can hide a short read.** Show whether every byte of a file was consumed, as
  `validate.py` does. A picture can look right while the decoder skips a field.
- **The output holds game material.** Keep the catalogue and extracted pictures in ignored
  folders (`derived/`). Never commit them. Screenshots are always fine: copy the ones a document
  or the site uses out of `derived/`, compress them (WebP) and commit them.
