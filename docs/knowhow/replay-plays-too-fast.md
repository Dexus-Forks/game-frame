---
id: replay-plays-too-fast
title: A replayed race plays several times too fast in the app
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [wrong-result]
tags: [replay, pacing, frame-time, clock, port, app, window]
tools: []
skills: [re-port, re-verify]
docs: [docs/games/interaction.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A replayed race plays several times too fast in the app

## Symptoms

A port that replays a recorded run gives the right results, but in the window the race runs
at several times real speed after the start, while the countdown before it looks right.

## Cause

The app ran one pass of the game loop per redraw. A replay rebuilt from a recording that was
sampled from outside the game has far fewer frames than the original: in POD one per
sample, about a tenth of a second of game time each, against the original's 12 to 20 ms. Each
pass imposes its rebuilt frame time, so one pass per redraw at 60 Hz plays about six seconds of
race per second. The countdown runs on the wall clock (`timeGetTime`), not on frames, so it
looked right.

## What to do

Pace the passes on the game clock, not on their count: for each redraw add the real time
elapsed (in POD's unit, ms × 65) to a budget, and run passes while the budget is positive,
taking from it each pass's frame time. A live run (measured frame times) and a replay (imposed
ones) then both play in real time, and a speed factor is one multiplication.
