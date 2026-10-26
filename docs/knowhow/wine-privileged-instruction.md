---
id: wine-privileged-instruction
title: A privileged instruction under Wine, from a Windows 95 port probe
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [ports, cyrix, x86-cpu]
symptom: [crash]
tags: [cpu-probe, port-io]
signatures:
  - 'privileged instruction'
  - '\bc0000096\b'
  - 'EXCEPTION_PRIV_INSTRUCTION'
tools: [insns, peinfo]
skills: [re-observe, re-trace]
docs: [docs/observe/running-the-original.md, docs/knowhow/context-windows-9x-directdraw.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A privileged instruction under Wine, from a Windows 95 port probe

## Symptoms

The game stops at once with "A privileged instruction was executed at ADDRESS". The exception
code is `0xc0000096` (`EXCEPTION_PRIV_INSTRUCTION`).

## Cause

Windows 95 let a program read and write hardware ports with `in` and `out`, and run `cli` and
`sti`. Windows NT and Wine do not. Many games of 1995 to 1998 probed the CPU this way. POD reads
the Cyrix configuration registers through ports `0x22` and `0x23` (*observed*). Other probes
read the timer at ports `0x40` to `0x43`, the joystick at `0x201` and a Sound Blaster at `0x220`.

## Evidence

The address in the message is the address of the instruction. Disassemble it, or list every
port instruction of the program with `insns`.

## What to do

Patch the probe in a copy of the program so it returns the answer a modern CPU would give,
without running the port instruction. For a Cyrix probe that is "not a Cyrix" (POD's routine
returns `0xff`).

1. `./gameframe run insns game/GAME.EXE '^(in|out|ins|outs|cli|sti)' --functions analysis/export/functions.tsv`
   lists every such instruction and its function. Do this before the first run of any
   Windows 9x game, so every probe is known.
2. Find what the function's callers expect, and return that from the function's start.
3. Write the patch as a script that checks the original bytes first, and record it as a
   Deviation. Never change the player's own file.
