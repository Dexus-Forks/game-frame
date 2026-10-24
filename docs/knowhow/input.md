---
id: input
title: Game input devices and event timing
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine]
hardware: [ports]
api: [dos-ints, directinput, win32, winmm]
compiler: []
genre: []
symptom: []
tags: [input, keyboard, mouse, joystick, gamepad, key-state, events, focus, cursor]
skills: [re-trace, re-observe, re-verify]
docs: [docs/observe/inputs.md, docs/hardware/other.md, docs/knowhow/directinput-not-foreground.md, docs/knowhow/held-keys.md, docs/knowhow/directinput-cursor-click-timing.md, docs/knowhow/wine-input-without-accessibility.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Game input devices and event timing

## What to know

Find every place the game reads controls. It may poll a key table during a frame, handle window
messages as they arrive, or read a joystick through a timed hardware loop. Record the value the
game sees and the frame or command where it sees it.

Host clicks and keys are only a way to drive the original. They may be lost when the window is
not foreground, held for too little time, or mapped to the wrong game coordinates. For a port,
put the values the game reads into one input record and send them through the same code path on
every run.

The input guide lists read sites for keyboards, mice, joysticks, clocks, files and messages. The
linked cases cover DirectInput focus, held keys, cursor timing and Wine input.
