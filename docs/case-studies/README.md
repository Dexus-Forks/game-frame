# Case studies

Five projects built with GameFrame, of five different kinds of game, on four different
platforms. Each took a different route. All five aim for the same results as the original
for the same inputs.

| Project | Game | Family | Platform | Route |
|---|---|---|---|---|
| [Un-Banished](banished.md) | Banished (2014) | [City building](../games/management.md) | Windows x64, C++ | Port from the decompile, check each part against the original run in an emulator, then whole runs and saves |
| [Tyrian 2026](tyrian.md) | Tyrian 2000 (1999) | [Arcade action](../games/arcade-action.md) | DOS, 16-bit Borland Pascal | Names from debug symbols; the main loop ported in fragments, each checked from captured states |
| [Frontier](frontier.md) | Frontier: Elite II (1993) | [Space simulation](../games/vehicle-sims.md) | DOS, assembly and C, overlays | Run the original inside the port, replace it routine by routine |
| [POD](pod.md) | POD: Planet of Death (1997) | [Racing](../games/racing-sports.md) | Windows 95, Watcom C, DirectDraw, five builds for CPUs and 3D cards | Formats with writers first; the original under Wine, then in an emulated PC; races recorded from memory; the rules ported before the physics |
| [Hardwar](hardwar.md) | Hardwar (1998) | [Vehicle simulation](../games/vehicle-sims.md) over an [economy](../games/management.md) | Windows 95, Watcom C, four renderers, one resource pack | The story read first for what each system is for; a viewer and reader crates for every format before the game; the foundations traced by parallel sessions |

## The tooling they built to capture a game

Each project needed the same things and built them its own way. GameFrame keeps the generic
part of each.

| Need | Un-Banished | Tyrian 2026 | Frontier | In GameFrame |
|---|---|---|---|---|
| Decompile and search | Ghidra export, `decomp.py` | Ghidra export, labels from debug symbols | Ghidra export, functions seeded from overlay tables | `scripts/ghidra-export.sh`, `decomp_query.py`, `tdinfo.py` |
| Run one original function | Unicorn (`pe_emu.py`) | `dos_emu` | the project's 386 runtime | `pe_emu.py`, `dos_emu` |
| Run the whole original | Windows PC streamed with Steam Remote Play | `dos_env` headless; DOSBox-X with a debugger | the project's runtime, in-process; DOSBox Staging | `dos_env`, `dbx_control.py`, [Running the original](../observe/running-the-original.md) |
| Name the original's state | labels files, RTTI, exported C++ names | generated `Globals` from debug symbols | labels, overlay tables | `labels/`, `td_rust.py` |
| Measure formats | readers checked on every resource, the engine's type registry | `fileio_trace.py` on DOSBox-X file logs | MZ readers | `tools/formats/`, `fileio_trace.py` |
| Drive the game by script | the probe (queries and clicks, in tests, headless runs and the browser) | demo input per frame | the runtime's input | [A model to query](../observe/object-model.md) |
| Compare state | tests against emulator output; saves loaded and continued | data-segment images before and after each fragment | registers and memory after each routine | `bitcompare.py` |
| Compare sequences | random-draw traces | random events by fuzzing | | `call_trace.py`, `seqdiff.py` |
| Compare frames | headless captures and `imgdiff` against the original's stream | screen memory per frame | frames over 60 seconds | [Running the original](../observe/running-the-original.md#comparing) |
| Compare versions of the port | `tools/compat`: replays, saves and frames between builds | | | `re-verify` |
| Track what differs | a deviations ledger generated from code comments | | | `deviations.py`, `coverage.py` |
| Inspect assets | a viewer app | a viewer app, native and in the browser | | |

## What they agree on

- **Name everything first.** The original's names (symbols, exports, resources) make every
  later step faster and every finding easier to check.
- **Capture from the original, then port.** Every port began with a way to run the original and
  record what it does.
- **Build an entry point into the game.** A probe, a state image or a runtime hook: one narrow way in
  that tests, scripts and agents all use ([A model to query](../observe/object-model.md)).
- **Read the story for what each system is for.** The manual and the program's own text name
  the systems the designers cared about, and so which must be exact
  ([Reading the story](../working/lore.md), from Hardwar).
- **Check in small pieces, then as a whole.** Functions, then fragments or routines, then whole
  runs, demos and saves.
