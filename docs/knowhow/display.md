---
id: display
title: Display modes, surfaces and palettes
kind: context
status: context
platform: [dos, windows-9x]
runtime: [dosbox, wine, virtual-machine]
hardware: [display]
api: [dos-ints, directdraw]
compiler: []
genre: []
symptom: []
tags: [graphics, video, framebuffer, palette, pixel-format, mode, pitch, resolution, vga]
skills: [re-observe, re-trace, re-verify]
docs: [docs/hardware/graphics-2d.md, docs/hardware/software-rendering.md, docs/knowhow/context-palette-light-tables.md, docs/knowhow/directdraw-16bit-back-surface.md, docs/knowhow/directdraw-windowed-16bit-desktop.md, docs/knowhow/wine-palette-colours.md]
seen: ['POD: Planet of Death (1997), 2026-10-23', 'Hardwar (1998), 2026-10-24']
---

# Display modes, surfaces and palettes

## What to know

Find where the game draws and what state gives those pixels their meaning. DOS games may write
to video memory and the VGA palette. Windows games may draw through DirectDraw surfaces. Record
the mode, dimensions, pitch, pixel format, palette, and page or surface that reaches the screen.

Compare the finished output with the original while keeping palette and display state alongside
the pixels. A correct image buffer can still show the wrong colours, scale, or screen region if
the mode, palette, surface format, or viewport differs. Check the game's drawing code before
changing the desktop or adding a wrapper.

The 2D graphics guide covers VGA and DirectDraw. The software-rendering guide covers exact pixel
rules when the game draws its own frame.
