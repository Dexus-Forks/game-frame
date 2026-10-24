---
id: wine-host
title: Wine on this machine, for Windows games
kind: runtime
runtime: [wine-host]
provides: [cpu:x86, cpu:x64, cpu:mmx, cpu:sse, os:windows-9x, os:windows-xp, os:windows-vista-or-later, api:directdraw, api:direct3d-8, api:direct3d-9, api:opengl, api:directsound, api:directinput, api:xinput]
requires: []
vars: [WINEPREFIX]
seen: ['POD: Planet of Death (1997), Wine 11.0 and 11.18 on macOS, 2026-10-23']
---

# Wine on this machine, for Windows games

## What it does

Runs the game in a Wine prefix on this machine. It is the quickest runtime to start, and the
first to try. On an Apple-silicon Mac, Wine runs under Rosetta, and some 32-bit games fault
there; move to `emupc-wine` when they do.

## Set up

```bash
export WINEPREFIX="${WINEPREFIX:-$PWD/derived/wine}"
wineboot --init
```

## Check

`wine cmd /c ver` prints a Windows version.

## Pitfalls

- Read [context-wine-macos](../../knowhow/context-wine-macos.md) before running a game on
  macOS: trace channels, scripted runs, leftover processes.
- A 32-bit system call faults in `wow64cpu` on Apple silicon:
  [wine-wow64-rosetta-fault](../../knowhow/wine-wow64-rosetta-fault.md). Use `emupc-wine`.
- macOS ignores input from a process without the Accessibility permission: send input with
  `tools/live/winput.py` inside the prefix.
