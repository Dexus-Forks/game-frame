---
id: context-palette-from-16bit-twin
title: '8-bit textures with no palette in the file: infer it from the 16-bit copy of the same pictures'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: []
hardware: [display, 3d-card]
api: []
genre: []
tags: [modding, textures, palette, per-card-build]
commands: []
tools: []
skills: [re-format]
docs: [docs/working/mods.md, docs/hardware/graphics-2d.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# 8-bit textures with no palette in the file: infer it from the 16-bit copy of the same pictures

## What to know

Games with a software build and a 3D-card build often ship each texture twice: 8-bit
indices for the software renderer and 16-bit colour for the cards. The 8-bit file may hold no
palette at all, because the game builds or loads one at run time. Until that code is traced,
the 16-bit copy gives the colours: lay the two pictures over each other (scale by the size
ratio) and give each index the mean of the 16-bit texels it lies under.

In POD each car comes as `.BV3` (256 × 256 8-bit pages, a palette slot of zeros) and `.BV4`
(128 × 128 RGB565 pages, the same pictures). The palette inferred this way draws the `.BV3`
pages within 3.2 to 7.7 colour levels of the `.BV4` on all 16 cars of POD Gold (*measured*).

## What to do

- Check that the two copies hold the same pictures first: the mean difference after mapping
  should be a few levels, not tens.
- To edit the 8-bit copy (a tint, a repaint), change the 16-bit colours, then map each index
  to the index whose inferred colour is nearest. The file keeps the game's own palette
  indices, so it still works with whatever palette the game supplies.
- Mark the result *inferred*: the game's real palette may differ in indices no texel uses.
