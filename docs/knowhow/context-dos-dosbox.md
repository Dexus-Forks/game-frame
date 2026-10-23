---
id: context-dos-dosbox
title: 'DOS games in DOSBox-X: what to know before the first run'
kind: context
status: context
era: [1980s, early-1990s]
platform: [dos]
runtime: [dosbox, unicorn]
hardware: [ports, sound-card, display, x86-cpu]
api: [dos-ints]
compiler: [turbo-pascal, borland, watcom, djgpp]
tags: [mz, ne, le, overlays, dpmi, cycles, packed-executable]
commands:
  - '(?:^|[;&|(]\s*|/)(?:dosbox|dosbox-x)(?=\s|$|")'
  - '\bdosbox-run\.sh\b'
tools: [dosinfo, dosbox-run, fileio_trace, dbx_control, dos_emu, dos_env, insns]
skills: [re-inventory, re-observe, re-emulate]
docs: [docs/DOS.md, docs/observe/running-the-original.md, docs/computers/ibm-pc.md]
seen: ['the Turbo Pascal case study (Tyrian 2000), 2026']
---

# DOS games in DOSBox-X: what to know before the first run

## What to know

A DOS game is one of four kinds of program. The kind decides which tools you use after it.
Run `./gameframe run dosinfo GAME.EXE` first. Then run the game in DOSBox-X through
`scripts/dosbox-run.sh`. Run it on a copy of its folder, with a fixed CPU speed, and log its
file reads. Use the debugger through the `dosbox` MCP server or `dbx_control.py`.

- **The kind of program.** `MZ` is real mode (Turbo Pascal, Turbo C, Microsoft C). `NE` is
  16-bit protected mode with a DPMI host (Borland Pascal 7, Borland C++). `LE` or `LX` is 32-bit
  code behind a DOS extender (Watcom with DOS/4GW, PMODE/W). `PE` is a Windows program
  ([docs/DOS.md](../DOS.md)).
- **Packed programs.** LZEXE, PKLITE, EXEPACK and DIET must be unpacked first. `dosinfo` names
  them. Unpack with UNP, or dump memory from DOSBox-X after the unpacker has run.
- **Overlays and appended data.** Data after the MZ load image is an overlay (Borland's starts
  with `FBOV`), a packed payload or debug symbols. Check for debug symbols before anything else.
- **Addresses.** Use Ghidra's numbering everywhere: segment `1000` for MZ, selectors `1000`,
  `1008` and on for NE. The DPMI host chooses a live selector. Map it through the `LDT`.
- **Speed.** Many games time themselves by the CPU or by timer interrupts. Fix the speed with
  `--cycles N` and use the same value for every run you compare.
- **Files.** `--fileio` logs every open, read, seek and write, and `fileio_trace.py` turns it
  into a map per file. The Pascal runtime reads a text file in 128-byte blocks. For such a
  file, read the code.
- **Hardware.** `insns GAME.EXE '^(in|out)'` lists the ports: `0x201` joystick, `0x40` to
  `0x43` timer, `0x220` to `0x22f` Sound Blaster, `0x388` AdLib. `'^int '` lists the BIOS and
  DOS calls.
- **Exit codes.** A batch file reads them with `IF ERRORLEVEL`. DOSBox-X's shell does not expand
  `%ERRORLEVEL%`.
- **One function at a time.** `dos_emu` runs a single routine for exact results. `dos_env` runs
  the whole program headless, with frames and memory for scripts.
