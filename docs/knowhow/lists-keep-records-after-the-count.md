---
id: lists-keep-records-after-the-count
title: A newer build's list files hold records after their count that the game never reads
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
genre: [racing-sports]
tags: [format, count, round-trip, version, writer]
tools: [catalog]
skills: [re-format]
docs: [docs/working/mods.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A newer build's list files hold records after their count that the game never reads

## What to know

POD Gold's `CIRCUITS.BIN` says 32 and holds 35 rows; its `VOITURE2.BIN` says 8 and holds 23
records (*measured*). Gold's readers read the counted records only (*observed*:
`FUN_00471f24`, `FUN_00472020`). The extra records name tracks and cars that are not on the
disc ("Night", web cars): left over from the tools that wrote the files.

A reader that checks "file size = header + count × record" fails on such files; one that reads
only the count cannot write the file back byte for byte. Read the count, read every whole
record in the file, return the counted ones as the game sees them, and keep the rest for the
writer.
