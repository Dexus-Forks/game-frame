---
id: variable-frame-step-replay
title: A replay drifts from the original because the game steps by the measured frame time
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
genre: [vehicle-sims, racing-sports, first-person]
symptom: [tool-error]
tags: [frame-time, delta-time, variable-step, replay, determinism, timegettime, clock, main-loop]
tools: [call_trace, memrec]
skills: [re-trace, re-observe, re-port, re-verify]
docs: [docs/GAME-PATTERNS.md, docs/knowhow/game-clock-not-real-time.md]
seen: ['Hardwar (1998), 2026-10-24', 'POD: Planet of Death (1997), 2026-10-23']
---

# A replay drifts from the original because the game steps by the measured frame time

## Symptoms

The port replays the same inputs as a recording of the original, and the two drift apart after
a few seconds. The drift changes from run to run of the original.

## Cause

The game does not run a fixed step. Each frame it measures the time since the last frame and
moves the world by that much. Hardwar sets a global to the milliseconds since the last frame,
capped at 166 ms, and keeps the same delta in seconds × 16,384 beside it. The first two frames
after a reset have a delta of 0, and another function can set the delta from outside
(*observed*, `FUN_004b3564`, `FUN_004b3658`). POD clamps its frame time and splits a long
frame into equal steps ([the game's clock](game-clock-not-real-time.md)). The sequence of frame
times is an input, as real as the controls.

## What to do

Record the frame delta as an input of every frame, beside the controls, and feed the port the
same deltas on replay. Find the timer reads first (`timeGetTime`, `QueryPerformanceCounter`,
`GetTickCount`, `rdtsc`: `./gameframe run insns GAME.EXE '^rdtsc'` and the imports), then the
global that holds the delta, its cap, and any frames that force it to 0. Record those rules in
the clock document.

- Under the emulator, set the delta global directly before each step.
- From a running original, read the delta with `memrec` at each frame, or log the timer's
  return values with a call trace.
- For tests, a fixed delta that the original also accepts (one that is under the cap) gives
  runs that repeat.
