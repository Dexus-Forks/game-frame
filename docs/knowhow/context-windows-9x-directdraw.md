---
id: context-windows-9x-directdraw
title: 'Windows 95 and DirectDraw games: what to know before the first run'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, virtual-machine]
hardware: [display, ports, cd-rom, x86-cpu, mmx]
api: [win32, directdraw, directinput, directsound, mci]
tags: [directx, cd-check, ini, registry, single-instance, indeo]
commands:
  - '(?i)\bddraw\b'
tools: [insns, peinfo, pe32_emu]
skills: [re-inventory, re-observe]
docs: [docs/hardware/graphics-2d.md, docs/hardware/card-apis.md, docs/hardware/simd.md, docs/observe/running-the-original.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Windows 95 and DirectDraw games: what to know before the first run

## What to know

A game made for Windows 95 and DirectX 1 to 5 assumed a 1990s machine. It expected port I/O to
be allowed, a 16-bit desktop, a dozen display modes, a CD in the drive and the settings its
installer wrote. Each assumption fails in its own way on Windows 10 or under Wine. Make the
checks below before the first run. Most failures then need only a one-line fix.

Before the first run:

- **Privileged instructions.** Windows 95 let a program use `in`, `out`, `cli` and `sti`.
  Run `./gameframe run insns GAME.EXE '^(in|out|ins|outs|cli|sti)'` and patch every probe
  ([privileged instruction](wine-privileged-instruction.md)).
- **CPU checks.** Games use `cpuid`, `rdtsc` and Cyrix register probes to choose a build or to
  refuse to run.
  `insns GAME.EXE '^(cpuid|rdtsc)'` finds them. A game may have an MMX build
  ([docs/hardware/simd.md](../hardware/simd.md)).
- **Display modes.** The game lists modes with `EnumDisplayModes` into a fixed table, and may
  overflow it ([mode table](directdraw-mode-table-overflow.md)). It may allow a window only on a
  16-bit desktop ([windowed mode](directdraw-windowed-16bit-desktop.md)). It may draw 16-bit
  pixels in RGB565 or RGB555 ([back surface](directdraw-16bit-back-surface.md)).
- **The CD.** The game finds its disc by drive type and volume label. It reads data from the
  disc.
  Every failed file open may show as "insert the CD" ([insert CD](insert-cd-file-open-failed.md)).
  Copy the disc's files and give the drive the same label.
- **Settings.** The installer wrote an ini file in the Windows folder or registry keys. Read the
  installer's script or strings. Write exactly the same values
  ([ini paths](installer-ini-path-separator.md)).
- **One copy.** A named mutex allows only one copy to run. A leftover process blocks every new
  run ([leftover process](single-instance-leftover.md)).
- **Video.** Intro videos often need the Indeo codec from the disc's install folder. Leave them
  out at first.
- **Sound and CD audio.** DirectSound and `mcicda` can fail on a modern machine. Most games
  offer to continue without them. Accept that until the picture works.

The places to run it, best first for observing, are Wine on Linux or macOS (trace channels,
scriptable), a virtual machine with a real Windows 98 (exact behaviour, harder to observe) and
GameFrame's `pe32_emu` for single routines with no system calls.

Wine's trace channels for this era are `+ddraw` (surfaces and modes), `+dinput`, `+dsound`,
`+file` (every file open) and `+seh` (exceptions with registers). Use `+relay` with
`RelayFromInclude=GAME` to see only the game's own calls.
