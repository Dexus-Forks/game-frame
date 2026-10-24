---
id: context-port-rules-before-physics
title: Port a game's rules before its physics, driven by a recorded run
kind: context
status: context
era: [late-1990s]
genre: []
tags: [port, rules, core, replay, recording, ground-truth, verify, physics, race]
tools: [pe32_emu, memrec]
skills: [re-port, re-verify, re-emulate]
docs: [docs/working/building-the-port.md, docs/games/racing-sports.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Port a game's rules before its physics, driven by a recorded run

## What to know

A game's rules (laps, checkpoints, the order, the end, the results) can be ported and
proved exact before its physics, if the core takes the actors' motion as an input and a
recording of the original supplies it. In POD the race rules (`crates/pod-race`) replay a
recorded race and end with every word of the eight race records, the order and the results
screen exactly the original's.

What made it work:

- **Cut the core where the original cuts.** POD's physics step moves a car and then runs the
  gate test on the move. The core's `step` takes each car's move (the point before and
  after) and runs the gate test and the rules; drawing, sound and HUD leave as events. The
  same entry point serves the physics port later.
- **Keep the original's memory layout where the results are compared.** The race record is
  kept as its 47 words, pointers stored as the game's addresses, so the final records
  compare byte for byte with the recording.
- **Measure the arithmetic first** (`re-emulate`): the gate test, the order updates and the
  time format ran under Unicorn on 20,000 inputs each before the replay; the replay then
  only tests how they combine.
- **Say what the replay cannot test.** The recording sees about one frame in eight, so the
  replay rebuilds frames at the clocks the game counted each gate at, which it takes from
  the recording. What it tests is everything else: which crossings count, where each time
  lands, places, flags, best laps, the finishing order and the printed results.
- **Compare at every sample, not only at the end**, with a window for reads that straddle
  frames ([replay-sample-straddles-frames](replay-sample-straddles-frames.md)), and check
  the test can fail (a one-unit change to a split was caught).

What it turned up: a second writer of the order table that the trace had missed
([trace-claims-need-an-address-scan](trace-claims-need-an-address-scan.md)), and a wrong
inference about the FPU mode ([x87-magic-add-rounding](x87-magic-add-rounding.md)).
