---
id: single-instance-leftover
title: Every new run exits at once, because a leftover game process holds its mutex
kind: failure
status: solved
platform: [windows-9x]
runtime: [wine]
api: [win32]
symptom: [wont-start]
tags: [mutex, single-instance, leftover-process, createmutex]
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-wine-macos.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Every new run exits at once, because a leftover game process holds its mutex

## Symptoms

The game starts and quits at once, with no message. A trace shows the DLLs detach right after
they attach.

## Cause

The game makes a named mutex to allow one copy. An earlier run that hung or crashed still holds
it. Each new copy sees the mutex and quits. `wineserver -k` did not stop the old process
(*observed*).

## What to do

Before each run, list the game's processes by name and kill them (`pkill -f GAME.EXE`, then check
with `pgrep -fl GAME.EXE`). Make the run script do it.
