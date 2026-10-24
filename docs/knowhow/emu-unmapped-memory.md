---
id: emu-unmapped-memory
title: An emulated function stops with an unmapped memory read, write or fetch
kind: failure
status: solved
runtime: [unicorn]
symptom: [tool-error, crash]
tags: [unicorn, pe_emu, pe32_emu, dos_emu, globals, teb, tls, crt]
signatures:
  - 'UC_ERR_(?:READ|WRITE|FETCH)_UNMAPPED'
  - 'Invalid memory (?:read|write|fetch)'
  - 'bad memory access at 0x[0-9a-f]+'
tools: [pe_emu, pe32_emu, dos_emu, call_trace]
skills: [re-emulate]
docs: [docs/knowhow/emulate-function-on-game-structures.md]
seen: ['Un-Banished and POD: Planet of Death, GameFrame case studies']
---

# An emulated function stops with an unmapped memory read, write or fetch

## Symptoms

A call under GameFrame's emulators stops with `UC_ERR_READ_UNMAPPED`, `UC_ERR_WRITE_UNMAPPED`
or `UC_ERR_FETCH_UNMAPPED` ("Invalid memory read"), and `pe32_emu` prints `bad memory access
at ADDRESS from EIP`.

## Cause

The function reached memory the emulator never set up. The usual reasons, from the case
studies (*observed*):

- a pointer field of an input structure that the test did not fill, or a global that the
  game's loader or `DllMain` sets, which the emulator does not run;
- the C runtime's thread data: `sprintf` and similar library calls read FS or GS, and
  `pe32_emu` maps no TEB (POD's Watcom `sprintf`);
- thread-local variables: `pe_emu` maps a TEB at GS, but `__declspec(thread)` also needs the
  TLS slot array at `TEB+0x58`;
- a fetch at a small address: a function pointer in the input that was left 0.

## What to do

Read the faulting address and the instruction at EIP. A small address is a field you did not
fill: find its offset in the instruction and fill it (`emu.alloc` and `write`). A global's
address is a global the loader sets: write it from the original's own initialisation or a
memory dump. For a library call that needs the C runtime, stop at the call with
`call(..., until=CALL_SITE)` and read its arguments from `stack`. For an import, give a stub
(`stubs=`).
