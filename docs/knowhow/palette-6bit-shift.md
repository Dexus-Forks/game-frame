---
id: palette-6bit-shift
title: The port's colours are a little brighter than the original's, because 6-bit palette values were scaled to 255
kind: failure
status: solved
era: [1980s, early-1990s, late-1990s]
platform: [dos, windows-9x]
hardware: [display]
genre: []
symptom: [wrong-picture]
tags: [palette, vga, dac, 6-bit, shift, colour, pixel-exact, 252]
tools: [rawimage, catalog]
skills: [re-format, re-browse, re-verify]
docs: [docs/hardware/README.md]
seen: ['Hardwar (1998), 2026-10-23']
---

# The port's colours are a little brighter than the original's, because 6-bit palette values were scaled to 255

## Symptoms

A frame of the port and a capture of the original differ in almost every pixel by a few
levels. The brightest colours are 255 in the port and 252 in the original.

## Cause

VGA palettes hold 6-bit values (0 to 63). A tool that turns them into 8 bits by scaling maps
63 to 255. Many games shift each value left by 2 instead, so 63 becomes 252, and the screen
shows those values. Hardwar's palette loader does this with a `shl 2` for each component before
it sets the colour (*observed*, `pal_apply`, `0x4aaca8`). Its palettes also set only 248 of
the 256 colours.

## What to do

Read the palette loader and use its rule. GameFrame's `rawimage` and `catalog` take
`--dac shift` for games that shift, and `--dac scale` (the default) for ones that scale. The
catalogue fills a palette with fewer than 256 colours with magenta, so unset colours show.
For a pixel-exact interface the port's palette code must use the original's rule; record it in
the palette's format document.

```bash
./gameframe run catalog game derived/catalog --palette PAL --palette-offset 4 --dac shift
./gameframe run rawimage dump.bin frame.png --mode linear8 --palette PAL --dac shift
```
