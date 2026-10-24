---
id: ptrace-breakpoints-wine
title: ptrace breakpoints in a Wine game leave every thread stopped
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
genre: [racing-sports]
symptom: [hang]
tags: [ptrace, breakpoint, wine, recording, deadlock, code-cave]
tools: []
skills: [re-observe, re-emulate]
docs: [docs/semantics/floats.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# ptrace breakpoints in a Wine game leave every thread stopped

## Symptoms

A Python recorder attached to every thread of the Wine process with `PTRACE_SEIZE`, wrote
`int3` at a few addresses in the game's step, and stepped over each hit. It recorded two or
three hits, then every thread of the game sat in a tracing stop ("t" in
`/proc/PID/task/*/stat`), while the recorder waited in `wait4` for a stop that never came.
After the recorder was killed (with the original bytes written back first through
`/proc/PID/mem`), the game crashed.

## Cause

Not pinned down. Wine uses signals between its threads (suspension, exceptions). A tracer has
to deliver every one of them in the right order, with group stops and interrupt stops handled,
or the process hangs.

## What to do

Record from inside the game instead: a small code cave patched into a copy of the EXE writes the
state at each point of interest to a file (`run/vm/steplog.s`, `run/vm/patch_steplog.py`). The
game runs at full speed, and each record is exact for its instant. Keep the cave inert until a
word in memory turns it on (`run/vm/steplog.py`). Then the patched copy can stay installed.

Room for the cave: the end of the code section's file bytes, and the slack after the
relocations in `.reloc`. Make `.reloc` executable in the copy's section header (the EXE is never
relocated). Put the cave's data in the slack of a writable page (`.idata` past its file bytes).
Call imports through the IAT (`CreateFileA`, `WriteFile`).
