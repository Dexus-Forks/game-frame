# DOS sample

A tiny 16-bit program for testing GameFrame's DOS tools. It is our own code: no game is
involved.

- `sample.s`: routines in the style of a Borland Pascal program (FAR, Pascal calling
  convention, DS = data segment): a Turbo-Pascal-style random generator, a loop that draws
  numbers, a checksum over a far pointer, a selector fixup, an imported call (`KERNEL.3`) and
  a DOS call (`INT 21h`), and `main_loop`: eight frames of a tiny main loop for
  `tools/emu/fragments.py` (DPMI blocks, the VGA display, a file read each frame).
- `wait_keys`: installs a keyboard handler on INT 9 that reads port `0x60` into a key table
  and a log and acknowledges the PIC, as a game's handler does, then waits for scan codes
  (dos_env's keyboard test).
- `make_exe.py OUT`: assembles it with `zig cc` (LLVM, `.code16`) and links it with Python into
  `sample.exe` (NE, 16-bit protected mode), `sample-td.exe` (the same with Turbo Debugger
  symbols appended) and `sample-mz.exe` (real-mode MZ). The data segment holds Pascal strings,
  two of them back to back. The symbols include a variable and a type with anonymous records
  inside, and an imported procedure (`host_call`) whose segment number is past the program's.

`tests/run.sh` builds it and checks dosinfo, readseg, tdinfo, dos_emu, Ghidra and DOSBox-X
against it. The first random draw from the built-in seed `0x1234` with range 100 is 23: the
emulator, the program's exit code under DOSBox-X and the debugger channel must all agree.

Pitfall found while writing it: LLVM's Intel syntax assembles `retf 2` in `.code16` as a 32-bit
return (`66 CA 02 00`). The source uses AT&T syntax (`lretw $2`).
