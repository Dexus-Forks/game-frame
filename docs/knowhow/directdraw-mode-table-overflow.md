---
id: directdraw-mode-table-overflow
title: A DirectDraw game asks for an impossible mode, because its mode table overflowed
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [display]
api: [directdraw]
symptom: [wont-start, wrong-mode]
tags: [enumdisplaymodes, setdisplaymode, buffer-overflow]
signatures:
  - 'SetDisplayMode\b.*?\b(?:[1-9]\d{3,}|[4-9]\d\d)\s*(?:bpp|bits)'
  - 'SetDisplayMode\b.*\b(?:[1-9]\d{3,}|[4-9]\d\d)\)'
tools: [insns]
skills: [re-observe, re-trace]
docs: [docs/hardware/graphics-2d.md, docs/knowhow/context-windows-9x-directdraw.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A DirectDraw game asks for an impossible mode, because its mode table overflowed

## Symptoms

The game calls `SetDisplayMode` with a depth or size that no display has, such as 640 × 480 at
1,640 bits per pixel. The call fails with `DDERR_UNSUPPORTED` or `DDERR_INVALIDMODE`. The game
then quits or shows an error about the video mode. The values on screen look like a mix of two
variables.

## Cause

The game's `EnumDisplayModes` callback copies every mode into a fixed table, with no limit. A
1997 display listed a dozen modes. A modern display, or Wine, lists hundreds. The table
overflows into the variables after it (*measured* in POD: a 60-entry table at `0x6920b8`
overwrote the desktop depth).

## Evidence

`WINEDEBUG=+ddraw` shows the modes listed and the mode asked for. A breakpoint on the callback
shows the right value on entry and the wrong one later.

## What to do

Patch the callback so it keeps only modes the game can use (up to 1024 wide, at most as many
as the table holds). Make it return "stop" once the table is full. A code cave in the code
section's zero padding holds the new routine. Record it as a Deviation.

To find the table, start at the callback. Its address is an argument of the `EnumDisplayModes`
call (`IDirectDraw` vtable slot 8). The callback stores to `[table + index * size]` with no
compare.
