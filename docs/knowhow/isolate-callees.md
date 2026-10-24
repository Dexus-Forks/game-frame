---
id: isolate-callees
title: 'Measuring a function that calls the whole game: intercept its callees'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [callees, isolate]
tools: [pe32_emu]
skills: [re-emulate, re-ghidra, re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Measuring a function that calls the whole game: intercept its callees

## What to know

*2026-10-24, Hardwar's `isolate.py`, Hardwar's `plot_truth.py`.*

The story's update (`0x46eb60`) calls 36 other functions: pilots, vehicles, map blocks,
buildings, objects. Building enough of the world in the emulator for them to run is not
practical. Instead `isolate.py` finds every direct `call` target in the function (capstone),
hooks each target's first instruction, records the registers and stack words, answers with a
value drawn by the harness, and returns to the caller. The port's test replays the same
answers through its world trait and compares the calls one by one, with the block, the
messages and the random state. Small pure helpers the function relies on (the clock split,
the generator, `msg_post`) are left out of the hook set and run as they are.

Pitfalls met on the way:

- **Callee-popped stack arguments.** Watcom's register convention passes the fifth argument
  on and the callee drops them (`ret 8`, `ret 0x18`). The hook must drop as many bytes as the
  callee's first `ret` does, or the caller's frame shifts. `ret_pops` reads it.
- **One hook per target.** Unicorn keeps every hook added, so two `Isolated` objects on one
  emulator both fire on a shared callee and pop the return address twice. Only the running
  one may act (`active`).
- **Unset words.** Message bodies and command records are built in the caller's frame and
  only some words are written; the rest is old stack. Filling the stack with `0xcc` before
  each run makes those words a known value (`UNSET` in the port).
- **Registers that Ghidra loses.** The start-role notice loop in `plot_update` reloads its
  counter from ECX after the call, which Watcom preserves, so the loop stops after one
  notice. Ghidra's pseudocode shows a three-way loop; the disassembly and the emulator agree
  on one.
- **The worktree has no `derived/`.** Tests that read ground truth need
  `HARDWAR_PLOT_TRUTH=/path/to/main/derived/emu/plot` when run in a worktree, or they skip.
- **Do not trust a pass without a mutation.** The first green run had skipped every test.
  Changing one comparison in the port (`&&` to `||`) must turn the test red.

Candidate for GameFrame next to `pe32_emu`.
