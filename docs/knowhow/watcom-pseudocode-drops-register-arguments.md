---
id: watcom-pseudocode-drops-register-arguments
title: Ghidra's pseudocode of a Watcom function leaves out arguments and stores that the disassembly shows
kind: failure
status: solved
era: [early-1990s, late-1990s]
platform: [dos, windows-9x]
compiler: [watcom]
genre: []
symptom: [tool-error]
tags: [ghidra, watcall, register-calling-convention, pseudocode, disassembly, decompile, arguments]
tools: [ghidra-run, decomp_query, refs]
skills: [re-ghidra, re-trace]
docs: [docs/ABI.md, docs/working/ghidra.md, docs/knowhow/context-watcom.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# Ghidra's pseudocode of a Watcom function leaves out arguments and stores that the disassembly shows

## Symptoms

A function's pseudocode shows fewer arguments than its callers pass, or reads registers that
were never set. A value the port computes from the pseudocode is wrong, and nothing in the
pseudocode explains why. A string that the program obviously uses has no user in `decomp.c`.

In Hardwar the pseudocode of `crater_lights_place` (`0x44b9c8`) showed each light's colour and
radius being passed on, so the port drew the lights at their stored height of 0, below the
crater floors. The disassembly showed that the function copies the stored position to a vector
on the stack, writes the radius shifted right by 3 over its height, and passes that vector in a
register to the next call (*observed*).

## Cause

Watcom C passes the first four arguments in EAX, EDX, EBX and ECX (`__watcall`). Ghidra's
default convention for a 32-bit Windows program passes them on the stack. With the wrong
convention the decompiler does not see the register arguments, so it drops them and treats the
stores that fill them as dead. A pointer to a string loaded into EAX before a call is lost the
same way (*observed* in Hardwar and in POD).

## What to do

For any Watcom function the port depends on, read the disassembly and check every register set
before each call. The pseudocode is a guide to the shape of the function only.

```bash
./gameframe run ghidra-run GAME.EXE disasm analysis/notes/lights.asm 0x44b9c8        # the whole function
./gameframe run ghidra-run GAME.EXE disasm analysis/notes/span.asm 0x44b9c8-0x44ba40 # a range, in order
./gameframe run refs GAME.EXE 0x535f43 --functions analysis/export/GAME.EXE/functions.tsv   # users of a string
```

Set the calling convention of the functions you trace to `__watcall` in Ghidra, or give them
custom storage, and decompile again (`docs/working/ghidra.md`). Record the real arguments in
the topic document. Keep the `.asm` files in `analysis/`: they reproduce the program's code.

## Tried

- Trusting the pseudocode's argument list: the crater lights hung at height 0 until the
  disassembly was read.
