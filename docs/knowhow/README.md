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
| [16-bit Windows (Windows 3.x) programs: what to know before the first run](context-windows-3x.md) | context |
| [A changed value in a data file loads but changes nothing in the game](context-mod-value-replaced-at-run-time.md) | context |
| [A data editor agents and tests can drive: one edit type for the window and the command line](context-editor-one-edit-type.md) | context |
| [Before modding a format: a writer that gives every file back byte for byte](writer-round-trip-every-file.md) | context |
| [Before moving geometry: find the bounds and normals stored with it](context-geometry-derived-data.md) | context |
| [CD-era games often look in their install folder before the CD: a mod can be a file, with no patch](context-mods-install-folder-first.md) | context |
| [DOS games in DOSBox-X: what to know before the first run](context-dos-dosbox.md) | context |
| [Games with a build per 3D card: Glide, ATI3DCIF and the S3D Toolkit](context-3d-card-builds.md) | context |
| [Keep a capture of every error dialog and crash, as it happens](context-keep-every-dialog.md) | context |
| [Measure a function that walks the game's data structures under the emulator](emulate-function-on-game-structures.md) | context |
| [Port a game's rules before its physics, driven by a recorded run](context-port-rules-before-physics.md) | context |
| [Record a whole run of the original from its memory as the reference for a port](record-a-run-from-memory.md) | context |
| [Running the original in an emulated PC when Wine on the host fails](context-emulated-pc.md) | context |
| [Start a recorder when the next race begins instead of driving the menus](wait-for-the-next-race.md) | context |
| [The game's clock runs at a slightly different rate from real time](game-clock-not-real-time.md) | context |
| [Watcom C/C++ programs: calling convention, runtime and linker quirks](context-watcom.md) | context |
| [Windows 95 and DirectDraw games: what to know before the first run](context-windows-9x-directdraw.md) | context |
| [Wine on macOS with Apple silicon: what to know before running a game there](context-wine-macos.md) | context |
| [Writing a Glide 2 replacement for an early Glide game](context-glide2-wrapper.md) | context |
| [A 32-bit game's system call faults in wow64cpu under Wine on Apple silicon](wine-wow64-rosetta-fault.md) | open |
| [Attaching winedbg to the running game kills it](winedbg-attach-kills.md) | dead-end |
| [A game suspends its own thread with SuspendThread under Wine on macOS](wine-macos-suspendthread.md) | workaround |
| [Clicks and keys from the agent do not reach a Wine window on macOS](wine-input-without-accessibility.md) | workaround |
| [DirectInput ignores keys, because a window that is not foreground cannot acquire the device](directinput-not-foreground.md) | workaround |
| [Scripted menu clicks land one time in three in a game with its own cursor](directinput-cursor-click-timing.md) | workaround |
| ["S3DTK Error : SetState" from a stand-in S3D Toolkit](s3dtk-error-setstate.md) | solved |
| [`pkill -f` or `pgrep -f` run over SSH matches its own command line](vm-ssh-pkill-matches-itself.md) | solved |
| [A 16-bit game on a 32-bit surface shows a doubled picture, then wrong colours](directdraw-16bit-back-surface.md) | solved |
| [A 3D view draws every level upside down and nothing looks wrong](axes-measure-in-the-running-game.md) | solved |
| [A 3D-card build will not start under Wine: no ati3dcif.dll, S3DTKW.dll or glide.dll](wine-missing-card-library.md) | solved |
| [A DirectDraw game asks for an impossible mode, because its mode table overflowed](directdraw-mode-table-overflow.md) | solved |
| [A DirectDraw game runs in a window only on a 16-bit desktop, else goes fullscreen](directdraw-windowed-16bit-desktop.md) | solved |
| [A driver run with sudo sends no keys: "Can't open display"](sudo-drops-display.md) | solved |
| [A function has no callers, or no function exists, because Ghidra missed code reached through pointers](decompile-missed-pointer-functions.md) | solved |
| [A helper script named like a standard module breaks an import, such as dis.py and capstone](python-script-shadows-stdlib.md) | solved |
| [A line-following driver stalls or wedges at the same place every lap](racing-line-must-be-a-closed-lap.md) | solved |
| [A path from an installer's ini file lacks its last backslash, so files are not found](installer-ini-path-separator.md) | solved |
| [A PE section has a VirtualSize of 0, and a tool skips or misreads it](pe-virtualsize-zero.md) | solved |
| [A port diverges because a trace said "only this function writes that table" and was wrong](trace-claims-need-an-address-scan.md) | solved |
| [A ported fixed-point function differs from the original in a few cases out of thousands](x87-magic-add-rounding.md) | solved |
| [A privileged instruction under Wine, from a Windows 95 port probe](wine-privileged-instruction.md) | solved |
| [A reference recording was made with modded files loaded](mods-loaded-in-a-reference-recording.md) | solved |
| [A replay matches the recording at the end but not at some samples, one sample early or late](replay-sample-straddles-frames.md) | solved |
| [A run in a shared test VM dies: "X connection to :2 broken"](shared-vm-pkill-display.md) | solved |
| [A run script hangs on `wineserver -w`](wineserver-wait-hangs.md) | solved |
| [An editor's save changes bytes nobody edited (fixed point through floats)](editor-float-view-changes-bytes.md) | solved |
| [Every new run exits at once, because a leftover game process holds its mutex](single-instance-leftover.md) | solved |
| [Keys reach a game under Wine on Xvfb only with a window manager running](xvfb-needs-window-manager.md) | solved |
| [macOS drops DYLD_ variables through /bin/sh, so a Wine build cannot load its libraries](macos-dyld-stripped.md) | solved |
| [Porting more of the original made a replay agree less with the recording](replay-stand-in-for-unported-logic.md) | solved |
| [S3 ViRGE builds draw 15-bit pixels: their 16-bit surfaces are 5-5-5](s3-virge-15-bit-surfaces.md) | solved |
| [SSH to a VM fails with "Too many authentication failures"](ssh-too-many-auth-failures.md) | solved |
| [The game asks for its CD under Wine on Linux, though the disc's folder is mapped with the right label](wine-linux-cd-drive.md) | solved |
| [The game asks for its CD while the disc is there, because some file failed to open](insert-cd-file-open-failed.md) | solved |
| [The player's car drives itself after a scripted driver was stopped](input-driver-keys-held-after-kill.md) | solved |
| [Times printed by the game differ by a hundredth from the same values printed by a tool](x87-truncating-time-format.md) | solved |

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
- `winevdm`: otvdm (winevdm), which runs 16-bit Windows programs on 64-bit Windows and Wine. Also: otvdm.
- `glide-wrapper`: programs that translate Glide calls to a modern API: dgVoodoo 2, nGlide, OpenGlide. Also: dgvoodoo, dgvoodoo2, nglide, openglide.
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
- `ati3dcif`: ATI's 3D Rage interface. Also: ati3dcif.dll.
- `s3d`: the S3D Toolkit for S3 ViRGE. Also: s3d toolkit, s3dtkw.
- `dos-ints`: DOS and BIOS interrupts. Also: int 21h, int 10h, int 33h.
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
