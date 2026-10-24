---
id: wine-missing-card-library
title: 'A 3D-card build will not start under Wine: no ati3dcif.dll, S3DTKW.dll or glide.dll'
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [3d-card]
api: [ati3dcif, s3d, glide]
symptom: [wont-start]
tags: [per-card-build, missing-dll, probe-dll]
signatures:
  - '(?i)library (?:ati3dcif|s3dtkw|glide2?x?)\.dll .*not found'
  - '(?i)import_dll .*(?:ati3dcif|s3dtkw|glide)'
tools: [peinfo]
skills: [re-trace, re-observe]
docs: [docs/hardware/card-apis.md, docs/knowhow/context-3d-card-builds.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A 3D-card build will not start under Wine: no ati3dcif.dll, S3DTKW.dll or glide.dll

## Symptoms

The ATI, S3 or 3dfx build of a game exits at once under Wine, or Wine reports that the card's
library, which the EXE needs, is not found.

## Cause

Wine has no ATI3DCIF, S3D Toolkit or Glide: they were the card vendors' own DLLs, installed
with their drivers (*observed*: no such DLL in Wine 10).

## Evidence

The EXE's import table names the DLL (`peinfo GAME.EXE --imports`).

## What to do

Write the library: export exactly the imported names (decorated `__stdcall` for ATI3DCIF and
Glide, plain `__cdecl` for the S3D Toolkit), and start with a **probe** version that logs
every call and dumps the first records it is given (vertices as floats and hex, surface
structures as words). One run settles the layouts from the game's own data, without the SDK
headers. Read each call's success value at its call site: they differ (ATI3DCIF 0; the S3D
Toolkit's `InitLib` 0 but `CreateRenderer` and every table function 1). Then draw what the
game gives the chip, in software into its own surfaces, or with a modern API. POD's three are
in `wrapper/` (`ati3dcif`, `s3dtkw`, `glide`).
