---
id: timing
title: Game clocks, ticks and frame timing
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine, unicorn]
hardware: [x86-cpu, display, sound-card]
api: [dos-ints, winmm]
compiler: []
genre: []
symptom: []
tags: [timing, clock, timer, tick, frame-time, interrupt, rdtsc, replay, determinism]
skills: [re-trace, re-observe, re-port, re-verify]
docs: [docs/observe/inputs.md, docs/observe/running-the-original.md, docs/hardware/cpus.md, docs/knowhow/game-clock-not-real-time.md, docs/knowhow/variable-frame-step-replay.md, docs/knowhow/replay-plays-too-fast.md]
seen: ['POD: Planet of Death (1997), 2026-10-23', 'Hardwar (1998), 2026-10-24']
---

# Game clocks, ticks and frame timing

## What to know

Find which clock advances the game. It may be a timer interrupt, a display retrace, an audio
callback, a CPU counter, or elapsed time read by the main loop. Record the source, rate, and
point where the game reads it.

Do not treat emulator speed as the game's clock. Fix the emulator settings for repeatable
captures, then measure the game's own tick or frame step. A port that uses a different clock can
run too quickly, drop events, or make a replay drift even when each update routine is correct.

The input guide treats clock reads as inputs. The linked cases cover frame-step replays, clock
rates, and playback speed.
