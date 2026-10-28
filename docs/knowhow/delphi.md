---
id: delphi
title: 'Delphi programs: forms, class tables and event handlers'
kind: context
status: context
era: [late-1990s]
platform: [windows-3x, windows-9x]
compiler: [borland]
tags: [delphi, vcl, object-pascal, dfm, tpf0, vmt, rtti, published-methods, event-handlers, win87em, n-plus, ne]
tools: [dfm, delphi1, dosinfo, exportcov, dos_emu]
skills: [re-inventory, re-ghidra, re-emulate, re-port]
docs: [docs/engines/delphi.md, docs/DOS.md]
seen: ['Holiday Island (1997), 2026-10-28']
---

# Delphi programs: forms, class tables and event handlers

## What to know

A Delphi game is a set of forms whose event handlers hold the game's code. Read the forms
first with `tools/formats/dfm.py GAME.EXE derived/forms/`, then name every handler with
`tools/dos/delphi1.py GAME.EXE labels` (Delphi 1, 16-bit) and give that file to
`scripts/ghidra-export.sh --labels`. The guide is [Delphi programs](../engines/delphi.md).

- **Recognising it.** `RCDATA` resources that start with `TPF0`, class names such as `TForm`
  and `TApplication` as Pascal strings, the Borland Pascal runtime in `dosinfo`.
- **Handlers are not called.** The VCL finds each event's method by name in the class's method
  table when it reads the form. No call instruction leads there, so Ghidra's export can miss
  them. `delphi1.py handlers` maps each form's events to code.
- **New objects are zeroed.** `InitInstance` fills every new object with zeros. A port must
  start every field of a form at 0.
- **Floating point.** With `{$N+}` the code holds real x87 instructions with OS fixups
  (`WIN87EM`). Each step rounds to 64 bits and each `Double` store to 53; `Round` takes a half
  to the even neighbour. Port it with [gameframe-x87](../../crates/x87/README.md).
- **Under the emulator.** `dos_emu` leaves the x87 control word unset: Unicorn reports 0 but
  computes at 64 bits until an `fldcw`. Set the program's word before measuring code that
  loads one (`Trunc` does).
- **Data.** Strings are `ShortString`. The data segment is often stored iterated; read it with
  `readseg.py`, not from Ghidra's listing.
