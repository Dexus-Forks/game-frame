---
id: scene-data-in-plans
title: 'A story''s "scene data" can live in its actors'' AI plans'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [data, plans, scene]
tools: []
skills: [re-port, re-verify]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A story's "scene data" can live in its actors' AI plans

## What to know

*2026-10-24, `crates/plot`, Hardwar's `plot_scenes.py`, plot.md.*

The story's trigger (`plot_trigger` 0x46de5c) takes a code, and nothing in the story calls it
with a constant. The callers were found by scanning every `call` to it in the program
(capstone over `bodies.tsv`, then the nearest constant loads into EAX and EDX): one of them is
the pilots' plan executor, which calls it with word 1 of the pilot's current plan. The story
writes those plans itself when it builds a scene: "pick up the wreck, then plan `0x1d` with
code 2, then deliver, then go home". So the scene data is the sequence of plans, and each
code fires when an actor reaches that point of its errand. Look for this whenever a trigger
has no visible caller: an AI's plan or script executor with one case that calls back into
the story.

Other lessons from the same work:

- **Name unnamed plan types from their routines.** `PilPlanName` answered "Unknown" for the
  story's types, but each type's routine logs a debug error with its own name
  ("DoPcBuzzShip", "DoRescueVh"). The routines are found by following the executor's
  compare tree to each case.
- **`rng_range(n)` gives 0 to `n`.** A table picked with `rng_range(4)` has five entries; the
  port first indexed past a four-entry table. Read the table's length from the draw, then
  check the data.
- **A scene writes its plans from one stack buffer.** Later plans carry the words earlier
  plans left (a 16-bit store into word 1 keeps the high half). Port the buffer, not each plan.
- **Junk register arguments.** Several call sites pass leftover EDX values (or a stack
  address) to `building_find`, which overwrites EDX at once. Check whether a callee reads a
  register before comparing it in a replay test; compare only what it reads.
- **A tail jump into another story function** (`plot_fought` ends with `je plot_crash_scene`)
  runs that function natively: its callees must be hooked too (`Isolated(..., extra=...)` with
  the helper's call targets).
- **The answer may sit in another system.** The ending's unarmed test is not in any story
  function: it is in the buildings' docking check (`0x41fd2c`), under a test of the building's
  kind (`0xf`, the Abandoned Terminal). When a rule is "only X may enter", read the admission
  routine of the place, not the story. Likewise two story flags have their meaning in the
  economy's readers (food made four at a time; fusion goods ordered).
- **Measure coverage of the original, not only equality.** A hook on every instruction of the
  functions under test showed the generator never reached Psycho Bob's later chapters; a
  wider range for four fields brought every non-debug instruction in.

Candidate for GameFrame's `isolate.py` notes (the `extra` targets of natively-run helpers,
the coverage hook).
