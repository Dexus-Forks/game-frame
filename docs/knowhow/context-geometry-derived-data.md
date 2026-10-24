---
id: context-geometry-derived-data
title: 'Before moving geometry: find the bounds and normals stored with it'
kind: context
status: context
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
tags: [editor, geometry, bounds, normals, collision, physics, level]
commands: []
tools: []
skills: [re-format, re-trace]
docs: [docs/hardware/graphics-3d.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Before moving geometry: find the bounds and normals stored with it

## What to know

Level files often store data computed from the vertices: per-object bounds, face normals or
planes, visibility. Moving vertices without writing these again gives culling, lighting or
physics from the old shape. Find them first by computing candidates from the vertices and
comparing with unexplained fields, over every object of every file.

In POD (*measured* on all 36 tracks): the six words after each sector are the lowest and the
highest corner of its vertices, exactly. Face normals match Newell's method over all corners on
97.9 % to 99.6 % of faces (dot product above 0.999), the cross product of the first two edges
on only 74 %; the rest are quads that are not flat. With bounds and normals written again, a
raised road carried every car in the game.

## What to do

For each unexplained field near the geometry, test the usual candidates (min/max of vertices,
normals by cross product and by Newell, plane distance) on every record. Keep a test that asserts
the match, and make the editor write the derived fields again after each move.
