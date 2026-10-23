# Know-how

These entries record what happened when a game would not run or a tool failed. They also
record what was done about it. Each entry is one failure mode or one piece of context. An agent
finds them with `./gameframe advise`. A hook names them when a command's output matches. The
design is in [Know-how: failure modes and context](../working/knowhow.md).

```
./gameframe advise "A privileged instruction was executed at 004a88cd"
wine GAME.EXE 2>&1 | tail -100 | ./gameframe advise -
./gameframe advise directdraw wine
./gameframe advise --list
./gameframe advise --entry wine-privileged-instruction
```

An entry has one of five statuses.

| Status | Meaning |
|---|---|
| `solved` | The cause is known and the fix works. |
| `workaround` | Something avoids the problem, but the cause stays, or the fix is partial. |
| `dead-end` | A path that was tried and given up. The entry says what was tried, so nobody tries it again without a new idea. |
| `open` | Not solved yet. The entry lists every attempt. Try something new and add it. |
| `context` | Not a failure. What to know when work turns to an era, a platform, a runtime or a kind of hardware. |

## Entries

| Entry | Status |
|---|---|
| [Windows 95 and DirectDraw games](context-windows-9x-directdraw.md) | context |
| [Wine on macOS with Apple silicon](context-wine-macos.md) | context |
| [Watcom C/C++ programs](context-watcom.md) | context |
| [A privileged instruction under Wine: port I/O](wine-privileged-instruction.md) | solved |
| [A DirectDraw mode table overflows](directdraw-mode-table-overflow.md) | solved |
| [A window only on a 16-bit desktop](directdraw-windowed-16bit-desktop.md) | solved |
| [16-bit pixels on a 32-bit surface](directdraw-16bit-back-surface.md) | solved |
| ["Insert the CD" with the disc present](insert-cd-file-open-failed.md) | solved |
| [An installer's ini path without its last backslash](installer-ini-path-separator.md) | solved |
| [A PE section with a VirtualSize of 0](pe-virtualsize-zero.md) | solved |
| [`SuspendThread` under Wine on macOS](wine-macos-suspendthread.md) | workaround |
| [DirectInput: the window is not foreground](directinput-not-foreground.md) | workaround |
| [Clicks and keys do not reach a Wine window](wine-input-without-accessibility.md) | workaround |
| [macOS drops `DYLD_*` through `/bin/sh`](macos-dyld-stripped.md) | solved |
| [`wineserver -w` never returns](wineserver-wait-hangs.md) | solved |
| [A leftover game process holds the single-instance mutex](single-instance-leftover.md) | solved |
| [Attaching winedbg kills the game](winedbg-attach-kills.md) | dead-end |
| [A system call faults in `wow64cpu` under Rosetta](wine-wow64-rosetta-fault.md) | open |

A project keeps the entries for its own game in its own `docs/knowhow/`, in the same format.
`./gameframe advise` reads both. An entry that holds for other games too moves here, with a
test or a sample where it changes a tool.

## Taxonomy

Each entry names values from these facets. `./gameframe advise --check` rejects a value that is
not listed here. Add a value here first, with its aliases. An alias is another way a person or
a log names the value. `advise` matches an alias as a whole word or phrase.

### era

- `1980s`: 8-bit home computers, early PC and DOS. Also: 8-bit era.
- `early-1990s`: DOS with VGA, real mode and the first DOS extenders. Also: early 90s.
- `late-1990s`: Windows 95 and 98, DirectX 1 to 7, the first 3D cards. Also: late 90s, 1997, 1998, 1999.
- `2000s`: Windows XP, DirectX 8 and 9, programmable shaders. Also: windows xp era.

### platform

- `dos`: MS-DOS programs, real mode or a DOS extender. Also: ms-dos, dos4gw, dos/4gw.
- `windows-3x`: 16-bit Windows 3.0 and 3.1, NE executables. Also: windows 3.1, win16.
- `windows-9x`: Windows 95, 98 and ME. Also: windows 95, windows 98, win95, win98, win9x.
- `windows-nt`: Windows NT, 2000, XP and later. Also: windows xp, winnt.
- `macos`: Apple's macOS as the machine that runs the tools. Also: mac, darwin, os x.
- `linux`: Linux as the machine that runs the tools.

### runtime

- `wine`: Wine, CrossOver and other Wine builds, on macOS or Linux. Also: wineserver, winedbg, crossover, wine-staging, sikarugir.
- `rosetta`: Rosetta 2, Apple's x86-64 translator. Also: rosetta 2.
- `wow64`: 32-bit Windows code on a 64-bit Wine or Windows. Also: wow64cpu, syswow64.
- `apple-silicon`: Macs with an Apple ARM CPU. Also: arm64, m1, m2, m3, m4.
- `dosbox`: DOSBox and DOSBox-X. Also: dosbox-x.
- `virtual-machine`: a real old Windows in a PC emulator or virtual machine. Also: 86box, pcem, virtualbox, qemu, vmware.
- `unicorn`: GameFrame's emulators on Unicorn (pe_emu, pe32_emu, dos_emu). Also: pe_emu, pe32_emu, dos_emu.

### hardware

- `x86-cpu`: the CPU, its identification and its privileged instructions. Also: cpuid, rdtsc.
- `cyrix`: Cyrix CPUs and their configuration registers. Also: cyrix 6x86, mediagx.
- `mmx`: MMX and other SIMD code. Also: 3dnow, sse.
- `display`: the screen, its modes, depths and pixel formats. Also: display mode, video mode, bpp, pixel format.
- `cd-rom`: the CD-ROM drive, CD checks and CD audio. Also: cd, cdrom, cd audio, redbook.
- `ports`: hardware I/O ports. Also: port i/o, io port.
- `3d-card`: 1990s 3D cards: 3dfx, ATI Rage, S3 ViRGE. Also: 3dfx, voodoo, rage, virge.
- `sound-card`: Sound Blaster and other sound hardware. Also: sound blaster, adlib, opl2, opl3.

### api

- `win32`: Windows' own API: kernel32, user32, gdi32. Also: kernel32, user32, gdi32.
- `directdraw`: DirectDraw, 2D surfaces and display modes. Also: ddraw, ddraw.dll, idirectdraw.
- `directinput`: DirectInput keyboard, mouse and joystick. Also: dinput, dinput.dll.
- `directsound`: DirectSound. Also: dsound.
- `direct3d`: Direct3D. Also: d3d, d3dim.
- `glide`: 3dfx Glide. Also: glide2x, glide3x.
- `mci`: the Media Control Interface: CD audio and video. Also: mcicda, mciavi.

### compiler

- `watcom`: Watcom C/C++ and its linker. Also: open watcom, wlink, wcc386.
- `msvc`: Microsoft Visual C++. Also: visual c++, visual studio.
- `borland`: Borland C++ and Delphi. Also: bcc, delphi.
- `turbo-pascal`: Turbo Pascal and Borland Pascal.
- `djgpp`: DJGPP, GCC for DOS.

### genre

The values are the guides in [docs/games/](../games/README.md).

- `racing-sports`: racing and sports games. Also: racing, racer.
- `first-person`: first-person shooters. Also: fps, shooter.
- `strategy`: real-time and turn-based strategy. Also: rts.
- `rpg`: role-playing games. Also: role-playing.
- `adventure`: adventure games. Also: point and click.
- `arcade-action`: arcade and action games. Also: platformer.
- `vehicle-sims`: flight and vehicle simulators. Also: flight sim.
- `management`: management and simulation games. Also: tycoon.
- `puzzle-board`: puzzle and board games. Also: puzzle.

### symptom

- `crash`: the program stops with an exception or a crash dialog. Also: exception, segfault, access violation.
- `hang`: the program or a tool stops responding, or never returns. Also: freeze, frozen, hangs, stuck.
- `wont-start`: the program exits or fails before its first screen. Also: does not start, exits at once.
- `wrong-picture`: the picture is wrong: doubled, wrong colours, wrong size. Also: wrong colours, garbled.
- `wrong-mode`: the program picks the wrong window or display mode. Also: fullscreen, windowed.
- `no-input`: keys, clicks or the joystick do not reach the program. Also: input ignored, keys ignored.
- `file-not-found`: the program cannot find or open a file. Also: missing file, insert cd.
- `tool-error`: a GameFrame tool or another tool fails on the program's files.
