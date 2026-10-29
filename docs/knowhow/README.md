# Know-how

These entries record what happened when a game would not run or a tool failed. They also
record what was done about it. Each entry is one failure mode or one piece of context. An agent
finds them with `./gameframe advise`. A hook names them when a command's output matches. The
design is in [Know-how: failure modes and context](../working/knowhow.md).

```
./gameframe advise "A privileged instruction was executed at 004a88cd"
wine GAME.EXE 2>&1 | tail -100 | ./gameframe advise -
./gameframe advise directdraw wine
./gameframe advise sound-card
./gameframe advise x86-cpu
./gameframe advise cd-rom
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
| [A port's floating-point result differs in the last bit from the original's x87 code](x87-f64-last-bit.md) | solved |
| ["S3DTK Error : SetState" from a stand-in S3D Toolkit](s3dtk-error-setstate.md) | solved |
| [16-bit Windows (Windows 3.x) programs: what to know before the first run](context-windows-3x.md) | context |
| [8-bit textures with no palette in the file: infer it from the 16-bit copy of the same pictures](context-palette-from-16bit-twin.md) | context |
| [`pkill -f` or `pgrep -f` run over SSH matches its own command line](vm-ssh-pkill-matches-itself.md) | solved |
| [A 16-bit game on a 32-bit surface shows a doubled picture, then wrong colours](directdraw-16bit-back-surface.md) | solved |
| [A 32-bit game's system call faults in wow64cpu under Wine on Apple silicon](wine-wow64-rosetta-fault.md) | open |
| [A 3D view draws every level upside down and nothing looks wrong](axes-measure-in-the-running-game.md) | solved |
| [A 3D-card build gives the card only part of the frame: measure before upscaling](card-draws-part-of-frame.md) | context |
| [A 3D-card build will not start under Wine: no ati3dcif.dll, S3DTKW.dll or glide.dll](wine-missing-card-library.md) | solved |
| [A capture's last rows and columns can repeat their neighbours](capture-edge-rows-repeat.md) | solved |
| [A changed value in a data file loads but changes nothing in the game](context-mod-value-replaced-at-run-time.md) | context |
| [A data editor agents and tests can drive: one edit type for the window and the command line](context-editor-one-edit-type.md) | context |
| [A decompiled switch drops values: run the function for every case instead](jump-table-measure-not-read.md) | solved |
| [A DirectDraw game asks for an impossible mode, because its mode table overflowed](directdraw-mode-table-overflow.md) | solved |
| [A DirectDraw game runs in a window only on a 16-bit desktop, else goes fullscreen](directdraw-windowed-16bit-desktop.md) | solved |
| [A disk cleanup deletes a project's target/, the copy of the game, as Cargo build output](game-folder-deleted-as-build-output.md) | solved |
| [A driver run with sudo sends no keys: "Can't open display"](sudo-drops-display.md) | solved |
| [A flag patched in the data is set back in code by a newer build](patched-flag-reset-in-code.md) | solved |
| [A function has no callers, or no function exists, because Ghidra missed code reached through pointers](decompile-missed-pointer-functions.md) | solved |
| [A game suspends its own thread with SuspendThread under Wine on macOS](wine-macos-suspendthread.md) | workaround |
| [A game that loads its own x87 control word, such as 0x0c7f (single precision, truncation)](context-x87-control-word.md) | context |
| [A game's sound effects play at volume 0 under Wine, because it reads its volume from the mixer](wine-mixer-wave-volume-zero.md) | solved |
| [A GameFrame tool fails with "No module named unicorn", pefile or capstone](tool-without-venv.md) | solved |
| [A helper script named like a standard module breaks imports, as dis.py broke capstone](python-script-shadows-stdlib.md) | solved |
| [A level drawn from inside shows walls in front of the road](painter-order-big-faces.md) | solved |
| [A line-following driver stalls or wedges at the same place every lap](racing-line-must-be-a-closed-lap.md) | solved |
| [A newer build's list files hold records after their count that the game never reads](lists-keep-records-after-the-count.md) | context |
| [A newer edition of the game turns up mid-project](context-newer-edition-mid-project.md) | context |
| [A path from an installer's ini file lacks its last backslash, so files are not found](installer-ini-path-separator.md) | solved |
| [A PE section has a VirtualSize of 0, and a tool skips or misreads it](pe-virtualsize-zero.md) | solved |
| [A per-step recorder fits in the code section's mapped slack](recorder-hook-in-section-slack.md) | context |
| [A port diverges because a trace said "only this function writes that table" and was wrong](trace-claims-need-an-address-scan.md) | solved |
| [A ported fixed-point function differs from the original in a few cases out of thousands](x87-magic-add-rounding.md) | solved |
| [A privileged instruction under Wine, from a Windows 95 port probe](wine-privileged-instruction.md) | solved |
| [A project's ./gameframe runs a second clone or another project's copy of GameFrame, and a tool is missing](project-uses-another-projects-gameframe.md) | solved |
| [A reference recording was made with modded files loaded](mods-loaded-in-a-reference-recording.md) | solved |
| [A register the routine never sets is still an input](registers-left-over.md) | context |
| [A replay drifts from the original because the game steps by the measured frame time](variable-frame-step-replay.md) | solved |
| [A replay matches the recording at the end but not at some samples, one sample early or late](replay-sample-straddles-frames.md) | solved |
| [A replayed race plays several times too fast in the app](replay-plays-too-fast.md) | solved |
| [A run in a shared test VM dies: "X connection to :2 broken"](shared-vm-pkill-display.md) | solved |
| [A run script hangs on `wineserver -w`](wineserver-wait-hangs.md) | solved |
| [A save made in flight ends the game when continued](in-flight-save-ends-on-continue.md) | context |
| [A screen that closes "on a key" closes at once when the key is still held](help-closes-while-key-held.md) | solved |
| [A script driving a JSON-lines probe hangs when an answer is not an object](probe-reader-non-object-answer.md) | solved |
| [A scripted edit to source code fails to match after a formatter has run](scripted-edits-after-formatter.md) | solved |
| [A scripted key tap the game never saw](capture-key-tap-missed.md) | workaround |
| [A session in a git worktree finds no game, no Ghidra export and no Python environment](worktree-ignored-folders-missing.md) | solved |
| [A software mixer packs both stereo gains into one double](x87-one-multiply-two-channels.md) | context |
| [A stand-in sound-card library starts the game, but the game turns its own sound off](sound-card-library-empty-answer-silences.md) | solved |
| [A story's "scene data" can live in its actors' AI plans](scene-data-in-plans.md) | context |
| [A table the game builds at start-up comes out wrong under the emulator](unicorn-fpu-starts-single-precision.md) | solved |
| [A third of HARDWARW.EXE was not in any Ghidra function](ghidra-function-recovery.md) | context |
| [A tools/llm run is refused, though the key works and the credits are there](api-credits-refused.md) | solved |
| [A value that comes from a register nobody set there: read the disassembly](registers-left-by-callees.md) | context |
| [A wait loop with `pgrep -f` never ends after the process has finished](pgrep-wait-matches-itself.md) | solved |
| [A Windows ISO times out at "Press any key to boot from CD" in VMware Fusion](fusion-boot-prompt-keys.md) | workaround |
| [A wrapper DLL draws nothing under Wine on macOS, and its log says "version '130' is not supported"](wine-macos-opengl-2-1.md) | solved |
| [Almost every function differs between two builds, though the game behaves the same](rebuilt-build-every-function-differs.md) | solved |
| [An editor's save changes bytes nobody edited (fixed point through floats)](editor-float-view-changes-bytes.md) | solved |
| [An emulated function calls a Windows import that has no handler](emu-import-no-handler.md) | solved |
| [An emulated function stops with an unmapped memory read, write or fetch](emu-unmapped-memory.md) | solved |
| [Attaching winedbg to the running game kills it](winedbg-attach-kills.md) | dead-end |
| [Before modding a format: a writer that gives every file back byte for byte](writer-round-trip-every-file.md) | context |
| [Before moving geometry: find the bounds and normals stored with it](context-geometry-derived-data.md) | context |
| [CD-era games often look in their install folder before the CD: a mod can be a file, with no patch](context-mods-install-folder-first.md) | context |
| [CD-ROM data, disc checks and audio](cd-rom.md) | context |
| [Choosing a Windows machine for a game on an Apple-silicon Mac](context-windows-machines-apple-silicon.md) | context |
| [Clicks and keys from the agent do not reach a Wine window on macOS](wine-input-without-accessibility.md) | workaround |
| [Debugging a game under Wine or CrossOver through winedbg's GDB server: what works on Apple silicon](winedbg-gdb-server.md) | workaround |
| [Delphi programs: forms, class tables and event handlers](delphi.md) | context |
| [DirectInput ignores keys, because a window that is not foreground cannot acquire the device](directinput-not-foreground.md) | workaround |
| [Display modes, surfaces and palettes](display.md) | context |
| [DOS games in DOSBox-X: what to know before the first run](context-dos-dosbox.md) | context |
| [Emulating a range of code inside a big message handler](emulating-inside-a-handler.md) | context |
| [Emulating a whole system routine: stubs, whole records, and stale registers](emulating-a-system.md) | context |
| [Emulating game rules on a world built by hand](fake-worlds.md) | context |
| [Every new run exits at once, because a leftover game process holds its mutex](single-instance-leftover.md) | solved |
| [Game clocks, ticks and frame timing](timing.md) | context |
| [Game files and data formats](file-formats.md) | context |
| [Game input devices and event timing](input.md) | context |
| [Games with a build per 3D card: Glide, ATI3DCIF and the S3D Toolkit](context-3d-card-builds.md) | context |
| [Ghidra headless refuses a project path inside a folder whose name starts with a dot](ghidra-dot-folder-path.md) | solved |
| [Ghidra's pseudocode of a Watcom function leaves out arguments and stores that the disassembly shows](watcom-pseudocode-drops-register-arguments.md) | solved |
| [Globals in .bss hold code under the emulator, because the section's file pointer is 0](pe-bss-file-pointer-zero.md) | solved |
| [Gremlin Digital Video (`.GDV`) in a 1998 Windows game](gremlin-video.md) | context |
| [In a shared, loaded VM the game dies with the SSH session or misses key presses](emulated-pc-input-and-sessions.md) | workaround |
| [Keep a capture of every error dialog and crash, as it happens](context-keep-every-dialog.md) | context |
| [Keys must be held for a frame to see them](held-keys.md) | context |
| [Keys reach a game under Wine on Xvfb only with a window manager running](xvfb-needs-window-manager.md) | solved |
| [Lighting in a 256-colour renderer is a lookup in a shade table, with one level that changes nothing](context-palette-light-tables.md) | context |
| [macOS drops DYLD_ variables through /bin/sh, so a Wine build cannot load its libraries](macos-dyld-stripped.md) | solved |
| [Measure a function that walks the game's data structures under the emulator](emulate-function-on-game-structures.md) | context |
| [Measure a library's calls that lock mutexes, under the emulator](emulate-library-calls-stub-kernel-imports.md) | context |
| [Measuring a function that calls the whole game: intercept its callees](isolate-callees.md) | context |
| [Measuring ported systems when an unported one draws first](unported-share-search.md) | context |
| [Moving a project's labels and references to a newer version of the game](context-move-to-a-newer-version.md) | context |
| [Networking and multiplayer messages](networking.md) | context |
| [Port a game's rules before its physics, driven by a recorded run](context-port-rules-before-physics.md) | context |
| [Porting more of the original made a replay agree less with the recording](replay-stand-in-for-unported-logic.md) | solved |
| [Porting routines that build records in their stack frame: keep the frame as an array](frames-as-arrays.md) | context |
| [ptrace breakpoints in a Wine game leave every thread stopped](ptrace-breakpoints-wine.md) | solved |
| [REA through Ghidra: what a session costs, what a snapshot keeps and how labels interact](context-rea-sessions-and-snapshots.md) | context |
| [Record a whole run of the original from its memory as the reference for a port](record-a-run-from-memory.md) | context |
| [Record a Windows game's sound in a VM that has no sound card](record-sound-without-a-sound-card.md) | context |
| [Run the original's per-object step under the emulator on states from the running game](emulate-step-from-step-log.md) | context |
| [Run the original's routines on a whole-process snapshot](running-the-original-on-a-snapshot.md) | context |
| [Running the original in an emulated PC when Wine on the host fails](context-emulated-pc.md) | context |
| [Running the original's code on a whole-process snapshot](snapshot-emulation.md) | context |
| [Running the original's renderer on a snapshot of the game](renderer-on-a-snapshot.md) | context |
| [S3 ViRGE builds draw 15-bit pixels: their 16-bit surfaces are 5-5-5](s3-virge-15-bit-surfaces.md) | solved |
| [Scripted menu clicks land one time in three in a game with its own cursor](directinput-cursor-click-timing.md) | workaround |
| [Set the game's x87 control word before each emulated run](x87-control-word-per-run.md) | context |
| [Sound cards and audio paths in games](sound-card.md) | context |
| [SSH to a VM fails with "Too many authentication failures"](ssh-too-many-auth-failures.md) | solved |
| [Start a recorder when the next race begins instead of driving the menus](wait-for-the-next-race.md) | context |
| [The front end and the docked screens run at 640 x 400](docked-screens-640x400.md) | context |
| [The game asks for its CD under Wine on Linux, though the disc's folder is mapped with the right label](wine-linux-cd-drive.md) | solved |
| [The game asks for its CD while the disc is there, because some file failed to open](insert-cd-file-open-failed.md) | solved |
| [The game exits at once with code 0 and no message on a modern or emulated CPU](cpu-check-quits-on-unknown-cpu.md) | solved |
| [The game thread's FPU precision is not the one the start-up code sets](fpu-control-word-thread.md) | context |
| [The game wants the Manager's command line as well as its registry key](manager-command-line.md) | context |
| [The game's clock runs at a slightly different rate from real time](game-clock-not-real-time.md) | context |
| [The manual and the program's own text say what a system is for before the code does](context-read-the-story.md) | context |
| [The mouse pointer in the captures is drawn by the game, not the system](pointer-drawn-by-game.md) | context |
| [The opponents make choices but no random number generator is called](ai-chance-from-the-frame-clock.md) | context |
| [The player's car drives itself after a scripted driver was stopped](input-driver-keys-held-after-kill.md) | solved |
| [The port's colours are a little brighter than the original's, because 6-bit palette values were scaled to 255](palette-6bit-shift.md) | solved |
| [The text routine's line buffer outlives each text](stale-text-buffer.md) | context |
| [Times printed by the game differ by a hundredth from the same values printed by a tool](x87-truncating-time-format.md) | solved |
| [Verifying a decision routine by replaying its calls](replaying-calls.md) | context |
| [Verifying a system that calls into another, on a snapshot of the running game](snapshot-callback-oracle.md) | context |
| [Watcom C/C++ programs: calling convention, runtime and linker quirks](context-watcom.md) | context |
| [Windows 11 Arm Setup stops on the Windows logo in a VMware Fusion VM made with vmcli](fusion-windows-arm-setup-hangs.md) | open |
| [Windows 95 and DirectDraw games: what to know before the first run](context-windows-9x-directdraw.md) | context |
| [Windows Setup in QEMU says "The computer restarted unexpectedly" and will not continue](qemu-windows-setup-restarted.md) | solved |
| [Wine on macOS with Apple silicon: what to know before running a game there](context-wine-macos.md) | context |
| [Wine shows each palette index in a colour of its own](wine-palette-colours.md) | context |
| [Writing a Glide 2 replacement for an early Glide game](context-glide2-wrapper.md) | context |
| [Writing a PowerVR SGL replacement for a PCX1/PCX2 game](context-powervr-sgl-wrapper.md) | context |
| [x86 CPUs, instructions and arithmetic](x86-cpu.md) | context |

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
- `2010s`: Windows 7 to 10, DirectX 9 to 11, 64-bit builds. Also: windows 7 era, windows 10 era.

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
- `sound-card`: Sound Blaster and other sound hardware. Also: sound blaster, adlib, opl2, opl3, maxi sound, sam9407, awe32, awe64, aureal.

### api

- `win32`: Windows' own API: kernel32, user32, gdi32. Also: kernel32, user32, gdi32.
- `directdraw`: DirectDraw, 2D surfaces and display modes. Also: ddraw, ddraw.dll, idirectdraw.
- `directinput`: DirectInput keyboard, mouse and joystick. Also: dinput, dinput.dll.
- `directsound`: DirectSound. Also: dsound.
- `direct3d`: Direct3D. Also: d3d, d3dim.
- `glide`: 3dfx Glide. Also: glide2x, glide3x.
- `ati3dcif`: ATI's 3D Rage interface. Also: ati3dcif.dll.
- `s3d`: the S3D Toolkit for S3 ViRGE. Also: s3d toolkit, s3dtkw.
- `sgl`: PowerVR's SGL (Simple Graphics Library), the API of NEC's PowerVR PCX cards. Also: sgl.dll, powervr, pcx2.
- `dos-ints`: DOS and BIOS interrupts. Also: int 21h, int 10h, int 33h.
- `mci`: the Media Control Interface: CD audio and video. Also: mcicda, mciavi.
- `winmm`: WinMM's wave output, MIDI output and mixer. Also: waveout, midiout, mixer, winmm.dll.
- `card-sound-api`: a sound card maker's own library: the Maxi Sound 64's, A3D, EAX, the SoundFont Manager. Also: 94dll32, a3d.dll, eax, sfman32, qmixer.

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
- `no-sound`: the program runs, but silent or with sounds missing. Also: silent, no audio, muted, volume 0.
- `wrong-result`: a number, a table, a timing or a picture's order comes out different from the original, with no error. Also: wrong value, wrong table, too fast, out of order.
