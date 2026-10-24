---
id: docked-screens-640x400
title: 'The front end and the docked screens run at 640 x 400'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [640x400, docked, screens]
tools: []
skills: [re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# The front end and the docked screens run at 640 x 400

## What to know

*2026-10-24, measured in the running original (`docs/ground-truth.md`).*

With `ResolutionX` 640 and `ResolutionY` 480, the window was 640 x 400 in the front end, the
pause console and the docked computer, and 640 x 480 in flight (*measured*: the window's
geometry, and the mode at `0x599efc`/`0x599f00` read from the recorder's memory dumps).

Why: the console code sets 640 x 400 at 8 bits itself, at `0x4435ce`
(`FUN_0049b3b8(0x280, 0x190, 0, 0x200)`); flight sets the configured mode through the calls at
`0x43b3b7`-`0x43b626` (*observed*). `WM_GETMINMAXINFO` (case `0x24` of the window procedure)
caps the window at the current mode (`0x599a28`/`0x599a2c`), so it cannot be resized from
outside.

What it means for recorded frames: the docked computer is drawn into 400 lines, and its
cockpit frame is cut at the bottom of the window. Whether the original under Windows 95
shows the same cut, or draws the docked screens into a 480-line mode, is *unknown*. Compare
with a frame from a real Windows machine before treating the cut as part of the reference.
