---
id: macos-dyld-stripped
title: macOS drops DYLD_ variables through /bin/sh, so a Wine build cannot load its libraries
kind: failure
status: solved
platform: [macos]
runtime: [wine]
symptom: [wont-start]
tags: [dyld, sip, dylib, library-path, shell]
signatures:
  - 'Library not loaded:.*\.dylib'
  - 'dyld\[\d+\]:.*(?:not loaded|image not found)'
tools: []
skills: [re-setup]
docs: [docs/knowhow/context-wine-macos.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# macOS drops DYLD_ variables through /bin/sh, so a Wine build cannot load its libraries

## Symptoms

A Wine build from a wrapper (CrossOver, Sikarugir and others) fails at once with
a dyld message that `libinotify.0.dylib`, or another library, is not loaded.

## Cause

Such builds expect their wrapper to set `DYLD_FALLBACK_LIBRARY_PATH`. System Integrity
Protection removes every `DYLD_*` variable when a protected program starts. `/bin/sh` and
`/bin/bash` are protected. A variable exported before a script starts is therefore gone inside
it.

## What to do

Set the variable inside the script, on the line that starts Wine:
`DYLD_FALLBACK_LIBRARY_PATH="$ENGINE/lib:/usr/lib" "$ENGINE/bin/wine" GAME.EXE`.
