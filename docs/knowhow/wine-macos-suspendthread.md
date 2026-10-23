---
id: wine-macos-suspendthread
title: A game suspends its own thread with SuspendThread under Wine on macOS
kind: failure
status: workaround
era: [late-1990s]
platform: [windows-9x, macos]
runtime: [wine, rosetta]
api: [win32]
symptom: [hang]
tags: [suspendthread, resumethread, threads, signals]
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-wine-macos.md, docs/knowhow/wine-wow64-rosetta-fault.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A game suspends its own thread with SuspendThread under Wine on macOS

## Symptoms

The game pauses its own game thread with `SuspendThread` while its window is inactive, and at
start on some desktops. Under Wine on macOS this is one more place where a thread is stopped by
a signal while a 32-bit game runs under Rosetta.

## Cause

Wine on macOS suspends a thread by sending it a signal (*inferred* from Wine's design). Nobody has
measured whether this causes a failure by itself. In POD the calls were patched out during the
search for the [`wow64cpu` fault](wine-wow64-rosetta-fault.md). The fault stayed.

## What to do

If you suspect it, skip the calls. Patch each `SuspendThread` and `ResumeThread` call on the game
thread to pop its argument and return 0 (`add esp, 4; xor eax, eax` in place of the call). The
game then keeps running in the background instead of pausing. Record it as a Deviation.

## Tried

- Skipping every `SuspendThread` and `ResumeThread` call in POD: the game no longer pauses. The
  `wow64cpu` fault stays.
