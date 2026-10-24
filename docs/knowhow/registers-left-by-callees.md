---
id: registers-left-by-callees
title: 'A value that comes from a register nobody set there: read the disassembly'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [callees, left, registers]
tools: []
skills: [re-emulate, re-ghidra]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A value that comes from a register nobody set there: read the disassembly

## What to know

*2026-10-24, `vh_yaw_rate` (`0x47a00c`), `airlocks_update` (`0x41648c`).*

Two traps met while porting the attitude step and the airlock doors, both invisible in the
decompiler's output and both caught by the emulator on the first comparison.

- **A register kept across a call.** `vh_yaw_rate` loads `EDX = −0x600` as if it were an
  argument to `slot_user` (`0x4fa844`), which takes only EAX and does not touch EDX. After the
  call the code multiplies by EDX: the constant is the yaw's speed factor
  (`0x2000 − 0x600 × speed`). Ghidra drops the "argument" and shows a product with an
  undefined value. Under Watcom's register convention, check the callee's body for which
  registers it writes before deciding what a register holds after a call.
- **A global, not the argument.** `airlocks_update` reads `dt_fix` from its global
  (`0x59a708`). A harness that passes the step only to the routines that take it as an
  argument leaves the global at 0, and the doors never move: 4,161 of 10,000 cases differed
  until the harness wrote the global too. Grep the routine for the frame timer's addresses
  (`0x59a704`, `0x59a708`) before building its harness.
- **Stubbing a callee.** To test `vh_steer`'s collision response without a world, a code hook
  on `0x4f8c68` pops the return address, sets EAX to 1 and writes EIP: the routine returns at
  once with a hit. The same trick logs `0x4f0b44` and `0x4ed7c4` in the airlock case; see
  `isolate-callees.md` for callees that pop stack arguments.
