---
id: record-a-run-from-memory
title: Record a whole run of the original from its memory as the reference for a port
kind: context
status: context
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
genre: []
tags: [ground-truth, recording, proc-mem, memory, reference, port, verify]
tools: [memrec, memscan]
skills: [re-observe, re-verify, re-port]
docs: [docs/observe/memory.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Record a whole run of the original from its memory as the reference for a port

## What to know

A port that must behave like the original needs a reference run of the original: the state
of every actor over a whole game, with the game's own results to check it against. When the
game runs under Wine on Linux (natively or in a VM), the simplest recorder reads the
process's memory from outside through `/proc/PID/mem` as root. The game is not changed or
slowed, and nothing is injected.

GameFrame's `tools/live/memrec.py` is that recorder: `snap` copies the writable memory for
`memscan.py` diffs, and `record SPEC` writes named values as JSON Lines. What worked for POD,
whose recorder it came from:

- **Find the state first**, by tracing and by memory diffs: the eight race cars, the race
  records, the order table, the clock, the frame time. Static addresses stay the same in
  every run of a non-relocated EXE.
- **Keep raw words.** Store 16.16 values as integers in JSON Lines, one sample per line, so
  nothing is rounded.
- **Sample at a fixed rate** (10 Hz) and add the game's own counters: the clock, and the sum
  of frame times. They date each sample in game time.
- **Read the final state** a few seconds after the run ends: games often finish their
  bookkeeping after the main loop (POD drives unfinished cars to the line).
- **Check against the screen.** Take screenshots at events (each lap) and at the end. Print
  the recorded values the way the game prints them, and compare digit for digit. In POD all
  times on four results screens matched once the reader formatted like the game
  ([x87 time format](x87-truncating-time-format.md)).

Limits:
- A sample is not atomic: the values are read one after another while the game runs.
- At 10 Hz an outside reader sees about one frame in eight. Per-frame inputs (each frame's
  time, the keys) need a hook inside the game.
- A screenshot taken in the sampling loop stops sampling while it runs (0.7 s in POD).

Related: [mods spoil a reference recording](mods-loaded-in-a-reference-recording.md),
[wait for the next race](wait-for-the-next-race.md).
