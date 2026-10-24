---
id: capture-edge-rows-repeat
title: A capture's last rows and columns can repeat their neighbours
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [wrong-picture]
tags: [reference-frames, capture, window, scaling, port, menus, verify]
tools: []
skills: [re-verify, re-observe]
docs: [docs/observe/ui.md]
seen: ['POD: Planet of Death (1997), POD Gold 2.8 in Wine in the emulated PC, 2026-10-24']
---

# A capture's last rows and columns can repeat their neighbours

## Symptoms

A port's screen matches a captured frame everywhere but a few dim pixels along the bottom or
right edge. The differences show only on some screens: where the picture differs from its
neighbouring row or column there. A pointer that reaches the bottom edge seems to repeat its
last row.

## Cause

The display that was captured did not show the game's frame pixel for pixel near the edge.
In POD Gold's captures (Wine in the emulated PC, the window cropped to its client area), row
478 repeats row 477, and columns 636 to 638 show columns 635 to 637. Row 479 and column 639
are the window's border. Inferred: the display scales the window by a pixel or so.

## What to do

Compare the capture against the frame as that display would show it. In POD this is
`pod_ui::check::presented`: it copies row 477 to 478 and moves columns 635 to 637 right by
one. Do not change the port's drawing to fit the edge. A rule fitted to the edge, such as a
pointer that repeats its last row when clipped, explains one screen and breaks on the next.

To find the mapping, take a screen whose background varies along the edge. For each
differing pixel, find which pixel of our frame equals the capture there. A whole row or
column with the same offset is presentation, not drawing.
