# Case study: POD: Planet of Death

[POD: Planet of Death](https://github.com/tanepiper/pod-planet-of-death) documents and rebuilds
the racing game POD: Planet of Death (Ubi Soft, 1997) for Windows 95, from the player's own
copy. It reads the game's disc image. It decodes each format with a tested reader and writer.
It runs the original under Wine and in an emulated PC. It ports the race rules exactly. It
ships none of the game's files.

## Why it is a useful example

POD is a game of the year when PCs changed fastest. One disc holds five builds of the same
game: software, MMX, 3dfx Glide, ATI Rage (ATI3DCIF) and S3 ViRGE (S3D Toolkit). The code is
Watcom C with register calls, DirectDraw for 2D and DirectInput for the controls. The game
assumes Windows 95: port I/O, a 16-bit desktop, a short list of display modes, a CD in the
drive and an installer that wrote its settings. Nearly every one of those assumptions failed on
a modern machine. GameFrame's [know-how](../knowhow/README.md) began here.

## What the project did, step by step

- **Editions:** a month of work on the March 1997 first edition, then POD Gold 2.8 turned up:
  October 1997 executables, English, 32 tracks, 16 cars, Direct3D and PowerVR builds. Gold
  became the reference, as the last version released, and the work moved to it
  ([a newer edition mid-project](../knowhow/context-newer-edition-mid-project.md)).
- **The disc:** the BIN/CUE read in place, 781 files byte-identical to an extracted copy, with
  a disc reader for the viewer, native and in the browser.
- **Formats with writers:** the block container, text tables, menu images, video stills,
  texture pages, fonts, sound banks, tracks and cars, each with a reader and a writer that gives
  every file back byte for byte ([round trip of every file](../knowhow/writer-round-trip-every-file.md)).
- **Running the original:** under Wine on an Apple-silicon Mac, after patching a Cyrix port
  probe, an unbounded mode table, the windowed mode and the back surface's pixel format. A
  `wow64cpu` fault under Rosetta stayed open after seven attempts. Every one is recorded
  ([the open fault](../knowhow/wine-wow64-rosetta-fault.md)). The project then moved to an
  emulated x86-64 PC with QEMU, Debian and Wine 10, which runs every build
  ([the emulated-PC route](../knowhow/context-emulated-pc.md)).
- **The card builds:** replacement libraries for Glide 2, ATI3DCIF, the S3D Toolkit and
  DirectDraw let the unchanged 3dfx, ATI and S3 builds run in the emulated PC. The Glide build
  draws at any resolution. The ATI and S3 builds turned out to give their chips only part of
  the frame ([3D-card builds](../knowhow/context-3d-card-builds.md)).
- **Mods:** the game opens each data file under its install folder before the CD, so a changed
  file is a mod with no patch ([install folder first](../knowhow/context-mods-install-folder-first.md)).
  Some values in the files are replaced at run time by settings
  ([a value replaced at run time](../knowhow/context-mod-value-replaced-at-run-time.md)).
- **Recording the original:** whole races recorded from the game's memory through
  `/proc/PID/mem` in the emulated PC, checked against every digit on the results screens
  ([record a run from memory](../knowhow/record-a-run-from-memory.md)).
- **Porting the rules before the physics:** the race rules take each car's move as input and
  replay a recorded race and match every word of the original's race records
  ([rules before physics](../knowhow/context-port-rules-before-physics.md)). The gate test and
  the time format were measured under Unicorn on 20,000 cases each, which settled the FPU's
  rounding mode ([x87 rounding](../knowhow/x87-magic-add-rounding.md)).
- **A track editor:** the window and the command line share one edit type. A save with no
  edits is byte-identical. The original game shows the edits
  ([one edit type](../knowhow/context-editor-one-edit-type.md)).

## What it gave GameFrame

- The know-how layer itself: 56 entries on failure modes, dead ends and context, most of them
  from this project, and the hook that names them after a command.
- Tools: `winput` (input from inside Wine), `memrec` (snapshots and recordings of a running
  game's memory), `refs` (every instruction that refers to an address, found in the code
  bytes), `discinfo --iso` (a BIN/CUE's data track as an ISO), `emupc` (the emulated PC's
  files), `shot` (captures that are never overwritten) and `modfolder` (mods in an install
  folder).
- Guides on [recording the original and replaying it](../observe/recording.md),
  [replacement libraries](../hardware/replacement-libraries.md) and
  [writers, editors and mods](../working/mods.md), and more tools: `exportcov`, `memlog`,
  `emulock`, `loopcut`, `derived`, `zcc.sh`, `validate.py --write`, and `pe32_emu`'s control
  word, layout helpers and stop at a call.
- Two crates, `gameframe-mmx` and `gameframe-gpu-1997`, and the guides on
  [MMX code](../hardware/simd.md) and [the card makers' 3D APIs](../hardware/card-apis.md).
- Fixes: PE sections with a VirtualSize of 0, and `peinfo --imports`.

## The lessons in short

- Ask the know-how before you debug a run. Most failures of a 1997 game on a modern machine
  are already known. Some of them are known dead ends.
- When the host's translator fails, move to an emulated PC early instead of patching around it.
- Trust no claim like "only this function writes that table" until the code bytes have been
  scanned for the address ([an address scan](../knowhow/trace-claims-need-an-address-scan.md)).
- Keep a capture of every error dialog as it appears
  ([keep every dialog](../knowhow/context-keep-every-dialog.md)).
