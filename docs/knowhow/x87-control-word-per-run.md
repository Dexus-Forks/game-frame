---
id: x87-control-word-per-run
title: 'Set the game''s x87 control word before each emulated run'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [control, per, run, word, x87]
tools: [pe32_emu]
skills: [re-emulate, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Set the game's x87 control word before each emulated run

## What to know

*2026-10-24, Hardwar's `audio_truth.py`.*

Hardwar runs the x87 with control word `0x0c7f` (24-bit precision, truncation), set once at
start-up by `fpu_setup` (`0x4af9a0`, maths.md). A routine emulated on its own
never runs that set-up, so Unicorn's FPU stays in its default mode (64-bit precision, round
to nearest), and every `fist`, `fstp` and division rounds differently from the game.

Symptom: the sound routing matched the port in 19,294 of 20,000 cases; the rest differed by
one unit in a pan word, which comes from `vec3_unit_from_i32` (`0x4b2784`, an `fistp` of a
scaled float). Nothing else differed, so it looked like an off-by-one in the port.

Fix: write the control word before each run,
`emu.uc.reg_write(UC_X86_REG_FPCW, 0x0c7f)`. With it every case matched. Any routine that
reaches the x87 (lengths, normalising, the sky and the light models) needs the same; the
alternative is to run `fpu_setup` once, which needs `.bss` zeroed first
(emulator-bss.md).

For GameFrame: a `pe32_emu` option for a start-up FPU control word, and a pitfall entry
("emulated routines miss the program's FPU set-up").
