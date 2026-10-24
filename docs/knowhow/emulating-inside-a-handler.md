---
id: emulating-inside-a-handler
title: 'Emulating a range of code inside a big message handler'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [emulating, handler, inside]
tools: []
skills: [re-emulate, re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Emulating a range of code inside a big message handler

## What to know

*2026-10-24, Hardwar's `flight.py`.*

VhSlotControl (`0x47b1e0`) is one 12.9 KB function with a `0x1f0`-byte frame; the flight
step is a stretch of its `0x104` case, not a function. GameFrame's `pe32_emu.call` sets ESP
just under its stack top, so `[ESP+0x1ec]` lies past the mapped stack and the run fails.

What works: allocate a block on the emulator's heap, point ESP into the middle of it, write
the handler's frame words the range reads (here `+0x1ec` the slot index, `+0x1c8` a pointer
to `dt_ms`, `+0x1bc` the slot's user area), set the registers the range expects (`EBP` the
vehicle, `EDI` a pointer to `dt_fix`) and run `uc.emu_start(start, end)`. Calls inside the
range push below ESP, inside the block. Read which frame words a range uses with a grep for
`esp +` over its disassembly first.

A routine with side effects outside the state under test (here the event routine `0x432328`,
sound and HUD) can be made a `RET` by writing `0xc3` over its first byte and hooking the
address to record EAX: `uc.hook_add(UC_HOOK_CODE, fn, begin=addr, end=addr)`.

Linear disassembly of the whole code section (`objdump -d` on a copy of the EXE with the
sections' virtual sizes set to their raw sizes and the data directories cleared) desyncs at
jump tables, such as the ten words at `0x47b1b8`, but resynchronises within a few
instructions; check a function's first instruction by address before reading it.
