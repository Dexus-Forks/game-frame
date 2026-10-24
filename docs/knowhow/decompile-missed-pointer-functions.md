---
id: decompile-missed-pointer-functions
title: A function has no callers, or no function exists, because Ghidra missed code reached through pointers
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
genre: []
symptom: [tool-error]
tags: [ghidra, function-pointer, coverage, decompile, export, callback, dispatch-table, prologue, winmain, main-loop]
signatures:
  - 'Cannot invoke "ghidra\.program\.model\.address\.Address\.isExternalAddress\(\)" because "addr" is null'
tools: [exportcov, ghidra-export, ghidra-run, decomp_query, refs]
skills: [re-ghidra, re-trace]
docs: [docs/working/ghidra.md]
seen: ['POD: Planet of Death (1997), 2026-10-23', 'Hardwar (1998), 2026-10-24']
---

# A function has no callers, or no function exists, because Ghidra missed code reached through pointers

## Symptoms

A function you know is used has no callers in `decomp.c`. The function that should call it
is not in `functions.tsv` at all. A byte search finds `call` instructions to it at
addresses that lie inside no function. In POD the car physics step (`0x428b98`, 4 KB) was
missing, and with it the whole path from the race loop to the physics.

`ghidra-run ... decompile` with many addresses stopped part-way with a NullPointerException
(`... isExternalAddress() because "addr" is null`), and the remaining addresses never ran.
Given an address inside an unknown function, it made a new function starting there, cutting
the real one in two.

## Cause

Ghidra's auto-analysis follows direct calls from code it already knows. Code reached only
through pointers is not found: per-object function pointers (each POD car has a pointer to
its physics step and one to its driver), handlers registered by name in a table, and
callbacks. Code between those functions is lost with them. *Measured* in POD's
`WINPOD.EXE`: 263 KB of 857 KB of `BEGTEXT` lay outside every function after the first
export. In Hardwar's `HARDWARW.EXE`, also Watcom, about 774 KB of the 1.14 MB code section was
in functions: WinMain, the run loop, the 12.9 KB vehicle control function, two rasteriser
blocks and the Direct3D and DirectPlay code lay in the gaps (*observed*). Watcom functions
often open with a row of register pushes (`push ebx; push ecx; push edx; push esi`) instead of
`push ebp; mov ebp, esp`, which the analysis does not take as a start.

## Evidence

- Sum the sizes in `functions.tsv` and compare with the code section's size.
- Scan the code section for `E8 rel32` calls whose target is a known function; any caller
  address that no function contains is missed code.
- Linear disassembly (capstone) of each gap: a `ret` followed by a push or `mov` is a
  function start.

## What to do

`./gameframe run exportcov GAME.EXE analysis/export/GAME.EXE/functions.tsv --starts
starts.txt` measures the coverage after an export and lists the likely starts in the gaps.
Pass them to `scripts/ghidra-run.sh GAME.EXE decompile OUT.txt $(cat starts.txt)`: it makes the
functions, skips an address that nothing refers to unless it follows a `ret` or padding, and
reports a bad address without stopping the batch.

`./gameframe run refs GAME.EXE ADDRESS --functions analysis/export/GAME.EXE/functions.tsv`
lists every instruction that refers to an address from the raw bytes, and marks those that lie
in no function: each one is missed code.

After every export, measure coverage (bytes in functions / code bytes). If much is missing,
make functions at every call target and after every `ret` in the gaps, skipping the ones
that fail, then decompile what was made into a second file next to `decomp.c`. Then add the
handlers of any registration table (name → function pointer) the same way.

For POD a 40-line Ghidra script did this (create at each address with `createFunction`,
catch each failure, decompile the made functions): 1,123 functions from the gaps, 55 from
the handler table (`analysis/export/WINPOD.EXE/decomp-extra.c`). Before passing an address
to `decompile`, find the real start (after the previous `ret`), or the function will be
split.

## Tried

- `ghidra-run decompile` with all candidate addresses at once: stopped at the first bad
  address (no per-address error handling).
- `ghidra-run decompile` on a call site: made a function there, splitting the physics step.
