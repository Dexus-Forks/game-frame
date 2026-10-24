---
id: x86-cpu
title: x86 CPUs, instructions and arithmetic
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine, unicorn]
hardware: [x86-cpu, cyrix, mmx]
api: [dos-ints]
compiler: []
genre: []
symptom: []
tags: [cpu, cpuid, rdtsc, x87, fpu, fixed-point, rounding, cyrix, mmx, 3dnow, sse, timing]
tools: [insns]
skills: [re-trace, re-emulate, re-observe]
docs: [docs/hardware/cpus.md, docs/hardware/simd.md, docs/knowhow/cpu-check-quits-on-unknown-cpu.md, docs/knowhow/context-x87-control-word.md, docs/knowhow/wine-privileged-instruction.md]
seen: ['POD: Planet of Death (1997), 2026-10-24', 'Hardwar (1998), 2026-10-24']
---

# x86 CPUs, instructions and arithmetic

## What to know

Start with the instructions the game executes, then find the code paths that use them. The CPU
printed on the box is only a clue. Check for feature probes, timing reads, floating-point setup,
and separate builds or branches for Cyrix, MMX, 3DNow! or SSE.

Keep instruction behavior and CPU selection in the same reference record. A port can choose a
different branch, use a different floating-point precision, or round at a different point even
when its source looks equivalent. Read the disassembly around the relevant instructions, measure
the original on representative inputs, and compare the port's bits and path choice.

The CPU guide covers integer and floating-point formats, timing loops, self-modifying code and
SIMD. It also explains when a hardware-specific branch is part of the game's visible behavior.
