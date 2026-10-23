---
id: installer-ini-path-separator
title: A path from an installer's ini file lacks its last backslash, so files are not found
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
api: [win32]
symptom: [file-not-found]
tags: [ini, installer, path]
tools: []
skills: [re-observe]
docs: [docs/knowhow/insert-cd-file-open-failed.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A path from an installer's ini file lacks its last backslash, so files are not found

## Symptoms

The file trace shows paths with two names run together, such as `C:\GAMEData\...` for
`C:\GAME\Data\...`.

## Cause

The game appends its sub-folders to a folder read from an ini file (often in `C:\WINDOWS`, put
there by the installer) with no separator. The installer always wrote the folder with a
trailing backslash. A hand-made ini does not have it (*measured* in POD).

## What to do

Write every ini value exactly as the installer would. A folder ends in `\`. When the installer
does not run, read its script or its strings to learn what it writes. Set every key it sets.
