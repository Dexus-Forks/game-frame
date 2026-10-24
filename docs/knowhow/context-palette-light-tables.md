---
id: context-palette-light-tables
title: Lighting in a 256-colour renderer is a lookup in a shade table, with one level that changes nothing
kind: context
status: context
era: [early-1990s, late-1990s]
platform: [dos, windows-9x]
hardware: [display]
genre: []
tags: [palette, remap, shade-table, colormap, lighting, fog, depth-cue, software-renderer, light-table]
tools: [shadetable, rawimage, catalog]
skills: [re-format, re-trace, re-port]
docs: [docs/hardware/software-rendering.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# Lighting in a 256-colour renderer is a lookup in a shade table, with one level that changes nothing

## What to know

A software renderer with 256 colours cannot multiply a colour by a light value. It keeps a
table of levels, each a row of 256 bytes: the palette colour to draw for each colour at that
light. Doom's `COLORMAP` is one. Hardwar's remap file holds a 64-level light table. Level 0 is
black, levels 31 and 32 leave all 248 set colours as they are, and level 63 is white
(*measured*). The renderer turns a 2.14 intensity into a level with `>> 9`, so `0x4000`,
normal light, is level 32, and the textured span routine skips the lookup at level 32
(*observed*, `remap_lit`, `0x512a55`).

- Find the level that changes nothing. It tells you which intensity the game treats as normal
  light, and so how the numbers of the lighting code map to what is drawn:
  `./gameframe run shadetable REMAP.RMP --offset N --levels 64 --palette PAL --png table.png`.
- Look at the table as a picture. Light tables fade to black and white. Fog and haze tables
  fade every colour toward one colour; the tool names it.
- Files may store the table colour by colour (64 levels of colour 0, then of colour 1). The
  game may scatter them into rows when it loads them. `shadetable` tries both layouts.
- A game can keep several tables: one per crater or time of day, coloured filters, blends for
  translucency. Find which code selects each one before porting the lighting.
- The port draws through the same table, with the same shift, to match frames pixel for
  pixel. Lighting computed in full colour and mapped back to the palette will not match.
