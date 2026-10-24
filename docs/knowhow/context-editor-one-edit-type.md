---
id: context-editor-one-edit-type
title: 'A data editor agents and tests can drive: one edit type for the window and the command line'
kind: context
status: context
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
tags: [editor, modding, level-editor, undo, cli, agents, viewer]
commands: []
tools: []
skills: [re-browse, re-verify]
docs: [docs/working/viewer.md, docs/knowhow/writer-round-trip-every-file.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A data editor agents and tests can drive: one edit type for the window and the command line

## What to know

Make every change one value of one type (an `Edit` enum), and make both the window and a
command-line mode produce those values. Then an agent, a test or a script can do everything a
person can, and the tests cover the same code the window runs. Apply an edit by writing over the
stored fields at offsets the reader recorded, never by re-serialising the decoded view. Keep, per
edit, the bytes it replaced: undo and redo are then exact. After each edit, decode again from
the bytes, so the view always shows what will be saved.

The checks that make such an editor trustworthy:

- A save with no edits equals the original file, byte for byte, on every file of the kind.
- Each edit kind has a test that saves, reads back, and finds exactly the change.
- An edit and its undo give the original bytes again.
- One edit of each kind loads in the original game and shows there, against a stock run made
  the same way (`docs/modding.md`).

Keep the editor behind a build option when the project separates the exact core from additions.

## Evidence

POD's track editor (`apps/viewer`, cargo feature `editor`): `pod-viewer edit TRACK --move-prop N
DX DY DZ --out FILE` and the window's panel make the same `Edit` values; a no-edit save equals
the original on all 36 tracks; eight tests; three edits seen in the game (*measured*,
2026-10-24).

## What to do

1. Add stream offsets to the reader's records for every field the editor will change.
2. Write the enum, an `apply` that turns an edit into (offset, bytes) patches, and the undo stack.
3. Write the command-line parser to the same enum, with a `--list` mode to find indices.
4. Write the tests above, then the window.
