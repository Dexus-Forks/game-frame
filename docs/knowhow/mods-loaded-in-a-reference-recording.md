---
id: mods-loaded-in-a-reference-recording
title: A reference recording was made with modded files loaded
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, virtual-machine]
genre: []
symptom: [wrong-picture]
tags: [recording, reference, mods, ground-truth, shared-emulator, data-directory]
tools: [modfolder, memrec, emulock]
skills: [re-observe, re-verify]
docs: [docs/knowhow/context-mods-install-folder-first.md, docs/knowhow/record-a-run-from-memory.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A reference recording was made with modded files loaded

## Symptoms

A recording meant as ground truth shows changed text on screen (POD's lap counter read
"MOD 1/3" instead of "Tour 1/3"). Or nothing shows, but another session was testing mods in
the same emulator.

## Cause

The game reads files from its install folder before the CD (POD: `Directory` in `Ubi.ini`
first, `FUN_0047ba70`). A session testing mods put changed files there and restarted the
game. Our recorder joined its races. Some mods changed only text and textures; later ones
changed the car's grip, which changes the physics and every lap time.

## Evidence

The HUD in the recorder's screenshots; then the mods session's notes. Its grip-95 runs had
the same best laps as our two recordings (2'33"38 and 2'37"39), which showed that both had
the disc's physics.

## What to do

Before recording a reference, check that no modded file is loaded: list the install folder's
data directory (`C:\POD\Data` must not exist for POD) or compare the files the game opens
with the disc. `./gameframe run modfolder list INSTALL_DIR` says whether a mod it installed is
in place. Record the state in the recording's notes. In a shared emulator, ask the
other sessions first and keep a note of who is using it
(`emulock.py take gf-vm --who SESSION --why "reference recording"`). Re-record when in doubt. POD's
reference race `beltane-3` was recorded after the mods were cleared.
