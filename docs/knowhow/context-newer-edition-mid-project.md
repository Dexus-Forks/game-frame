---
id: context-newer-edition-mid-project
title: 'A newer edition of the game turns up mid-project'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
hardware: [cd-rom]
genre: [racing-sports]
tags: [versions, editions, gold, patch, reference, labels, re-record]
tools: [versiondiff, match_functions, compare_trees, discinfo, ghidra-export]
skills: [re-inventory, re-ghidra, re-verify]
docs: [docs/working/versions.md]
seen: ['POD: Planet of Death (1997), first edition and POD Gold 2.8, 2026-10-24']
---

# A newer edition of the game turns up mid-project

## What to know

Find every edition before deep work, and take the last one released as the reference by
default: it has the most fixes and improvements. When a newer edition turns up after the work
has started, move the project to it in order: record both editions, compare their data with
`versiondiff.py`, export the new binaries, match functions so the labels carry over, record the
ground truth again, verify the port again, and keep the old edition as documented differences
(`docs/working/versions.md`, "Moving to a newer version mid-project").

The POD case (*observed*, 2026-10-24): a month of work used the March 1997 first edition, a
European disc in four languages with 16 tracks and 8 cars. Then POD Gold 2.8 turned up, with
executables linked in October 1997, English as well, 32 tracks, 16 cars, Direct3D and PowerVR
builds, and official updates, fan files and era tools beside it. Gold has 423 data files
against 289, 136 of them new, and a track list whose format needs a variant reader.

What it changes:

- **Addresses.** Every finding that cites an address belongs to one build. Keep a map from old
  to new addresses (`labels/NEW/map.tsv`, from `match_functions.py`), so each finding can be
  found again in the newer build.
- **Ground truth.** Emulator measurements and recordings of whole runs were made on the old
  executables. Record them again on the reference, and keep the old ones as evidence of how the
  old edition behaved.
- **Patches.** Patches made to run the old executables check the old bytes. The newer edition
  needs its own patch list, and some of its patches may already be fixed in the game.
- **Readers.** Run every reader over the newer data. New content shows as new files; a changed
  format shows as a reader that fails.
- **Extras.** Updates, fan levels and era tools that came with an edition are a test set for
  every reader, writer and editor.
