---
id: writer-round-trip-every-file
title: 'Before modding a format: a writer that gives every file back byte for byte'
kind: context
status: context
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
tags: [modding, writer, round-trip, formats, container]
commands: []
tools: [validate]
skills: [re-format, re-verify]
docs: [docs/working/viewer.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Before modding a format: a writer that gives every file back byte for byte

## What to know

A reader that decodes every file is not yet a writer. Readers built for a viewer convert as
they go (palette colours to RGB, fixed point to floats, sections sorted) and skip what they do
not need (string pools, "unused" tails, the bytes after a name's terminator). A writer from
that view gives files the game may still load but that cannot be checked. The check that
catches every loss is: decode each file of the player's copy, write it back, compare all
bytes. Only then change one value and load it in the game.

What POD's writers needed (*measured* on every file of the disc, 2026-10-23):

- Keep the stored form beside the friendly view: raw palettes, each fixed name field whole,
  the unused tail of the last block.
- Keep pools and tables as stored and append to them. POD's text pool is not in key order:
  rebuilt from the keys it gives other bytes. A pool may also end without a terminator; add
  one before appending.
- Some leftovers follow a rule and can be made again: POD's font files end in a buffer where
  each byte is the byte one block earlier XORed with the key.
- Write from the reader's own walk, not from a signature scan. A scan that guesses a block's
  page size found a "block" in one track that runs past the end.
- A container whose entry offsets count raw bytes (checksum words included) needs its entry
  table recomputed when a part grows. Test it by growing a part (one more texture page) and
  reading every entry to its end.
- Checksummed containers must be encoded again after any change; a wrong sum stops the game.

## Evidence

`crates/pod-formats/tests/writers.rs` in the POD project: images (146), text (12), stills (6),
texture blocks (371 pages), fonts (4), effects (3) and tracks (36) write back identical, and a
changed copy reads back changed. Four changed files then loaded in the original game.

## What to do

For each format: add the round-trip test over every file first (`validate.py DECODER.py GLOB
--write`, with an `encode(result)` beside `decode`), then fix the reader until it
keeps every byte, then add the one edit the mod needs and a test that reads it back. Run the
tests in release builds: a nearest-colour search over every picture is slow in debug.
