---
id: patched-flag-reset-in-code
title: A flag patched in the data is set back in code by a newer build
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
api: [directdraw]
symptom: [wrong-mode]
tags: [patch, fullscreen, windowed, version, initialiser]
tools: []
skills: [re-trace]
docs: [docs/working/versions.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A flag patched in the data is set back in code by a newer build

## Symptoms

The same patch as on the older build, at the mapped address, has no effect: POD Gold still
asks DirectDraw for exclusive fullscreen (`SetCooperativeLevel` flags `0x51`) and fails with
`lpDD::SetDisplayMode->DDERR_UNSUPPORTED`.

## Cause

The first edition kept its fullscreen flag only as initialised data (`0x4f7260`). Gold has the
flag at `0x4eac84` and also stores 1 into it at start-up (`0x4a147d` in `FUN_004a1460`)
(*observed*).

## Evidence

A `+ddraw` trace showed the flags; listing every write to the mapped global found the store.

## What to do

After carrying a data patch to a new build, list every write to that global (the export's
decompile, or a byte search for the address in `mov` instructions) and patch the ones that
undo it.
