---
id: editor-float-view-changes-bytes
title: An editor's save changes bytes nobody edited (fixed point through floats)
kind: failure
status: solved
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
symptom: [tool-error]
tags: [editor, writer, fixed-point, 16.16, float, round-trip, precision]
signatures: []
tools: []
skills: [re-format, re-verify]
docs: [docs/knowhow/writer-round-trip-every-file.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# An editor's save changes bytes nobody edited (fixed point through floats)

## Symptoms

A file saved from the decoded view differs from the original in fields that were not edited,
usually in the last bits of positions far from the origin.

## Cause

The game stores positions as 16.16 fixed point; the viewer holds them as 32-bit floats, which
keep 24 bits. POD's tracks reach about 2,000 units, which needs 27 bits plus a sign, so a float
cannot hold every stored value (*inferred* from the ranges; the editor was written to avoid it).

## What to do

Never write fields back from the float view. Record each field's offset when reading; an edit
adds a rounded fixed-point delta to the stored integer and writes it at that offset. Untouched
fields keep their bytes. Test that a save with no edits is the original on every file.
