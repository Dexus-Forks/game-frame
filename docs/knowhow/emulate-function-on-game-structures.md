---
id: emulate-function-on-game-structures
title: Measure a function that walks the game's data structures under the emulator
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [unicorn]
compiler: [watcom]
genre: []
tags: [unicorn, pe32_emu, measure, ground-truth, structures, graph, globals, sprintf, crt, fuzz]
tools: [pe32_emu]
skills: [re-emulate, re-verify]
docs: [docs/knowhow/x87-magic-add-rounding.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Measure a function that walks the game's data structures under the emulator

## What to know

A function that reads pointers into loaded data (a graph, lists, tables) can still be
measured under Unicorn without running the game's loader: lay the structures out in
scratch memory exactly as the loader does, and point the game's globals at them. POD's
path tracker and order update (`measure/race_paths.py`) were measured this way on
Beltane's whole path graph, 20,000 cases each, and the port matched every one.

- **Read the loader first** for the run-time layout, which differs from the file: POD's
  links point at nodes and at a run of section points chosen by an index, its section
  entries overlap a 16-bit field with a pointer, and the "first list" of a node is the one
  the game walks as its links *out*.
- **Set the guard globals.** A function can return at once on a global the loader sets. POD's
  tracker does nothing while `DAT_00539280` (the first link's points) is 0, and the first
  2,000 cases all came back unchanged until it was set.
- **Make side effects harmless** where the case does not test them: the tracker calls the
  order update, which returns at once when the car stands first.
- **Stop before library calls** that need the C runtime's thread data: POD's `format_time`
  calls Watcom's `sprintf`, which faults under Unicorn (`UC_ERR_WRITE_UNMAPPED`). Stopping
  at the call and reading its arguments from the stack measures the function's own work:
  `emu.call(addr, args, until=CALL_SITE)['stack']`.
- **Load a control word** before the call when the code uses the FPU
  (`emu.call(..., fpu_cw=0x037f)`; [x87-magic-add-rounding](x87-magic-add-rounding.md)).
- **Bias the random cases** towards the branches: points near a gate's plane, trackers at a
  link's last point, cars on the same lap.

Keep each case as a JSON line with the original's answer; the port's test reads the file
and checks every line (`crates/pod-race/tests/measured.rs`).
