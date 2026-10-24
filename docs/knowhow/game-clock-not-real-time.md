---
id: game-clock-not-real-time
title: The game's clock runs at a slightly different rate from real time
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
genre: []
tags: [clock, timer, timegettime, fixed-point, frame-time, timing, port]
tools: [memrec]
skills: [re-trace, re-observe, re-port, re-verify]
docs: [docs/GAME-PATTERNS.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# The game's clock runs at a slightly different rate from real time

## What to know

A game converts its timer into its own time unit, and the conversion is often rounded. POD
multiplies `timeGetTime()` milliseconds by 65 to get 16.16 seconds, where 65.536 is exact,
so its race clock runs at 65/65.536 = 0.99182 of real time, 0.82% slow (*observed* in
`FUN_004ae894`). Lap times and records are in this slow time.

Measure it: record the game's clock and the wall clock over a long run and divide. POD gave
0.99181 over 490 s, against 0.99182 predicted (*measured*).

For a port:
- Keep the original's factor and units, and mark any change as a deviation.
- Look for a clamp on the frame time (POD: 700 to 30000, 10.7 ms to 458 ms) and for how a
  long frame is split into steps (POD: equal steps of at most 3 × a step unit). With a
  variable step, an exact replay needs the original's sequence of frame times, not only the
  inputs.
- Other clocks may run on real time. POD's countdown uses `timeGetTime` directly (5.5 s of
  wall time), while the race clock and physics use the converted frame time.
