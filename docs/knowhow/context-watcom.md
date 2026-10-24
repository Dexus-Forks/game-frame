---
id: context-watcom
title: 'Watcom C/C++ programs: calling convention, runtime and linker quirks'
kind: context
status: context
era: [early-1990s, late-1990s]
platform: [dos, windows-9x]
compiler: [watcom]
tags: [watcall, dos4gw, register-calling-convention, linker]
tools: [peinfo, insns, exportcov]
skills: [re-inventory, re-ghidra, re-trace]
docs: [docs/ABI.md, docs/COMPILED-CODE.md, docs/DOS.md]
seen: ['POD: Planet of Death (1997), 2026-10-23', 'Hardwar (1998), 2026-10-24']
---

# Watcom C/C++ programs: calling convention, runtime and linker quirks

## What to know

Watcom C/C++ built many DOS and Windows 95 games. Its default calling convention passes the
first four arguments in registers. Its runtime handles exceptions and segment registers in its
own way. Its linker writes PE headers that some tools misread. Set Ghidra's calling convention
before you read the pseudocode. Otherwise every function shows the wrong arguments.

- **Recognising it.** The binary holds strings such as `WATCOM C/C++32 Run-Time system`,
  `Open Watcom` or `DOS/4G`. Another sign is a DOS program with a DOS/4GW or PMODE/W stub.
- **Calling convention (`__watcall`).** Arguments go in EAX, EDX, EBX and ECX. The rest go on
  the stack, and the callee pops them. Until the convention is set, Ghidra shows such functions
  with unused parameters and with reads of uninitialised registers ([docs/ABI.md](../ABI.md)).
- **Runtime.** The startup code and exception handlers save and reload FS and GS. In POD,
  removing these reloads changed nothing under Wine (*measured*).
- **Linker.** Sections can have a VirtualSize of 0 ([VirtualSize 0](pe-virtualsize-zero.md)).
- **Pseudocode.** With the wrong convention, Ghidra's pseudocode drops register arguments
  and the stores that fill them. Read the disassembly of every function the port depends on
  ([register arguments dropped](watcom-pseudocode-drops-register-arguments.md)).
- **Function starts.** Watcom code often opens with a row of register pushes, which the
  analysis misses. In POD and Hardwar a quarter to a third of the code lay in no function
  ([missed functions](decompile-missed-pointer-functions.md), `exportcov`).
- **Section names.** `BEGTEXT`, `DGROUP` and `.bss` together are the Watcom linker's names
  (*observed* in Hardwar).
- **The FPU.** Look for the control word the game loads; Hardwar loads `0x0c7f`
  ([x87 control word](context-x87-control-word.md)).
