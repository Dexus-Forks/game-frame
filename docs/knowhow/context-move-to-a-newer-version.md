---
id: context-move-to-a-newer-version
title: Moving a project's labels and references to a newer version of the game
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
genre: [racing-sports]
tags: [version, label-map, match, relocations, rebuild, reference, port]
tools: [ghidra-export, ghidra-run, decomp_query, shot, memrec]
skills: [re-ghidra, re-trace, re-observe, re-verify, re-port]
docs: [docs/working/versions.md, docs/working/ghidra.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Moving a project's labels and references to a newer version of the game

## What to know

A project that started on one release and moves to a later one (POD's first edition of March
1997 to POD Gold 2.8 of October 1997) has to carry three things across: the names of
functions and globals, the patches that make the game run, and the recorded reference runs. In
POD's move (*measured*):

- **Export both builds the same way and fill the gaps in both.** Ghidra left 31% of each
  build's code outside functions ([missed functions](decompile-missed-pointer-functions.md)).
  Match before filling, and the functions the port depends on most (pointer-only ones) are
  missing on both sides.
- **Expect a rebuilt compiler, not just changed code.** Gold's code section moved and was
  renamed, jump targets are padded, registers and instruction order differ. Only 545 of 2,258
  matched functions kept the same instructions. Compare instructions with relocated words
  (from `.reloc`), call displacements and padding masked; then compare pseudocode; then what a
  function uses (constants, strings, mapped globals and callees). Only the last says whether
  the logic changed ([compiler noise](rebuilt-build-every-function-differs.md)).
- **Match by references when shape fails.** The race loop, lap logic and physics step were
  rewritten past shape matching, but each refers to the same mapped globals and callees: a
  pass that pairs functions by those sets matched them.
- **Hand-check what is reached only through pointers.** Controllers stored in per-object
  pointer slots paired up only by reading the setter in both builds.
- **Re-derive every patch.** The same fix sits at another address, may be gone (the Cyrix
  probe), or may need a second site (a flag the new build sets in code:
  [flag reset in code](patched-flag-reset-in-code.md)). New checks can stop the game silently
  ([unknown CPU](cpu-check-quits-on-unknown-cpu.md)).
- **Re-record references on the new build.** Data files changed too (Gold's tracks keep their
  geometry but have new race paths), so old recordings are no longer ground truth.
- **Keep readers for both, tests for both.** Point tests at the newer folder by default and
  give each count per edition; files of the newer build keep records after their counts
  ([records after the count](lists-keep-records-after-the-count.md)).

The tools POD used are in `run/versions` (match, what changed, labels) and `run/ghidra`
(gaps).
