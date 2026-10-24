---
id: cpu-check-quits-on-unknown-cpu
title: The game exits at once with code 0 and no message on a modern or emulated CPU
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
hardware: [x86-cpu]
symptom: [wont-start]
tags: [cpuid, cpu-detection, silent-exit, qemu, patch]
tools: [insns]
skills: [re-trace, re-observe]
docs: [docs/working/versions.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# The game exits at once with code 0 and no message on a modern or emulated CPU

## Symptoms

The game starts and quits within a second, exit code 0, nothing on screen. A `+relay` trace
(`RelayFromInclude=winpod`) ends with the display queries (`GetDeviceCaps`) and then
`ExitProcess(0)`.

## Cause

POD Gold's CPU check (`FUN_004a3dfc`) sets a CPU kind from CPUID: Intel family 5 or 6, AMD K5
or K6, Cyrix family 5 or 6. Any other CPU keeps the initial `0xff`, and the start-up code
(`FUN_004a1460`) switches on the kind and calls the exit for anything not listed
(*observed*). QEMU's emulated CPU reports `AuthenticAMD` family 6, an Athlon, which did not
exist in 1997. The first edition's check did not quit.

## Evidence

The relay trace's last calls; the start-up function's `switch` with a default that exits; the
VM's `/proc/cpuinfo`.

## What to do

Change the check's default instead of its branches: POD Gold's patch turns the initial `0xff`
into 4 (Intel P6), so any CPU the game does not know is treated as one it does
(`run/wine/patch_winpod.py`). Mark it as a deviation. Before patching, try reading what each
kind selects (here: two sets of drawing routines) and pick the most common one.

## Tried

- Nothing else: the trace pointed straight at it.
