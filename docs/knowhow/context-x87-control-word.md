---
id: context-x87-control-word
title: A game that loads its own x87 control word, such as 0x0c7f (single precision, truncation)
kind: context
status: context
era: [early-1990s, late-1990s]
platform: [dos, windows-9x]
compiler: [watcom, msvc]
hardware: [x86-cpu]
genre: []
tags: [x87, fpu, fldcw, control-word, precision, rounding-mode, truncation, float, 0x0c7f, 0x027f]
tools: [insns, pe32_emu]
skills: [re-trace, re-emulate, re-port, re-verify]
docs: [docs/ABI.md, docs/semantics/README.md, docs/knowhow/x87-magic-add-rounding.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# A game that loads its own x87 control word, such as 0x0c7f (single precision, truncation)

## What to know

Many games of the 1990s set the x87 FPU's control word at start-up. Direct3D did it too: it set
single precision unless told not to. The control word decides the precision of every result
(bits 8 and 9: 24, 53 or 64 bits) and the rounding mode (bits 10 and 11: nearest, down, up,
toward zero). Hardwar's maths start-up loads `0x0c7f` with `fldcw`: 24-bit precision, round
toward zero, all exceptions masked (*observed*, `FUN_004af9a0`). Host floats round to nearest
in 53 or 64 bits, so a port's float maths will differ in the last bits from such a game, and
the differences grow over a simulation.

- Find every `fldcw` and what it loads before you trust any float result:
  `./gameframe run insns GAME.EXE '^fldcw' --functions analysis/export/GAME.EXE/functions.tsv`.
  Read the word each one loads. A helper may change the mode for one conversion and put it
  back ([a rounding helper](x87-truncating-time-format.md)).
- Decode the word: `0x037f` is the default (64-bit, nearest); `0x027f` is 53-bit, nearest;
  `0x007f` is 24-bit, nearest; `0x0c7f` is 24-bit, toward zero. `0x0f7f` is 64-bit, toward zero.
- Measure under the emulator, with the game's own control word set first. GameFrame's
  emulators run the x87 code itself, so the results are the original's
  ([a mode measured in POD](x87-magic-add-rounding.md)).
- In the port, compute those routines with the same precision and rounding: a soft-float
  routine that rounds each result to 24 bits toward zero, or the fixed-point form the routine
  really computes. Mark any other choice as a `Deviation:`.
- A game can change the word later: a renderer's start-up, a DLL, a timer callback. Look for
  `fldcw` in every binary the game loads, and read the word at the points the port measures.
