---
id: fpu-control-word-thread
title: The game thread's FPU precision is not the one the start-up code sets
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
genre: [racing-sports]
tags: [x87, fpu, precision, control-word, thread, bit-exact, watcom]
tools: []
skills: [re-emulate, re-port]
docs: [docs/semantics/floats.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# The game thread's FPU precision is not the one the start-up code sets

## What to know

Watcom's start-up code runs `fninit` (control word `0x037f`: 64-bit precision). POD runs its
game in a second thread made with `CreateThread`, and a new thread starts with the Windows
default `0x027f`: 53-bit precision, round to nearest (*measured*: the step log stores `fnstcw`
at every step). A port tested under the emulator with the start-up word can agree on thousands
of cases and still differ where 53-bit and 64-bit rounding differ.

Read the control word in the running game, in the thread that runs the code being ported.
At 53-bit precision, x87 `fadd`, `fmul` and `fild` round exactly as IEEE `f64` does. The port can
then use `f64` in the original's order of operations.
