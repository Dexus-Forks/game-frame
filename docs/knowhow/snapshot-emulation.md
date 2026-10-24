---
id: snapshot-emulation
title: 'Running the original''s code on a whole-process snapshot'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [emulation, snapshot]
tools: [pe32_emu]
skills: [re-emulate, re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Running the original's code on a whole-process snapshot

## What to know

*2026-10-24, the run4 snapshot (`derived/truth/run4/frame-1004.snap`), `docs/sim.md`.*

A snapshot from Hardwar's `vm_snapshot.py` holds every writable mapping of the running
game, the program's own sections included (Watcom marks them writable). Mapped into Unicorn
page by page, it lets one routine of the original run on the real world of that step: the
pilots, the vehicles, the buildings as they were. That answered a question no emulation of
a routine on its own could: which system draws stream A on a paused step. `pilots_update`
(0x46a8d0), called on the snapshot with a hook on `rng_next` (0x4fc4e0) that logs the
return addresses on the stack, made exactly four draws, all in the weapons' fire control.

Three things it needed:

- **A stack outside every region.** Wine's mappings fill most of the low 2 GB and the top of
  the address space; a stack at `0xc0100000` was free.
- **`pop gs` breaks the stack.** Watcom saves and restores GS around some routines (`push gs`
  `0f a8`, `pop gs` `0f a9`). Unicorn loads the segment from a GDT it does not have, and the
  following stack accesses go to the low 16 bits of ESP (the symptom: a read at `0xea88`
  from a plain `pop ebp`). Hooking each `0f a8` / `0f a9` in the code section and replacing
  it with a move of ESP by 4 fixes it.
- **No imports on the path.** `pilots_update` and its callees call no Windows routine; a
  routine that does would need the import stubbed, since the DLLs' code is not in the
  snapshot (only their writable data is).

For GameFrame: a snapshot runner (map, stack, segment-op hooks, a draw logger) belongs next
to `pe32_emu`; the `pop gs` fix applies to any Watcom program under Unicorn.
