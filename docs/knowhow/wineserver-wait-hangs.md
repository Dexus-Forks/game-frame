---
id: wineserver-wait-hangs
title: A run script hangs on `wineserver -w`
kind: failure
status: solved
runtime: [wine]
symptom: [hang]
tags: [wineserver, scripts, services]
commands: []
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-wine-macos.md]
seen: ['POD: Planet of Death (1997), Wine 11.18, 2026-10-23']
---

# A run script hangs on `wineserver -w`

## Symptoms

A script that starts the game and then waits with `wineserver -w` never returns, though the game
has quit.

## Cause

`wineserver -w` waits until every Wine process of the prefix has ended. Wine's own services can
stay up after the game exits (*observed* under Wine 11.18).

## What to do

Do not wait on the server. Wait on the game's own process, or poll for it by name. If you need a
clean state, then stop the prefix with `wineserver -k`. Check for leftovers before the next run
([a leftover process](single-instance-leftover.md)).
