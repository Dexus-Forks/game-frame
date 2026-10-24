---
id: x87-magic-add-rounding
title: A ported fixed-point function differs from the original in a few cases out of thousands
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [unicorn]
compiler: [watcom]
hardware: [x86-cpu]
genre: [racing-sports]
symptom: [tool-error]
tags: [x87, fpu, rounding-mode, control-word, fixed-point, magic-number, double-rounding, port, measure]
tools: [pe32_emu]
skills: [re-emulate, re-port, re-verify]
docs: [docs/COMPILED-CODE.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A ported fixed-point function differs from the original in a few cases out of thousands

## Symptoms

A port of a 16.16 routine agrees with the original almost always, and differs in a handful
of cases when both are run on many inputs. In POD the gate test (`FUN_0042a1f0`, which
decides whether a car crossed a lap line or checkpoint) differed in 15 of 20,000 moves
between a floor model and a round-to-nearest model, all with a point within a hair of the
gate's plane.

## Cause

The routine converts a float sum to an integer with the "magic number" trick: `fadd` of a
large constant, `fstp` to a double, then read the double's low word. In POD's dot product
(`FUN_004db530`) the constant at `0x4fa000` is 1.5 × 2^68, so the low word holds the sum
divided by 65536: a 16.16 result. The division's rounding is the FPU's current mode, and
with the x87 at 64-bit precision it rounds twice: each `fadd` to a step of 32 (the
constant is added to the *first* product, then the others one by one), then the store to a
double to a step of 65536 (*observed* in the instructions; *measured* below).

## Evidence

The original run under Unicorn (`measure/race_rules.py`) with a small stub that loads a
chosen control word (`fninit; fldcw [cw]; call FUNC; hlt`), on the track's own gates and
points placed near each plane. Each control word needs its own model: under 0x037f (64-bit)
each sum rounds to a step of 32 first; under 0x027f (53-bit) straight to a step of 65536;
under 0x0f7f the sums are floored. Each model matched all 20,000 cases under its own word.
Which word the game uses had to be measured in the running game: POD races with 0x027f
(pod-physics' step log reads it each step), not the 0x037f its C library's start-up leaves.
On POD Gold's gate test the words differ in 11 (0x027f against 0x037f) and 20 (against
truncation) of 20,000 cases.

## What to do

Model the conversion exactly in integers: keep the magic constant, add each product in the
order the code does, round each `fadd` to the precision the control word sets, then to the
double's step, and subtract the constant (`crates/pod-race/src/gate.rs`, `dot`). Then read
the control word in the running game (a hook or a step log; the C library's start-up word
is not enough: POD's changes after it), and measure the function under that word. Keep the
measurement as the port's test.

`pe32_emu`'s `emu.fpu_modes(FUNC, cases, cws=(0x037f, 0x027f, 0x0f7f))` runs a routine under
each control word and lists the cases whose result differs; `call(..., fpu_cw=...)` runs it
under one.

## Tried

- A floor model (from an earlier, wrong reading that the game ran truncated): 15 of 20,000
  wrong under round to nearest.
- A 64-bit model (from the start-up's `fninit`): right under 0x037f, but the game races with
  0x027f; corrected on Gold.
- Rounding the exact sum once: matched these 20,000 cases too, but differs from the x87
  wherever the 64-bit step of a partial sum lands on a tie; the sequential model is the
  instruction's own.
