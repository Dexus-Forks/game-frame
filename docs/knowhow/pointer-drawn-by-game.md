---
id: pointer-drawn-by-game
title: The mouse pointer in the captures is drawn by the game, not the system
kind: context
status: context
era: [late-1990s]
genre: [racing-sports]
symptom: [wrong-picture]
tags: [cursor, pointer, mouse, font, reference-frames, port, menus]
tools: []
skills: [re-trace, re-verify]
docs: [docs/observe/inputs.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# The mouse pointer in the captures is drawn by the game, not the system

## What to know

Frames captured from the original show a coloured pointer that the port does not draw, and a
screen grab of an X display normally leaves the system cursor out.

The window's cursor resource is fully transparent. POD draws its own pointer as a glyph of a
menu font (`#` of font 2) where the mouse is (`FUN_004acfa0`), so the pointer is part of the
game's frame.

Check the cursor resource (`RT_CURSOR`) before treating a pointer as the system's. Port the
pointer; in frame tests whose mouse position is unknown, search for the position at which the
port's pointer matches the capture exactly (`crates/pod-ui/src/check.rs`): that also checks
the font drawing and its colours.
