---
id: ghidra-function-recovery
title: 'A third of HARDWARW.EXE was not in any Ghidra function'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [function, ghidra, recovery]
tools: [ApplyLabels, ExportAll, exportcov]
skills: [re-ghidra, re-format, re-trace]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A third of HARDWARW.EXE was not in any Ghidra function

## What to know

*2026-10-24. Ghidra 12.1.4, Hardwar's `RecoverFunctions.java`, GameFrame's `exportcov`.*

## What was missing

Ghidra's first export of `HARDWARW.EXE` (2026-10-23) had 2,633 functions. Their bodies held
774,013 of the code section's 1,142,272 bytes: 67.8% (*measured*, from the function bodies).
WinMain, the vehicle control function, most of the rasteriser and the Direct3D, DirectPlay
and sample-loading code lay in the gaps. Any trace that passed through them stopped there.

After the recovery: 4,251 functions, 4,169 of them decompiled with no failures (the rest are
thunks to imports), and 1,118,934 bytes in functions: 98.0% (*measured*). What is left is
mostly data in the code section (the runtime's tables at `0x5256c0`-`0x526c81`) and switch
cases still split off (below).

!Map of the code section before and after

## Why Ghidra missed it

Three causes, in order of size (*observed*):

1. **Functions reached only through stored addresses.** Watcom code passes callbacks as
   immediates (`mov [global], offset f`, `push offset f`) and keeps them in tables in
   `DGROUP`. Ghidra's analysis follows calls, not these. The program's own entry point is the
   first case: `0x4e92ce` stores `0x414068` (`win_main`) at `0x5a8434` and jumps into the
   runtime, which calls through it. The 12.8 KB `vh_slot_control` (`0x47b1e0`) is reached
   through 9 stored addresses and no call.
2. **Code nothing refers to.** 991 of the new functions (112 KB) have no call, no jump and no
   stored copy of their address anywhere in the file (*observed*, a search of every 4-byte
   value and every rel32 branch). They are probably code the linker kept but nothing runs
   (*inferred*). Examples: `0x413fc8` beside `win_main`, which also starts `Hardman.exe`, and
   the 74-byte routine at `0x47b16c` just before `vh_slot_control` and its switch table at
   `0x47b1b8`: it lowers a Moth's power and stops at 0 (`vh_power_use`, `docs/flight.md`), so it
   is no dispatcher, but nothing calls it. Check `analysis/notes/recovery/audit.tsv` before tracing one as live.
3. **Two functions wrongly marked as never returning.** `mem_free` (`0x499c48`) and
   `0x499970` both end in `RET` on every path, but Ghidra's "Non-Returning Functions -
   Discovered" analyser marked them so. Every caller ended at the call (the recovered program has 205 and 135 calls to them),
   and the rest of its code fell into the gaps. `game_save` was one (`docs/formats/sav.md`).
   Clearing the mark is not enough: the analyser marks them again on the next analysis
   unless it is switched off.

Two smaller faults came from earlier one-off decompiles saved into the project: a function
at `0x44b6e0` started in the middle of an instruction of `init_light_zone` (`0x44b664`) and
stopped it at `0x44b6dd`. That was the "range Ghidra decodes poorly" in `docs/lighting.md`.
`init_light_zone` is whole now: 226 bytes, ending `RET 0xc` at `0x44b743`.

## How it was recovered

Hardwar's `RecoverFunctions.java`, run on the analysed project, in six phases, with
Ghidra's analysis following the new code after each (Hardwar's `run.sh`
RecoverFunctions.java REPORT.tsv`; about 45 seconds):

| Phase | What it does | Result |
|---|---|---|
| misaligned | Removes a function nothing refers to that starts inside an instruction the function before it falls into; lets that one grow back | 1 removed (`0x44b6e0`) |
| noreturn | Switches the "Non-Returning Functions - Discovered" analyser off, clears the mark on functions whose body has a `RET`, resets the flow at their call sites | `0x499c48`, `0x499970` |
| pointers | Makes functions at code addresses kept in data (relocations outside the code) and at instruction operands that point into code nothing claims | 399 functions |
| gaps | In every run of unclaimed code: skips Watcom's padding (`NOP`, `MOV EAX,EAX`, `LEA EAX,[EAX+0]` and its longer forms, zeros, `INT3`), joins code that the instruction before falls into back onto its function, and otherwise makes a function at the first instruction; repeats until nothing changes (6 passes) | 1,206 functions |
| switches | For `JMP [REG*4 + TABLE]` whose table Ghidra had not read: reads the table, adds the cases, folds functions made at cases into the switch's function | 1 switch, 64 cases, 48 folded |
| merge | Folds a made function that only jumps from one other function reach back into it | 31 folded |

Any function it makes whose body has a bad instruction or a run of zero bytes is removed again
as data (11 rejections, all in the runtime's tables near `0x525980`). Then GameFrame's
`ApplyLabels.java` applied `labels/names.tsv` (240 functions, 3 globals) and `ExportAll.java`
wrote the export, first into `analysis/export.new/` and then moved over
`analysis/export/HARDWARW.EXE/`. The first export is kept in `analysis/export-2026-10-23/`
and the project before the recovery in `analysis/ghidra-project.bak-2026-10-24/`.

## Measuring coverage

`functions.tsv` gives each function's entry and its body's size in bytes. GameFrame's
`exportcov` treats that as one run from the entry, which is wrong when a body is in pieces.
The rasteriser's are: `0x4be4cc` jumps 52 KB forward into its own code. On the new export
`exportcov` reports 93.7%, the body ranges 98.0%. Hardwar's `ExportBodies.java` writes
the exact ranges, and the export now has them as `bodies.tsv` (start, end, entry).
Hardwar's `coverage_map.py` draws the map above from two such files.

## Still open

- Switches that load the case address into a register before jumping are not read by the
  switches phase. Their cases are still small separate functions ending in `JMP`, reached only
  from a table: about 110, for example the 10-byte pieces at `0x493a2e`-`0x4940ad` (table at
  `0x492060`).
- The rasteriser is hand-written: it stores code addresses into globals and jumps through
  them (`0x4be510` stores `0x4bf792`). Each such target is its own function. That is right
  for entry points, but the pseudocode of these pieces is fragmentary; read the disassembly.
- `init_light_zone` keeps two stale error bookmarks from the removed function.

## For GameFrame

`RecoverFunctions.java`, `ExportBodies.java` and `coverage_map.py` belong in GameFrame's
toolkit: the padding forms, the non-returning re-marking and the far-flung bodies are Watcom
traits, not Hardwar's. `exportcov` should read body ranges, and its padding list lacks
`8B C0` and `8D 40 00`, which Watcom uses between functions.
