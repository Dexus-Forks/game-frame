# Case study: Hardwar

[Hardwar](https://github.com/tanepiper/hardward-decomp) documents and rebuilds Hardwar (The
Software Refinery, 1998), a flight and trading game for Windows 95, as a 1:1 reimplementation
in Rust. It reads the player's own copy: the resource pack, the videos, the CD audio and the
saves. It ships none of the game's files.

## Why it is a useful example

Hardwar is a vehicle simulation over a living economy. The player flies a small craft through
a city of nine craters on Titan, trades, takes missions and follows a plot that changes the
city. Factions, traders, the police and factories run while the player is elsewhere. The game
is one Watcom C program with four renderers (software, Direct3D, 3dfx Glide and PowerVR), and
all its assets are in one pack. It is the first GameFrame project that read the game's story
as evidence for its design ([Reading the story](../working/lore.md)).

| | |
|---|---|
| Developer | The Software Refinery, Leeds. Published by Gremlin Interactive in Europe and Interplay in the US |
| Release | 1998, Windows 95, DirectX 5, two CDs with audio tracks |
| Program | One 32-bit executable, about 1.5 MB, linked 1998-08-13. Watcom C: sections `BEGTEXT`, `DGROUP`, `.bss`, register calls. A separate MFC launcher |
| Kind of game | [Vehicle and space simulations](../games/vehicle-sims.md) with the economy rules of [management](../games/management.md) |
| Time model | Real time with a world clock in days and hours, a variable frame step capped at 166 ms, and pause as a world-level input |
| What must repeat | The saves hold the whole world. The interface is pixel-exact in the vanilla build, by the user's decision; higher resolutions are a layer on top |

## Formats

Every asset is in one pack of 621 named files. Each format became a Python decoder for the
catalogue, then a Rust reader crate tested on every file of its kind, then a panel in the
project's viewer.

| Format | What it is | How it was found |
|---|---|---|
| The pack | Data one file after another, a directory of 28-byte entries at the end, and a final word that points to it. Entry names XORed with a 16-byte key | The open function seeks to 4 bytes before the end. The key showed in the file before the code was read: names are padded with zeros, so the padding stores the key itself (`xorscan --records`). The key is the developer's name, stored in the program |
| Palettes | A first index, a count, 6-bit RGB. The loader shifts each value left by 2, so 63 is 252. Only 248 colours are set | The palette loader's `shl 2` ([6-bit shift](../knowhow/palette-6bit-shift.md)) |
| Sprites | A 48-byte header, 8-bit pixels, an optional palette, and a second block the 2D blitter never reads | The 2D blitter reads the header fields |
| Remap tables | One 32-level shade table, or the game table: blend rows, a 64-level light table, four alternate tables, four filters | The light table's level 32 leaves every colour unchanged; a 2.14 intensity `>> 9` selects the level ([light tables](../knowhow/context-palette-light-tables.md)) |
| Fonts | Fixed cells and an advance width per character | The glyph routine |
| Object trees and 3D shapes | Trees of named parts placed relative to each other; shapes with i16 vertices and a scale shift, compact triangles and quads, uv in 2.14, normals × 16,384 | Measured on all 217 shapes, then traced |
| The world | Typed records placing 1,424 shapes 2,307 times, 2,378 lights, materials bound to polygons by name | The record walker and the material lookup |

At the end of the viewer milestone 614 of the pack's 621 entries decoded completely in the
viewer; the 7 left are force-feedback effects.

## Where things were found

- **Lighting.** A vertex's light is a sum: a base from the material and the ambient, the sun
  by the normal, point lights with a soft falloff, a cap at normal light, then the extra
  lights. The city has no sun. Its light comes from eight crater lights fixed in the program
  and scaled by a time factor that never reaches zero.
- **A stored value that is thrown away.** The crater lights store a height of 0. The function
  that places them overwrites the height with the light's radius shifted right by 3 before it
  passes the position on. The pseudocode did not show it; the disassembly did
  ([register arguments](../knowhow/watcom-pseudocode-drops-register-arguments.md)).
- **The foundations.** One Park–Miller generator with three seeds (main, a second stream, the
  renderer's), found by its multiplier 16,807 (`constants`). A maths start-up that sets the
  FPU to single precision with truncation ([control word](../knowhow/context-x87-control-word.md)).
  A variable frame step ([replays](../knowhow/variable-frame-step-replay.md)). A world clock of
  `0x10000000` units a day.
- **A log the game writes itself.** One function, called from 777 places, writes trades,
  manufacturing and story events to a file when a registry value is set. Under the emulator it
  is a trace of the simulation.
- **Code outside functions.** About a third of the code was in no function after Ghidra's
  analysis: WinMain, the run loop, the vehicle control function, two rasteriser blocks
  ([missed functions](../knowhow/decompile-missed-pointer-functions.md), `gaps`).

## What the story explained

The manual on disc 1 and about 200 long strings in the program (radio bulletins, faction and
police notices, e-mails, descriptions of goods) told the setting, the plot and many rules.
They explained what the code had shown and named systems still to find: craft that run on
solar cells, so lighting is game logic and the crater lights follow the time of day; a foggy
moon, so the fog is part of the look; two factions holding water and food, so the economy is
the core; a ceiling tied to the power cell; a clock with hours that rules run on; a plot that
reshapes the city, which the save must hold; and an ending that takes only an unarmed pilot.
The whole account is [Reading the story for design decisions](../working/lore.md).

## What worked

- **A viewer before the game.** The catalogue showed every decoder's output as pictures from
  the first day. The Rust viewer followed on the reader crates, with an `--export` mode that
  writes PNGs and a manifest for agents to check.
- **Reader crates tested on every file.** Each crate runs over every file of its kind in the
  player's pack and skips without it. The pack crate rebuilds the pack byte for byte, so a mod
  tool can replace an entry.
- **Measuring the data to settle a reading.** Which axis is up, and whether the crater lights
  hang above the floors, were measured over the world's polygons before the viewer changed.
- **A diary with figures from the start.** Every entry has captioned pictures from the
  project's own viewer and hand-drawn SVG diagrams of the formats, on a site themed in the
  game's colours. The site's workflow runs by hand only, to save Actions minutes.
- **Parallel sessions on the foundations.** Four sessions traced the random numbers, the
  maths, the clock and the saves at once, each in its own worktree
  ([worktrees](../knowhow/worktree-ignored-folders-missing.md)).

## What it gave GameFrame

- The lore step in the method: `re-inventory` "Read the story", the `docs/lore.md` template
  and the guide [Reading the story for design decisions](../working/lore.md).
- Tools: `sentences` (a program's long strings, for reading the story), `shadetable` (shade,
  fog and light tables of palette games), `gaps` (code Ghidra did not turn into functions,
  with likely starts), `xorscan --records` (a key XORed into one field of every directory
  record, found from the padding and then in the executable), `rawimage` and `catalog
  --dac shift`, and Park–Miller reported by `constants` when its modulus is beside it.
- Know-how on Watcom pseudocode, the x87 control word, variable frame steps, 6-bit palettes,
  light tables, worktrees, and reading the story.

## The lessons in short

- Read the manual and the program's sentences before the code. They say what each system is
  for and which must be exact.
- Read the disassembly of every Watcom function the port depends on.
- Measure the share of code in functions after the first export, and recover the gaps first.
- Record the frame time as an input from the first replay.
