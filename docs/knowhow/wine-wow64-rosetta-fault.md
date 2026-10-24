---
id: wine-wow64-rosetta-fault
title: A 32-bit game's system call faults in wow64cpu under Wine on Apple silicon
kind: failure
status: open
era: [late-1990s]
platform: [windows-9x, macos]
runtime: [wine, wow64, rosetta, apple-silicon]
symptom: [crash, hang]
tags: [wow64cpu, signal-stack, seh, threads]
signatures:
  - 'nested exception on signal stack'
  - 'Exception frame is not in stack limits'
  - 'wow64cpu(?:\.dll)?\s*\+\s*0x113[0-9a-f]'
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-wine-macos.md, docs/knowhow/wine-macos-suspendthread.md]
seen: ['POD: Planet of Death (1997), Wine 11.0 and 11.18, 2026-10-23', 'POD Gold PowerVR and ATI builds with stand-in card libraries, Wine 11.0, 2026-10-24']
---

# A 32-bit game's system call faults in wow64cpu under Wine on Apple silicon

## Symptoms

Some seconds into a run, a game thread dies. Under Wine 11.0 the log shows
`virtual_setup_exception nested exception on signal stack`, and the game freezes at 0% CPU.
Under Wine 11.18 it shows `Exception frame is not in stack limits` and a repeating fault at
`wow64cpu+0x1135`. The game then freezes at 100% CPU. The fault happens now and then, in any
thread that makes system calls, such as `user32` calls with a window handle or `ReleaseMutex` in
a sound mixer loop. One run in several gets further.

## Evidence

`WINEDEBUG=+seh` shows the fault with CS `0x0107`, the 32-bit code selector, at
`wow64cpu+0x1135`. Disassembled as 64-bit code that address is the system-call entry
(`xchg r14, rsp`). As 32-bit code it is `dec ecx; xchg esi, esp`, which puts the window handle
in ESP. The switch to 64-bit mode did not happen. The 64-bit entry ran as 32-bit code
(*measured* in POD). `atos` on the host address of Wine's own fault names
`_platform_memmove$VARIANT$Rosetta`.

*Inferred*: the fault is in Wine's 32-bit (WOW64) support under Rosetta. The game does not
cause it. This is not proven. A small 32-bit test program that makes the same calls from several
threads would show the fault without the game. Write that program before any report to Wine.

## Tried

- No sound (the game's driver question answered "Ignore"): the fault stays.
- No CD audio (`mcicda` disabled): the fault stays.
- A 1 MB stack reserve in the PE header instead of 64 KB: the fault stays.
- No `SuspendThread` or `ResumeThread` calls ([patched out](wine-macos-suspendthread.md)): the fault stays.
- Not reloading FS and GS in the Watcom runtime's handlers: the fault stays.
- Wine 11.18 (staging) in place of 11.0: the same fault, shown more clearly, and a freeze at 100% CPU.
- Sikarugir's CrossOver 23.7.1 engine: also WOW64, the fault stays.
- POD Gold's PowerVR and ATI builds, with stand-in card and DirectDraw libraries (Rust DLLs
  linked by zig): the same fault, `wow64cpu+0x1139`, read of `0x4ECD`, 32-bit SS shown as 0,
  every run at the same step of the main menu's second drawing. Without the stand-ins' OpenGL,
  without their vectored exception handler, without sound, and on Wine 11.18 (which shows
  `nested exception on signal stack`): the fault stays. A small 32-bit test program that loads
  the same libraries and draws through OpenGL from a second thread runs cleanly.

Not tried yet: a Linux x86-64 host, a virtual machine with a real Windows 95 or 98. The small
test program ran cleanly (above), so the fault needs something only the game does.
