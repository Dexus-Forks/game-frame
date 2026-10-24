---
id: s3dtk-error-setstate
title: '"S3DTK Error : SetState" from a stand-in S3D Toolkit'
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [3d-card]
api: [s3d]
symptom: [crash, wont-start]
tags: [return-value]
signatures:
  - 'S3DTK Error : \w+'
  - 'S3D Virge Library Fail'
tools: []
skills: [re-trace]
docs: [docs/hardware/card-apis.md, docs/knowhow/wine-missing-card-library.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# "S3DTK Error : SetState" from a stand-in S3D Toolkit

## Symptoms

The game stops with "S3D Virge Library Fail ! S3DTK Error : SetState" (or `CreateRenderer`),
or crashes in its clean-up right after `S3DTK_GetState`.

## Cause

The S3D Toolkit's functions disagree on success: `S3DTK_InitLib` returns 0 (`S3DTK_OK`), but
`S3DTK_CreateRenderer` and every function in the table it fills (`SetState`, `GetState`,
`TriangleSet`) return 1 (TRUE) (*observed*: `cmp eax, 1` after each call in `WPODS3D.EXE`).

## Evidence

The disassembly right after each call; the error strings next to the branches.

## What to do

Return 0 from `S3DTK_InitLib` and 1 from everything else. For `S3DTK_GetState` of
`S3DTK_VIDEOMEMORYADDRESS`, answer 0 and make `S3DTK_LinearToPhysical` the identity: the
offsets the game derives (`sfOffset`) are then plain linear addresses into its surfaces.
