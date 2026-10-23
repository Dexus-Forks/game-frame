---
id: directdraw-16bit-back-surface
title: 'A 16-bit game on a 32-bit surface shows a doubled picture, then wrong colours'
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [display]
api: [directdraw]
symptom: [wrong-picture]
tags: [rgb565, rgb555, pixel-format, back-surface, blit]
signatures:
  - 'Failed to blit'
tools: []
skills: [re-observe]
docs: [docs/hardware/graphics-2d.md, docs/knowhow/context-windows-9x-directdraw.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A 16-bit game on a 32-bit surface shows a doubled picture, then wrong colours

## Symptoms

Each row of the picture shows twice, side by side, in the left half of the window. After a
first fix, the colours are wrong. Wine may log "Failed to blit".

## Cause

The game writes 16-bit pixels into a surface that has the screen's format, which is 32-bit
today. Each 32-bit pixel holds two of the game's pixels, so a row fills half the width. The
game also reads the screen's pixel format to choose between RGB565 and RGB555. Wine's software
blitter converts 565 to a 32-bit screen. It does not convert 555 (*measured* in POD).

## Evidence

The evidence is screenshots and the game's own pixel-format query. In that query the green mask
`0x7e0` means 565. The mask `0x3e0` means 555.

## What to do

Give the back surface an explicit RGB565 pixel format (`DDSD_PIXELFORMAT`, 16 bits, masks
`0xf800`, `0x7e0`, `0x1f`). Make the game read its pixel format from that surface instead of
the screen. Wine then converts each frame to the screen.

## Tried

- RGB555: Wine fails to blit it to a 32-bit screen.
