---
id: context-windows-3x
title: '16-bit Windows (Windows 3.x) programs: what to know before the first run'
kind: context
status: context
era: [early-1990s]
platform: [windows-3x]
runtime: [winevdm, dosbox, 86box, virtual-machine, unicorn]
api: [win32]
compiler: [borland, turbo-pascal, msvc]
tags: [ne, win16, selectors, gdi, wing]
commands:
  - '(?i)(?:^|[;&|(]\s*|/)(?:otvdm|winevdm)(?:\.exe)?(?=\s|$|")'
tools: [dosinfo, ne, nedis, readseg, dos_emu]
skills: [re-inventory, re-ghidra, re-emulate, re-observe]
docs: [docs/DOS.md, docs/computers/ibm-pc.md]
seen: ['GameFrame guides, 2026']
---

# 16-bit Windows (Windows 3.x) programs: what to know before the first run

## What to know

A Windows 3.x game is an NE executable with 16-bit code in segments. 64-bit Windows cannot run
it. Read it with GameFrame's NE tools and Ghidra's New Executable loader. Run it in Wine,
in otvdm (winevdm) on 64-bit Windows, or in Windows 3.1 inside DOSBox-X. Emulate single routines
with `dos_emu` in protected mode.

Use 86Box when the result depends on a specific PC model, BIOS, expansion bus or sound card.
DOSBox-X remains useful for Windows 3.1 runs where its debugger helps trace behavior. Record the
machine and card settings with every capture ([86Box guide](../observe/86box.md)).

- **Recognising it.** `dosinfo GAME.EXE` says `NE`. A Windows NE program imports from `KERNEL`,
  `USER` and `GDI`, by ordinal more often than by name. A DOS program in the same format imports
  from a DPMI runtime (`RTM.EXE`, `DPMI16BI.OVL`) instead.
- **Addresses.** Ghidra numbers the segments as selectors `1000`, `1008` and on. Keep that
  numbering in every document. The system that runs the program chooses a live selector
  ([docs/DOS.md](../DOS.md)).
- **Reading code.** `ne.py` lists the segments, entries and relocations. `nedis.py`
  disassembles a segment. `readseg.py` reads bytes and constants by `seg:off`. Far calls
  between segments go through relocations. The relocation table holds a call's target. Do not
  look for the target in the bytes of the call.
- **Running it.** Wine runs many Win16 programs directly. On 64-bit Windows, otvdm (winevdm)
  runs them. Windows 3.1 inside DOSBox-X gives a convenient system with DOSBox-X's debugger;
  86Box gives a configurable period PC when machine or card details matter.
- **Drawing.** Win16 games drew through GDI, or later through WinG and DIB sections. Palette
  messages (`WM_PALETTECHANGED`, `WM_QUERYNEWPALETTE`) matter on an 8-bit desktop. Modern
  systems emulate the 8-bit desktop.
