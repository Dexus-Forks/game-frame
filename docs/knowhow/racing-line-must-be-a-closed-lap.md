---
id: racing-line-must-be-a-closed-lap
title: A line-following driver stalls or wedges at the same place every lap
kind: failure
status: solved
era: []
platform: []
runtime: []
hardware: []
api: []
genre: [racing-sports]
symptom: [tool-error]
tags: [racing-line, line-follower, recording, lap, automation]
signatures: []
tools: [memrec, loopcut]
skills: [re-observe]
docs: [docs/games/racing-sports.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A line-following driver stalls or wedges at the same place every lap

## Symptoms

A driver that steers along a recorded racing line runs well, then turns into a wall at one spot
and stays there; its log shows the line index stuck near the line's last point.

## Cause

The line was recorded for a fixed time shorter than a lap. At its end the next aim point is the
line's start, far across the level (*observed*: a 60 s recording on POD's Alderon, where a lap
takes about 70 s).

## What to do

Record more than a lap from a computer car and cut it where the car comes back within a few
units of its first point, so the line is a closed loop:
`./gameframe run loopcut REC.jsonl --x car_x --y car_y --index 1 --out LINE.jsonl`. Check the gap between the last and first
points before using a line.
