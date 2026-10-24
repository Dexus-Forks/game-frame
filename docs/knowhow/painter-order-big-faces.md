---
id: painter-order-big-faces
title: A level drawn from inside shows walls in front of the road
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [wrong-result]
tags: [renderer, painter, depth-buffer, sorting, viewer, port, camera]
tools: []
skills: [re-browse, re-port]
docs: [docs/hardware/graphics-3d.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A level drawn from inside shows walls in front of the road

## Symptoms

A viewer that draws a whole level well from outside (an orbit camera) draws it wrong from a
camera inside it: big flat walls and ceilings cover the road and the cars, and change as the
camera moves.

## Cause

The triangles were sorted by their mean distance and drawn far to near (the painter's
algorithm), which the viewer's egui meshes need: they have no depth buffer. From outside, the
faces are small against the distance to them, and the order holds. From inside a tunnel, one
long wall triangle has its mean far away while it passes right beside the camera, so it is
drawn after nearer faces and covers them. In POD the culprits were Beltane's big untextured
Gouraud faces.

## What to do

Draw with a depth buffer. The cheapest fix that keeps one picture for the window and for
headless frames: rasterise in software at the original's resolution (640 pixels across) with a
depth buffer and perspective-correct texturing, and show the image in the window as one
texture. A GPU depth buffer (egui's paint callback with eframe's `depth_buffer`) is the fast
road for larger sizes.
