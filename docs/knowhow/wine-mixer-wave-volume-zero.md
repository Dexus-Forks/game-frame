---
id: wine-mixer-wave-volume-zero
title: A game's sound effects play at volume 0 under Wine, because it reads its volume from the mixer
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x, macos]
runtime: [wine]
hardware: [sound-card, cd-rom]
api: [winmm]
symptom: [no-sound]
tags: [mixer, volume, mixerGetLineInfo, iat-hook]
signatures:
  - '(?i)mixerGetLineInfo[AW]? .*80000003'
  - '(?i)dwComponentType: 0x100[58]'
tools: []
skills: [re-trace, re-observe]
docs: [docs/hardware/sound-card-apis.md, wrappers/maxi94/README.md]
seen: ['POD Gold (1997), PODMAX64.EXE, Wine 11.0 on macOS, 2026-10-24']
---

# A game's sound effects play at volume 0 under Wine, because it reads its volume from the mixer

## Symptoms

The game opens its wave outputs and writes sound to them (`WINEDEBUG=+winmm` shows
`waveOutWrite`), but nothing is heard, or its volume setting reads 0 and cannot be raised. The
trace shows the game asking the mixer for a source line by component type
(`mixerGetLineInfoA` with flags `0x80000003`, `dwComponentType: 0x1008` for WAVEOUT or
`0x1005` for COMPACTDISC).

## Cause

Some games of 1997 keep their effect and music volumes in the WinMM mixer's source lines, as
the sound card's driver offered them. Wine's mixer has a destination line for the speakers and
answers the WAVEOUT query with it, and that line's volume control reads 0. It has no
COMPACTDISC line (*measured*, Wine 11.0 on macOS: the game's first read of the wave line's
volume gives 0, before it sets anything). POD's Maxi Sound 64 builds take that 0 as the
effect volume, write it back, and give their wave streams volume 0 (*observed*,
`PODMAX64.EXE` `FUN_004b9710`, `FUN_004b9ad4`, `FUN_004bbe0c`).

## Evidence

Logging the game's mixer calls from a stand-in DLL that hooks them in the import table:
`mixerGetLineControlsA` finds Wine's volume control (ID 2) on the line, and
`mixerGetControlDetailsA` reads its value as 0. `WINEDEBUG=+winmm` shows the queries but not
the values.

## What to do

Answer the game's two lines from a DLL the game loads, as the card's driver did. Hook
`mixerGetLineInfoA`, `mixerGetLineControlsA`, `mixerGetControlDetailsA` and
`mixerSetControlDetails` in the game's import table (`shim::iat`). For a WAVEOUT or
COMPACTDISC query, give a source line with one volume control (0 to `0xffff`, starting at
full), keep the value the game sets and read it back. Pass every other query to Wine.
`wrappers/maxi94` does this for POD (`maxi94/src/mixer.rs`, `mixer_dll.rs`). A game without a
card library of its own needs the hooks in another stand-in it loads, such as `ddraw.dll`.

## Tried

- Answering only when Wine's call fails: Wine answers the WAVEOUT query, so the game kept
  Wine's line and its volume of 0.
- Reading the component type after Wine's failed `mixerGetLineInfoA`: Wine had written its own
  fields back into the structure, so every query looked like the first. Read the query before
  calling Wine.
