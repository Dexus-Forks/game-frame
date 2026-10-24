---
id: emulate-step-from-step-log
title: Run the original's per-object step under the emulator on states from the running game
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, unicorn]
genre: [racing-sports]
tags: [ground-truth, emulator, snapshot, step-log, physics, bit-exact]
tools: [memrec]
skills: [re-emulate, re-verify, re-port]
docs: [docs/semantics/floats.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Run the original's per-object step under the emulator on states from the running game

## What to know

Random inputs make poor ground truth for a physics step. The step reads the level's geometry,
tables built at start-up, and counters other objects advance. What worked for POD:

- One **whole-process snapshot** of the running game (every writable mapping) gives the level,
  the tables and the heap at their real addresses.
- A **step log** written from inside the game (`ptrace-breakpoints-wine` for why) gives, every
  step, the blocks that change: the objects, the globals, and each object's state right before
  and right after its own step. Make the log describe itself with a table of (address, length)
  blocks, so a missing block can be added without changing the reader.
- Under Unicorn, put the step's blocks over the snapshot and run the original function. When it
  does not reproduce the next record, trace its memory reads and diff them against a second
  snapshot to find the state that is missing from the log. In POD that state was a counter
  (`0x502114`), the race records changed by earlier cars in the same step, and the scaled frame
  time.
- Stub what reaches the OS (sound), and give Unicorn a flat GDT: library code may reload segment
  registers.
- Once the whole step reproduces the recording, record each callee's calls inside it (inputs as
  the bytes read before being written, outputs as the bytes written). Test the port's functions
  on those calls, then the port's whole step on every recorded step.

In POD all 20,488 recorded car steps were reproduced first by the original under the emulator,
then by the port.
