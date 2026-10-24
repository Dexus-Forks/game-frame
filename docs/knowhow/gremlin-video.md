---
id: gremlin-video
title: 'Gremlin Digital Video (`.GDV`) in a 1998 Windows game'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [gremlin, video]
tools: []
skills: [re-format]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Gremlin Digital Video (`.GDV`) in a 1998 Windows game

## What to know

*2026-10-24, `crates/gdv`, `docs/formats/gdv.md`.*

- **Recognise it** by the u32 `0x29111994` at the start, a 24-byte header, a 768-byte 6-bit
  palette, then per frame the sound bytes and an 8-byte picture head starting `0x1305`.
  Walking the chunks with the sound size from the header must land exactly on the file's end;
  that check alone validates the container on every file.
- **Find the player** by the magic as a constant (`== 0x29111994` in the pseudocode) and
  `0x1305`; in Hardwar it is hand-written assembly at `0x51c023`, so read loop shapes, not
  argument lists.
- **The picture codes are FFmpeg's `gdv` decoder's**, with two differences that matter for
  exactness: the 4,096-byte preamble before the picture holds `i >> 3` (each value eight
  times, twice), not FFmpeg's pattern; and a picture may end by running past the end of the
  picture rather than with the end code `0xfff`, writing into the memory after it (the
  player checks the end only between codes). Allow writes past the end into a slack area,
  and count the pixels that read from it: in Hardwar, 2 pixels in 33,781 frames.
- **Word copies are safe to model byte by byte**: the player copies an odd byte then 16-bit
  words, which differs from a byte copy only at distance 1, and it turns distance 1 into a
  fill with the last byte.
- **The rate 21,168 Hz** in the header is 22,050 × 12 / 12.5: it is passed to DirectSound as
  is, with 1,764 samples a frame at 12 frames a second.
- **The DPCM sums are not clamped**: the player stores the low 16 bits of a running i32 sum;
  they reset at the picture flag `0x40`.
- **Test without the game** with a bit writer that reserves 16-bit words where the reader
  will refill (after any read that leaves 16 or fewer bits) and interleaves plain bytes.
