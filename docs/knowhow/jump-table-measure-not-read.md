---
id: jump-table-measure-not-read
title: 'A decompiled switch drops values: run the function for every case instead'
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [tool-error]
tags: [ghidra, decompile, jump-table, switch, unicorn, emulate, watcom, register-calls]
tools: [decomp_query]
skills: [re-ghidra, re-emulate]
docs: [docs/observe/inputs.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A decompiled switch drops values: run the function for every case instead

## Symptoms

A function chooses a few constants by a two-level switch (here: by menu, then by the selected
item) and passes them on in registers. The decompiler shows two of the four values, merged
across cases, and the port draws boxes in the wrong places.

## Cause

Watcom passes arguments in EAX, EDX, EBX and ECX. Ghidra loses register arguments it cannot
type, and merges the cases of a jump table whose blocks only load registers. The values are in
the binary, but reading each case by hand is slow and error-prone.

## What to do

Run the function under Unicorn once per input that selects a case, with the structures it
reads laid out in scratch memory, and stop at the call it makes: the registers there are the
answer. In POD, `measure/menu_outline.py` runs `draw_outline` (`FUN_0043873c`) for all 503
menu items and records x, y, width, height and the style (the style is on the stack above the
return address). The port's table is generated from that file.
