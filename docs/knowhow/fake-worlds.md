---
id: fake-worlds
title: 'Emulating game rules on a world built by hand'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [fake, worlds]
tools: []
skills: [re-emulate, re-ghidra, re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Emulating game rules on a world built by hand

## What to know

*2026-10-24, Hardwar's `pilots_truth.py` (pilots and Police).*

The Police and plan routines take pointers to pilot and building records and read a dozen
globals. They can be run on their own under `pe32.py` without loading a game:

- **Records at known addresses.** Map one region, put pilot *i* at `base + i × 0x400` and the
  null record at index 15, and point the globals at them (the null-pilot and null-building
  copies `0x582138`/`0x595ad8` and `0x582134`/`0x595ad4`, the faction HQs `0x585a7c` to
  `0x585a8c`, the building list `0x58214c`, the limbo building `0x582150` with its installed
  flag `0x582148`). Writing results back as indices (pilot 0x1000 + i, building 0x2000 + j)
  makes them comparable with a port that uses indices.
- **Stub callees with a code hook.** A hook on the callee's first instruction records its
  registers, sets EAX, pops the return address into EIP. For a callee that ends in `RET n`,
  move the return address up by `n` bytes first (`VhGetPower` 0x47f00c takes one stack word).
- **Stack arguments.** Watcom passes four registers (EAX, EDX, EBX, ECX) and the rest on the
  stack, cleared by the callee: write them above the return address before `Emu32.call`.
- **Map tables by running the code.** Rather than reading a binary-search `switch`, call the
  routine for every value: `PilPlanName` 0x4652c8 for each plan type gives the names, and the
  plan executor 0x45e2e8 with a hook on the DoPc range gives which action each type runs.
- **Stack garbage in records.** Some routines build a 24-byte plan on the stack and fill only
  the words they use; the rest is whatever the stack held, so fill the stack with a marker
  before the call and leave those words out of the comparison (and mark the port's choice
  `Deviation:`).

Ghidra's decompiler drops most register arguments of these routines (it shows them as
placeholders named after the registers), so read them as machine code: capstone over the image, restarting after any byte
it cannot decode, gives a whole-program listing in a few seconds.
