---
id: x87-f64-last-bit
title: A port's floating-point result differs in the last bit from the original's x87 code
kind: failure
status: solved
era: [early-1990s, late-1990s]
platform: [windows-3x, windows-9x, dos]
runtime: [unicorn]
hardware: [x86-cpu]
compiler: [borland, turbo-pascal, watcom, msvc]
symptom: [wrong-picture]
tags: [x87, fpu, double-rounding, extended-precision, double, round, frndint, n-plus, delphi, f64, bit-exact]
tools: [pe32_emu, dos_emu]
skills: [re-emulate, re-port, re-verify]
docs: [crates/x87/README.md, docs/semantics/floats.md, docs/engines/delphi.md]
seen: ['Holiday Island (1997), 2026-10-28']
---

# A port's floating-point result differs in the last bit from the original's x87 code

## Symptoms

A port computes a `Double` with `f64` arithmetic. Under the emulator, the original gives the
same value almost always, and a value one unit in the last place away in a few cases out of
thousands. Once the value is rounded, truncated or compared, a pixel moves or a branch goes the
other way. In Holiday Island the quotient 17616 / −23007 was one of them.

## Cause

The x87 keeps intermediate results in 80-bit registers. At its default precision every `fadd`,
`fsub`, `fmul` and `fdiv` rounds its exact result to a 64-bit significand, and a store to a
`Double` rounds that again to 53 bits. Plain `f64` arithmetic rounds the exact result once to 53
bits. The two differ when the first rounding lands exactly on a 53-bit tie (*measured* under
GameFrame's emulator: integer quotients in Holiday Island's map generator).

## Evidence

The original's instructions run under `dos_emu` (16-bit) or `pe32_emu` (32-bit) on many
inputs, and the stored bits compared with the port's. The differences all sat where the 64-bit
result ended in a 53-bit tie.

## What to do

Compute those steps with [gameframe-x87](../../crates/x87/README.md): load each operand with
`Ext::from_f64` or `from_i32`, do each instruction with `fadd`, `fmul`, `fdiv`, and store with
`to_f64` where the original stores. Keep the original's order of operations and its temporaries
(Borland spills ten bytes, exact; MSVC spills eight, a rounding).

1. List the routine's x87 instructions and every `fldcw` before it
   (`./gameframe run insns GAME.EXE '^fldcw'`); pass that control word with the `_with` methods.
2. Port instruction by instruction with the crate.
3. Compare with the original under the emulator, with the control word set first
   ([x87-control-word-per-run](x87-control-word-per-run.md)).
4. `Round` under `{$N+}` is `fistp` or `frndint` at the current rounding: ties go to the even
   neighbour (`Ext::round_i32`), unlike the 6-byte `Real`'s `Round`.
