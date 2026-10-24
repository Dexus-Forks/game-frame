---
id: registers-left-over
title: 'A register the routine never sets is still an input'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [left, over, registers]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A register the routine never sets is still an input

## What to know

*2026-10-24, `FactionUpdate` 0x41a0fc, `faction_daily_target` 0x41a028, the Moth roll-out
0x41ae2f; Hardwar's `economy_world.py`.*

Watcom's register convention keeps every register a callee does not return in, so a value
a caller leaves in a register survives into the next call. Two economy routines take such a
value as an input without meaning to:

- **A counter that starts where the caller left it.** `0x41a028` counts the factories it may
  choose among in EDX, but pushes EDX and never clears it. Its caller last loaded EDX with
  the address of a local (`lea edx, [esp + 0x1c]`, for `msg_read`), so the count starts at a
  stack address, and the draw that follows (`rng_range(count - 1)`) wraps in 32 bits. The
  pseudocode shows a plain counter; only the disassembly shows the missing `xor edx, edx`.
  The port takes the address as an input (`count_start`), and the emulator varies the stack
  pointer per case so the test covers it. In a real run the value depends on where Windows
  put the main thread's stack: something to read from the running game.
- **An output pointer that aliases a counter.** The Moth roll-out passes the address of its
  own free-bay counter as the routine's EBX, and `0x419d20` writes the bay it took there.
  So the counter becomes the bay index and the loop stops early. Reading the callee alone
  shows an output; reading the caller alone shows a counter. Follow every pointer argument
  back to what it points at.

How to catch them: when a routine reads a register before writing it (EDX, EBX, ECX after
the pushes), find what the caller last put there, even across calls, since the calls keep
it. Under the emulator, run the routine from its real caller, or vary the register, rather
than calling it with clean registers: a clean EDX of 0 would have made `0x41a028` look
correct.
