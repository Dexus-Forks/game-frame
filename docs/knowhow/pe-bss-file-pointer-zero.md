---
id: pe-bss-file-pointer-zero
title: Globals in .bss hold code under the emulator, because the section's file pointer is 0
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
hardware: [x86-cpu]
symptom: [tool-error, wrong-result]
tags: [pe, section, bss, pointertorawdata, sizeofrawdata, emulator, unicorn, globals]
tools: [pe32_emu]
skills: [re-emulate]
docs: [docs/knowhow/pe-virtualsize-zero.md, docs/knowhow/context-watcom.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# Globals in .bss hold code under the emulator, because the section's file pointer is 0

## Symptoms

An emulated routine that reads a global in `.bss` finds a value that looks like code. A set-up
routine returns at once because its "already done" flag is not 0, and leaves its state unset.
In Hardwar, `fpu_setup` (`0x4af9a0`) returned straight away and left the x87 control word at 0:
its flag at `0x59a690` read `0x848bc289`, the bytes of an instruction.

## Cause

Watcom's linker marks a section with no file data by setting its file pointer
(`PointerToRawData`) to 0, while `SizeOfRawData` still gives its size (*observed* in
`HARDWARW.EXE`: `.bss` at `0x580000`, raw size `0x28a00`, file pointer 0). The Windows loader
gives such a section zeros. A loader that copies `SizeOfRawData` bytes from the file pointer
copies the start of the file, the MZ header and the first code, into the section. `pefile`'s
`get_memory_mapped_image` does this, and so did GameFrame's `pe32_emu` until 2026-10-24.

## Evidence

The flag's value matched the bytes at file offset `0x59a690 - 0x580000` from the start of the
file. Zeroing `0x580000`–`0x5a8a00` before the first call made `fpu_setup` run and set the
control word (*measured*, with Hardwar's own maths measurement script).

## What to do

Zero every section whose file pointer is 0 before running anything. `pe32_emu` does this when
it loads an image, and `tests/run.sh` checks it on a copy of the sample DLL with one section
patched this way. A tool that maps a PE by hand must do the same. The related case of a
`VirtualSize` of 0 is [pe-virtualsize-zero](pe-virtualsize-zero.md).
