---
id: directdraw-windowed-16bit-desktop
title: A DirectDraw game runs in a window only on a 16-bit desktop, else goes fullscreen
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [display]
api: [directdraw, win32]
symptom: [wrong-mode]
tags: [windowed, fullscreen, desktop-depth, getdevicecaps]
tools: []
skills: [re-observe, re-trace]
docs: [docs/knowhow/context-windows-9x-directdraw.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A DirectDraw game runs in a window only on a 16-bit desktop, else goes fullscreen

## Symptoms

The game has a windowed mode, but every run ends up fullscreen. After the windowed flag is set,
the game switches back to fullscreen on activation or on `WM_DISPLAYCHANGE`.

## Cause

In 1997 a windowed DirectDraw game drew straight into the desktop's pixel format, so it allowed
a window only when the desktop had the game's own depth, usually 16 bits. It reads the depth
with `GetDeviceCaps(PLANES) × GetDeviceCaps(BITSPIXEL)`. A modern desktop is 32-bit, so the game
switches itself to fullscreen (*observed* in POD at three places).

## Evidence

Follow the writes to the windowed flag. A breakpoint with a stack dump on each write names the
code that switches back.

## What to do

Patch the depth reads to return 16, so the game sees a 16-bit desktop. Clear the fullscreen
flag. Then give the back surface the 16-bit format the game draws in
([16-bit pixels on a 32-bit surface](directdraw-16bit-back-surface.md)).
