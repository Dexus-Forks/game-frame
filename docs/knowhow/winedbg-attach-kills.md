---
id: winedbg-attach-kills
title: Attaching winedbg to the running game kills it
kind: failure
status: dead-end
platform: [macos]
runtime: [wine, rosetta]
symptom: [crash]
tags: [winedbg, debugger, attach]
tools: []
skills: [re-observe]
docs: [docs/observe/debuggers.md, docs/knowhow/context-wine-macos.md]
seen: ['POD: Planet of Death (1997), Wine 11.18, 2026-10-23']
---

# Attaching winedbg to the running game kills it

## Symptoms

`winedbg --pid PID` attaches, and the game dies.

## Tried

- Attaching to the running game under Wine 11.18 on macOS: the game died, twice (*observed*).
- winedbg breakpoints did work earlier in the same project: they measured POD's mode-table
  value. How that session was started is not recorded.
- Wine's trace channels (`+seh`, `+ddraw`, `+file`, `+relay`) in place of a debugger: these
  worked every time. Use them instead ([Wine on macOS](context-wine-macos.md)).
