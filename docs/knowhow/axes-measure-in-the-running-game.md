---
id: axes-measure-in-the-running-game
title: A 3D view draws every level upside down and nothing looks wrong
kind: failure
status: solved
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
symptom: [wrong-picture]
tags: [axes, coordinates, handedness, viewer, editor, height, mirror]
signatures: []
tools: [memrec]
skills: [re-browse, re-observe]
docs: [docs/observe/memory.md, docs/working/viewer.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A 3D view draws every level upside down and nothing looks wrong

## Symptoms

An editor's "raise" lowers the ground. In the viewer, tall objects hang below the level, but a
view from above still shows every road clearly, because the tunnel roofs are drawn under it.

## Cause

The viewer's axis convention was chosen from drawings of car models, whose height axis grows
downwards, and applied to tracks, whose height axis grows upwards (*measured*). A level mirrored
in height and seen from above looks plausible: an orthographic view of a mirrored scene is a
believable picture.

## Evidence

From the running game: the race keeps each car's position in the level's coordinates. Over three
stretches of POD's Beltane exactly one level face lies under the car, and the car is 0.45 above
it (road z 15.0, car 15.45; 0.0 and 0.45; a slope 54.8 to 58.8, car 56.7). A road raised in +z
lifted all eight cars in the game. A line follower's Left key turns the car anticlockwise in
(x, y) as seen from +z.

## What to do

Settle each model kind's axes from the running game before building tools on them: read an
object's position from memory over geometry you know (a car on a road, a player on a floor) and
find the face under it. Check handedness with a turn: the direction a Left input turns as seen
from +up. Store the convention per model kind (POD: tracks up, car models down) and test it (a
point with larger height projects higher on screen).
