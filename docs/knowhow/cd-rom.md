---
id: cd-rom
title: CD-ROM data, disc checks and audio
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine]
hardware: [cd-rom]
api: [dos-ints, mci, win32]
compiler: []
genre: []
symptom: []
tags: [cd, disc, cd-audio, redbook, volume-label, streaming, mscdex]
skills: [re-inventory, re-observe, re-trace]
docs: [docs/media/discs.md, docs/hardware/other.md, docs/knowhow/context-mods-install-folder-first.md, docs/knowhow/insert-cd-file-open-failed.md, docs/knowhow/wine-linux-cd-drive.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# CD-ROM data, disc checks and audio

## What to know

Separate the game's disc check from the data and audio it reads. Trace the failed file opens,
volume-label checks, and media calls. DOS games may use MSCDEX for files or CD audio. Windows
games may use ordinary file calls and MCI for audio tracks.

An ISO image can keep the files while dropping audio tracks. Games that stream from a disc may
also depend on when data arrives. Record which files and tracks the game uses, which drive and
label it expects, and whether timing affects the game. This distinguishes a missing-file problem
from a missing-audio or streaming problem.

The disc guide covers image formats and file systems. The linked cases show how a missing file or
drive mapping can appear as an insert-CD message.
