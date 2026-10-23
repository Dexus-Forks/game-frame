---
id: pe-virtualsize-zero
title: A PE section has a VirtualSize of 0, and a tool skips or misreads it
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
symptom: [tool-error]
tags: [pe, section, virtualsize, sizeofrawdata, patching]
signatures:
  - 'virtual ?size\W{0,3}(?:of )?(?:0x)?0\b'
tools: [peinfo, vtable, relocs]
skills: [re-inventory]
docs: [docs/knowhow/context-watcom.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A PE section has a VirtualSize of 0, and a tool skips or misreads it

## Symptoms

A tool that maps or patches a PE file finds no code, finds no room in a section, or computes a
size of 0. `peinfo` shows a section with size `0x0`.

## Cause

Some linkers, Watcom's among them, leave `VirtualSize` (`Misc_VirtualSize`) at 0 and set only
`SizeOfRawData` (*observed* in POD's `WINPOD.EXE`). Windows then maps `SizeOfRawData` bytes. A
tool that trusts `VirtualSize` sees an empty section.

## What to do

Where `VirtualSize` is 0, use `SizeOfRawData`. GameFrame's `peinfo`, `vtable` and `relocs` do
this. `tests/run.sh` checks `peinfo` and `vtable` on the sample game. `peinfo` marks the section `(raw size: VirtualSize 0)`. A
patch script that looks for room in a section must do the same.
