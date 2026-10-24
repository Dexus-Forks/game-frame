---
id: rebuilt-build-every-function-differs
title: Almost every function differs between two builds, though the game behaves the same
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
symptom: [tool-error]
tags: [version, match, compiler, padding, register-allocation, relocations]
tools: [ghidra-export, decomp_query]
skills: [re-ghidra, re-trace]
docs: [docs/working/versions.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Almost every function differs between two builds, though the game behaves the same

## Symptoms

Matching two releases finds few identical functions: in POD, 374 of 2,942 by masked bytes,
and GameFrame's decompile matcher 384 exact. The diffs show `nop` or `lea eax, [eax]` added,
registers swapped, function exits duplicated, branches inverted.

## Cause

The later build was compiled with other settings (*observed* in POD Gold: code section at
`0x401000` named `AUTO` instead of `0x410000` `BEGTEXT`, jump targets padded with `nop`,
`lea r, [r]` and `mov r, r`, other register allocation). Most "changes" are the compiler's.

## Evidence

Aligned instruction diffs of close pairs (`difflib` on masked instructions): the only
differences were padding and register choice.

## What to do

Grade each pair in steps: same bytes once relocated words, call displacements and padding
instructions are masked; same pseudocode once addresses and generated names are removed; else
compare what the two functions use (numeric constants, strings, mapped globals and callees).
Pairs that use the same things are the same logic. In POD that turned 1,551 "changed" pairs
into 267 same-logic and 1,267 with a concrete note (`labels/gold/changes.tsv`), and the
rule constants of the race could be checked one by one.

## Tried

- Masking relocated words only: 374 exact matches.
- Dropping trailing padding only: no change (the padding is inside functions).
