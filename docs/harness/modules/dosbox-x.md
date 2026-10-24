---
id: dosbox-x
title: DOSBox-X, for DOS games and their extenders
kind: runtime
runtime: [dosbox-x]
provides: [cpu:x86, cpu:mmx, os:dos, extender:dos4gw, extender:pmodew, extender:pharlap, extender:cwsdpmi, extender:djgpp, api:sound-blaster, api:gravis-ultrasound, lib:miles, lib:hmi-sos, media:cd, media:cd-check]
requires: []
vars: [GAME_DIR, GAME_EXE]
seen: []
---

# DOSBox-X, for DOS games and their extenders

## What it does

Runs a DOS program in DOSBox-X without a window, through `scripts/dosbox-run.sh`, with its exit
code, a log of its file reads and the debugger's control channel. DOSBox-X emulates the PC,
the Sound Blaster and the Gravis UltraSound, so the game's own sound drivers work.

## Set up

```bash
scripts/dosbox-run.sh "$GAME_DIR" "$GAME_EXE"
```

## Check

The script prints the program's exit code. `--fileio` logs every file it opens.

## Pitfalls

- The debugger control channel needs DOSBox-X 2026.10 or later; `scripts/setup.sh` says so.
- Read [context-dos-dosbox](../../knowhow/context-dos-dosbox.md) first.
- Windows 3.x programs (`os:windows-3x`) also run in DOSBox-X, with a Windows 3.1 installation
  you provide. No module sets that up yet.
