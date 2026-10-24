---
id: unicorn-fpu-starts-single-precision
title: A table the game builds at start-up comes out wrong under the emulator
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
hardware: [x86-cpu]
genre: []
symptom: [tool-error]
tags: [unicorn, x87, fpu, control-word, precision-control, fninit, emulator, lookup-table]
tools: [pe32_emu]
skills: [re-emulate, re-verify]
docs: [docs/semantics/floats.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A table the game builds at start-up comes out wrong under the emulator

## Symptoms

POD's `FUN_004cfde4` builds a square-root table (√j × 65536, truncated). Run under Unicorn,
10,289 of its 24,576 entries came out one above the truncated root, with no pattern by the
fraction (between 0.5 and 1 some rounded up and some did not).

## Cause

Unicorn starts the x87 with the control word 0. Its precision control (bits 8 and 9) is then
00: every result rounds to 24 bits, single precision. A program on Windows starts with 0x037f
(64-bit precision, round to nearest) because the C library's start-up runs `fninit`; in POD
that is Watcom's at 0x4d74e0. A function called directly under the emulator skips the
start-up, so its `fsqrt` and `fmul` rounded to 24 bits (*measured*: the control word read in a
code hook was 0, and the table matched `trunc(sqrt(j) × 65536)` once the word was set).

## What to do

Start every emulated call through a stub that runs `fninit` and loads the control word the
game runs with (`fninit; fldcw [cw]; call target; hlt`). That is not always the start-up's:
POD's start-up leaves 0x037f, but in the race the game runs with 0x027f (53-bit precision),
read from a step log recorded inside the game. Under 0x037f, one target point in about 18,000
game steps came out one unit off. Read the control word in the running game when you can. `measure/race_rules.py`'s
`run` and `measure/controllers.py`'s `run` use such a stub (the latter with 0x027f). Run a start-up routine that fills tables
(here `FUN_004cfde4`) the same way, then keep the filled image for the later calls. When an
emulated float result looks slightly off, read `FPCW` in a hook before blaming the code.

GameFrame's `pe32_emu` now starts the x87 at 0x037f, as Windows starts a program, and
`reset()` puts it back. Pass `fpu_cw=0x027f` (or the word you read) when the game runs with
another one.
