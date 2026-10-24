---
id: ai-chance-from-the-frame-clock
title: The opponents make choices but no random number generator is called
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
hardware: [x86-cpu]
genre: [racing-sports]
symptom: []
tags: [ai, random, determinism, replay, frame-time, behaviour-rules]
tools: [pe32_emu]
skills: [re-trace, re-port, re-emulate]
docs: [docs/games/interaction.md, docs/games/racing-sports.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# The opponents make choices but no random number generator is called

## Context

A search for the game's random number generator finds `rand()` and its callers, and none of
them is reachable from the opponents' code. The opponents still choose between behaviours by
chance. In POD each opponent's rules (from the track file) carry a chance in percent.
`FUN_00462f40` adds the step's frame time to a global (`DAT_00572458`) and uses that global's
remainder by 100 as the draw (*observed*; *measured* under Unicorn on 2,000 states and 15,000
race steps).

## What to know

- The "random" choices depend on the sequence of frame times. A port that is to replay a race
  exactly must replay the recorded frame times, not only the inputs and the seed.
- The accumulator is one global for all cars: the order in which the cars step changes every
  draw after it.
- A list of `rand()` callers is not a list of a game's chances. Look also for a global that
  is added to every frame and then taken modulo a small number or masked.

## What to do

Port the draw as the original does it (the add, the signed remainder kept as a byte, and
here a loop that does not stop at the first rule that takes the draw). Measure it with the
whole controller over many steps, so the accumulator's history is part of the test.
