---
id: file-formats
title: Game files and data formats
kind: context
status: context
platform: []
runtime: []
hardware: []
api: []
compiler: []
genre: []
symptom: []
tags: [file, files, format, formats, archive, container, resources, assets, save, data-file]
skills: [re-inventory, re-format]
docs: [docs/media/README.md, docs/media/files.md, docs/teams/data-format.md, docs/knowhow/writer-round-trip-every-file.md, docs/knowhow/lists-keep-records-after-the-count.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Game files and data formats

## What to know

Start with the file list, sizes, names, and signatures. Then trace which code opens each file
and how it uses the bytes. Extensions and likely layouts are clues. The reader in the game is
the evidence for the format.

Keep a decoder's bounds explicit. Check counts, offsets, lengths, byte order, and any data after
the last counted record. Before editing a format, write it back unchanged and compare every byte.
Then run that check over every file of the type before testing edited values in the game.

The media guides cover file triage and structure. The linked cases show why round-trip checks and
the game's own record count matter.
