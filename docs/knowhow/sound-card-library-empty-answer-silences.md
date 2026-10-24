---
id: sound-card-library-empty-answer-silences
title: A stand-in sound-card library starts the game, but the game turns its own sound off
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [sound-card]
api: [card-sound-api, winmm]
symptom: [no-sound]
tags: [stand-in, probe-dll, memory-map, wave-streams]
signatures:
  - '(?i)mpuMessageAc .*acknowledged'
tools: [peinfo]
skills: [re-trace, re-observe]
docs: [docs/hardware/sound-card-apis.md, wrappers/maxi94/README.md, docs/hardware/replacement-libraries.md]
seen: ['POD Gold (1997), PODMAX64.EXE with wrappers/maxi94, Wine 11 on macOS, 2026-10-24']
---

# A stand-in sound-card library starts the game, but the game turns its own sound off

## Symptoms

A game built for one sound card imports that card's library. A silent stand-in for the
library, which answers every call with success, lets the game start. The game then plays no
sound at all, even its plain wave output. Wine's trace shows no `waveOutWrite` calls.

## Cause

The game asks the card a question and turns its sound off when the answer is empty. POD
Gold's Maxi Sound 64 builds open two wave outputs, and read the card's memory map before and
after each to learn which of the card's wave streams it took. With no new stream in the map,
the game sets its "sound on" flag to 0 and closes both outputs (*observed*, `PODMAX64.EXE`
`FUN_004ba304`, `FUN_004b9cd4`; *measured* under Wine with the silent stand-in).

## Evidence

The card library's call sites in the EXE (`peinfo GAME.EXE --imports` gives the thunks; a
search for `call` to each thunk gives the callers). The decompiled caller shows what it does
with each answer. In POD, `UpLoad` fills a table that a scan reads for entries of kind `0x20`.
An empty table makes both scans answer -1, and the next test turns the sound off.

## What to do

Read what the game does with each answer before writing the stand-in's answers. Answer the
question as the card would. For the Maxi Sound 64, list one wave stream in the map for each
wave output the game has open. The stand-in learns which are open by hooking the game's
`waveOutOpen` and `waveOutClose` in its import table (`shim::iat`). `wrappers/maxi94` does
this. Then log the first buffer of each stream that is not silence, to show the sound reaches
the output.

Check the volume path next. POD's 64 builds also read their effect volume from the WinMM
mixer, which under Wine reads 0 ([the mixer's wave line](wine-mixer-wave-volume-zero.md)).

## Tried

- A silent stand-in with every call answered "success" and an empty map: the game starts and
  stays silent.
- The map filled from the open wave outputs: both outputs found (streams 0 and 1), sound
  written, at volume 0 until the mixer's lines were answered too.
