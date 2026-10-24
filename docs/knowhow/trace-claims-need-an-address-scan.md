---
id: trace-claims-need-an-address-scan
title: A port diverges because a trace said "only this function writes that table" and was wrong
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
genre: [racing-sports]
symptom: [tool-error]
tags: [trace, writer, xref, references, ghidra, coverage, capstone, linear-sweep, port]
tools: [refs]
skills: [re-trace, re-port, re-ghidra]
docs: [docs/knowhow/decompile-missed-pointer-functions.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A port diverges because a trace said "only this function writes that table" and was wrong

## Symptoms

The port follows the documented rule, and the recording shows something the rule cannot
do. In POD two cars swapped places while both stood at the same lap and checkpoint, though
the trace said the order changed only at gates (`FUN_0042c264`), and that function was
proved exact against the original on 20,000 inputs.

## Cause

The claim came from searching the decompiled text for the table's address. A second
writer, `FUN_004afd3c` (the order by progress along the race paths), sat in code that
Ghidra had made no function for in the first export, so the text never showed it. A
linear disassembly of the whole code section did not help either: it stopped at data
inside the section near 0x430000 (*observed*).

## Evidence

A scan of the raw code bytes for the table's address as a 32-bit immediate (and for its
neighbours ± 4 and ± 8, which loops use), decoding backwards one to seven bytes from each
hit to find the instruction that holds it. Two new swap sites turned up (0x4b0326,
0x4b0a56), in `FUN_004afd3c` and `FUN_004b0460`.

## What to do

Before writing "only X writes Y" in a document, scan the code bytes for every instruction
that references Y's address (and nearby offsets), whatever function holds it, and list
each writer: `./gameframe run refs GAME.EXE ADDRESS --near 8 --functions
analysis/export/GAME.EXE/functions.tsv` does this, and marks hits in no function. Then search for relative `call`s to each writer to find who runs it. Mark the
claim *observed* with the scan as its evidence. See also
[decompile-missed-pointer-functions](decompile-missed-pointer-functions.md).

## Tried

- `grep` of `decomp.c` and `decomp-extra.c` for the address: found the gate writers only.
- A capstone linear sweep of the code section: stopped early, silently.
