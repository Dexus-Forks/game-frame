---
id: winedbg-gdb-server
title: 'Debugging a game under Wine or CrossOver through winedbg''s GDB server: what works on Apple silicon'
kind: failure
status: workaround
era: [2010s]
platform: [windows-nt, macos]
runtime: [wine, rosetta, apple-silicon]
symptom: [hang, crash]
tags: [winedbg, gdb, lldb, debugger, breakpoint, int3, vcont, crossover]
signatures:
  - 'target remote localhost:\d+'
commands:
  - 'winedbg(?:\.exe)?["]?\s+--gdb'
tools: [winegdb, gdbremote]
skills: [re-observe]
docs: [docs/observe/debuggers.md, docs/knowhow/winedbg-attach-kills.md, docs/knowhow/context-wine-macos.md]
seen: ['Banished (2014), 64-bit, CrossOver 26.3 on an M3 Max, 2026-10-27']
---

# Debugging a game under Wine or CrossOver through winedbg's GDB server: what works on Apple silicon

## Symptoms

`winedbg --gdb --no-start 0xPID` attaches and prints `target remote localhost:PORT`. A
debugger connects, reads registers and memory, and then:

- after a continue, the game stays frozen: no frames, almost no CPU (*measured*: 37% before,
  0.9% while "running");
- a breakpoint is never hit, even in code that runs every frame;
- after a hardware breakpoint, the game ends some seconds later (*observed*, three times).

## Cause

Three things in winedbg's GDB server, each *measured* with packets sent by hand:

- **A bare `c` resumes one thread.** It continues the thread last chosen with `Hc` or `Hg`,
  often the thread winedbg made to stop the game. `Hc-1` then `c`, or `vCont;c`, resumed every
  thread (the game's CPU went back to 41 to 43%).
- **No software breakpoints.** `Z0` gets an empty reply, which means "not supported". `Z1`
  (hardware) and `Z2` (write watchpoint) get `OK`, and `M` (memory write) works.
- **Hardware breakpoints do nothing under Rosetta.** A `Z1` on `People::update`, set in every
  thread, was not hit in 15 seconds while every thread ran, and the game ended soon after.
  The same function was hit at once with an int3 byte.

lldb sends `Z0`, so it falls back on its own to writing int3 bytes, but a scripted lldb that is
killed while the game runs leaves them in the code.

## What to do

Use `tools/live/winegdb.py`. It sends `vCont;c`, writes the int3 byte itself, moves `rip` back
at a hit, and puts the original byte back on every exit: a hit, a timeout, an error, Ctrl-C.

```
./gameframe run winegdb procs --bottle Steam
./gameframe run winegdb read  --bottle Steam --name GAME.exe 0x140000000 64
./gameframe run winegdb regs  --bottle Steam --name GAME.exe
./gameframe run winegdb break --bottle Steam --name GAME.exe 0x140118340 --hits 3 --dump rsp:8 --dump rcx:0x60
```

- The game loads at its preferred base under Wine, so Ghidra's addresses work as they are
  (`read ... 0x140000000 2` shows `MZ`).
- Attaching stops the game until the tool detaches. Short commands are best.
- Do not kill winedbg while it is attached: that ends the game. After a detach it exits by
  itself.
- winedbg's module list (`qXfer:libraries`) has empty names and the Mac's own addresses. Use
  the import table to find a DLL function: read the 8-byte slot from the game's IAT.
- A Steam game may show its launcher first, in the same executable. Nothing of the game's frame
  loop runs there. Wait until a town or level is loaded.
- `watch` uses hardware watchpoints. Under Rosetta they never fired, so they are no use there.

## Evidence

On Banished's 64-bit build, `break 0x140118340 --hits 3` stopped three times in one thread.
The return address was `0x1400c978a`, `rsi` counted `0xd`, `0xe`, `0xf`, and `rbx` stepped by
`0xe38`: one citizen each call. The function's first bytes read `48 8b` again afterwards, and the
game ran on.

## Tried

- lldb `gdb-remote PORT`: registers, threads and memory worked, and `process detach` left the
  game running. lldb showed no images. A batch lldb killed during `process continue` left
  winedbg stuck on a closed connection with a breakpoint in the code. Ending winedbg ended the
  game.
- Plain `c`: the game froze (above).
- Hardware breakpoints (`Z1`), first in one thread, then in every thread: never hit, and the
  game ended after each try.
- int3 bytes through `M`, with `vCont;c`: hit at once, every time.
