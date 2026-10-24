---
id: s3-virge-15-bit-surfaces
title: 'S3 ViRGE builds draw 15-bit pixels: their 16-bit surfaces are 5-5-5'
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, glide-wrapper]
hardware: [3d-card, display]
api: [s3d, directdraw]
symptom: [wrong-picture]
tags: [pixel-format, rgb555]
tools: []
skills: [re-trace]
docs: [docs/hardware/card-apis.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# S3 ViRGE builds draw 15-bit pixels: their 16-bit surfaces are 5-5-5

## Symptoms

In a stand-in for the S3D Toolkit, the chip's triangles come out in the wrong colours against
the game's software-drawn pixels, or the reverse.

## Cause

The ViRGE's 3D engine draws 8-, 15- or 24-bit pixels, never 5-6-5: the S3D Toolkit's surface
formats are `S3DTK_VIDEORGB8` (0), `RGB15` (4) and `RGB24` (8). POD's S3 build describes its
16-bit draw surfaces as `RGB15` (*measured*: `sfFormat` 4 in the `S3DTK_SURFACE` it passes),
so on the real card its DirectDraw surfaces were 5-5-5 (*inferred*).

## Evidence

A probe `S3DTKW.dll` dumping the `S3DTK_SURFACE` set as `S3DTK_DRAWSURFACE`:
`{linear address, 640, 480, 4}`.

## What to do

Draw the chip's pixels in the format the game names (`sfFormat`), and make the DirectDraw
surfaces report the same masks (`0x7c00`, `0x3e0`, `0x1f`). POD's software renderer reads the
surface's pixel format and follows it, so both halves agree (`wrapper/ddraw`:
`DDRAW_PIXELS=555`).
