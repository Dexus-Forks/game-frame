# Gaps

The list of topics found missing from GameFrame's guides and tools, and where each one is now
covered. Tick an item when its guide section or tool is merged. Add new gaps at the end of
their group. Items marked *out of scope* stay listed, with the reason, so they are not
proposed again.

## Reading compiled code ([COMPILED-CODE.md](docs/COMPILED-CODE.md), [ABI.md](docs/ABI.md))

- [x] What each compiler stage removes, with a diagram
- [x] Order of evaluation of arguments and operands, with a diagram
- [x] Folded constants, and multiplication by a reciprocal
- [x] Strength reduction, code moved out of loops, unrolled loops
- [x] Sparse `switch` tables of values and addresses
- [x] Code moved away from the common path (`.cold`)
- [x] Registers and stack slots: live ranges, spills, shared slots, scheduling, removed writes
- [x] Cloned functions, unswitched loops, Pascal's static link
- [x] Checks the compiler added: range and overflow checks, stack checks
- [x] Code from compilers that do not optimise
- [x] `setjmp` and `longjmp`
- [x] Compiler quirks and real compiler bugs
- [x] What the linker leaves: link order, library members, relocations, functions found only through tables, with a diagram
- [x] What breaks the link order: unused-code removal, folding, LTO, profile order
- [x] Templates kept once, incremental-link jump tables and `int3` padding
- [x] Imports by ordinal, thunks, delay-loaded DLLs
- [x] Shared libraries on Linux and macOS: PLT, GOT, `get_pc_thunk`, Mach-O stubs, constructors, bundled versions
- [x] Weak and common symbols
- [x] Code compiled by the game's own tools: bytecode, tokenised BASIC
- [x] `movsx` and `movzx`, C's widening of small integers, mixed-type conversions
- [x] x86 BCD instructions (`daa`, `aam` and the rest), `fbstp`
- [x] `fprem` and `fprem1`
- [x] x87 reversed and popping forms (`fsubr`, `fdivr`), x87 compares and NaN

## Types and data ([TYPES.md](docs/TYPES.md))

- [x] Dates and times in data, nine stored forms (`tools/triage/dates.py`)
- [x] Format strings as evidence of argument types
- [x] Headers before the data: reads at negative offsets

## Game patterns ([GAME-PATTERNS.md](docs/GAME-PATTERNS.md))

- [x] Update order: objects added or removed during the update, double buffers, pools, queues, dirty flags, grids, with a diagram
- [x] Update groups (buckets)
- [x] Game time and real time
- [x] Start-up order of subsystems
- [x] Types defined in data with a parent; objects made from a template
- [x] Tables in place of maths
- [x] Actors as coroutines
- [x] Angles stored as integers, and the order of rotations
- [x] Procedural generation: seeds from coordinates, density maps, hand-made exceptions, templates, syllable names
- [x] Text ids by range, recursive escapes and formatters

## Graphics ([graphics-2d.md](docs/hardware/graphics-2d.md), [software-rendering.md](docs/hardware/software-rendering.md))

- [x] VGA split screens
- [x] What stays on screen when a game redraws only what changed
- [x] Surfaces lit in texture space
- [x] Antialiased lines
- [x] Polygons that are not convex: even-odd and non-zero rules
- [x] Hidden surfaces: z-buffer compare, sorted spans, BSP, portals, PVS
- [x] The card makers' 3D APIs in detail: Glide 2.x, ATI3DCIF, S3D Toolkit; one build per card ([card-apis.md](docs/hardware/card-apis.md), `gameframe-gpu-1997`, `re-accelerator`)
- [x] Hand-written MMX code: finding it, matching it to the plain build, measuring, porting ([simd.md](docs/hardware/simd.md), `gameframe-mmx`, `re-vectoriser`)
- [x] PowerVR SGL's scene-graph half: the 46 calls of POD's PowerVR build, a stand-in `sgl.dll` on OpenGL ([wrappers/sgl](wrappers/sgl/README.md))
- [x] PowerVR SGL's immediate-mode half (`sgltri_*`), which Hardwar's PowerVR mode uses. The 13 calls it lacked, and `sgltri_rerender`, are in `wrappers/sgl` ([sgl](wrappers/sgl/README.md), `test/sgltritest.c`). Shadow volumes given as triangles are still not drawn. No run of Hardwar has checked it
- [ ] Rendition RRedline and Matrox MSI in the same detail

## Sound ([sound.md](docs/hardware/sound.md), [audio coverage](docs/hardware/audio-coverage.md))

- [x] Map common PC audio device families by era, separate chip emulation from card behavior,
  compatibility libraries and host output, and record current GameFrame coverage
- [x] Implement the documented YMF262 register banks, two- and four-operator synthesis, eight
  waveforms, four output words, compatibility mode, timers, indexed ports and serial DAC frames
  (`gameframe-opl3`). Software tests cover native output routing and mode-gated pair selection.
- [x] Implement YAC512 serial input, FSEL decoding and channel latches (`gameframe-yac512`);
  software tests cover bit order, channel latches and captured FSEL mode. Its floating-DAC
  transfer remains open with software-only checks documented.
- [ ] Establish the YAC512 floating-DAC transfer curve from reliable specifications or a named
  software reference ([test plan](docs/hardware/ymf262-open-questions.md)); physical testing is not
  a blocker
- [ ] Verify how four-operator carrier groups use the paired `C0` output selectors across YMF262
  outputs A–D. The AdLib Gold Toolkit documents its D4/D5 stereo rule; a software black-box capture
  can resolve the remaining four-output mapping ([test plan](docs/hardware/ymf262-open-questions.md));
  physical hardware testing is not a blocker
- [ ] Compare YMF262 output and access timing against a named software reference when useful; this
  is not a release blocker and physical hardware testing is optional
- [x] Implement the documented AdLib Gold Control Chip bank switch, register/status access,
  EEPROM commands and audio-port relocation (`gameframe-adlib-gold`). Software tests cover the
  busy windows, restore, relocation, mixer controls and MMA-to-Control-Chip interrupt path. The
  Developer Toolkit resolves the driver's `0xFE`/`0xFF` toggles and second data-port reads.
- [ ] Compare those card paths with the original GSS drivers under an instrumented NE emulator;
  external DMA/IRQ delivery and revision-specific defaults remain open. Physical testing is not
  required.
- [x] Add the YMZ263B/MMA digital model and route its ports and sampling interrupt through
  `gameframe-adlib-gold`; implement the documented FIFO, rate, timer and MIDI surfaces
- [x] Apply AdLib Gold's documented linear source volumes, speaker/microphone/telephone gates,
  output channel selection, forced mono, mute and final volume controls in `mix_sources()`
- [ ] Compare YMZ263 ADPCM rounding, interleave parity and FIFO threshold transitions against the
  original GSS Wave/MIDI driver or a named software reference. The digital status map and MIDI
  request masks follow the toolkit. Add software-only checks as behavior is resolved; physical
  hardware is not required.
- [ ] Model TDA8425 tone and pseudo/spatial processing, analog filter/ADC paths and connect external
  DMA/IRQ controllers where software call paths establish their card-specific behavior
- [x] Add a Philips SAA1099 and Creative Music System/Game Blaster model using the chip data
  handbook and static traces of the original CMS drivers. The four-port write window, two-chip
  routing and documented registers are implemented; generated tone/noise vectors and CMS driver
  port traces still need software comparison ([findings](docs/hardware/cms-saa1099.md)). Physical
  testing is optional.
- [ ] Compare the SAA1099 noise sequence, tone-plus-noise gate, envelope update boundaries and
  CMS card-detection read against a named software reference or traced DOS driver. Physical card
  testing is optional ([findings](docs/hardware/cms-saa1099.md)).
- [ ] PC speaker, Tandy PSG and LPT DAC components
- [x] Add a reusable Sound Blaster DSP, mixer and SB16 MPU-401 UART model for Creative DSP
  generations 1.xx–4.xx (`gameframe-sound-blaster`); reuse the OPL2/OPL3 crates for card FM ports,
  including SB Pro 1's dual-OPL2 stereo routing. Software checks cover reset/version, MIDI
  timestamps and UART reset, 8-bit playback/capture, 16-bit playback/capture, FM port maps,
  high-speed command gating, IRQ acknowledgement and mixer profiles
- [x] Decode Creative 2-, 3- and 4-bit ADPCM to PCM in `gameframe-sound-blaster`. The 2-, 3- and
  4-bit recurrences, step changes, reference-byte initialization and packed-byte order come from
  static CT1741 firmware 4.05 disassembly. Sample parity is not yet measured.
- [ ] Compare the ADPCM decoder and byte request cadence against CT1741 firmware in a Ghidra 8051
  emulator with a stub sample port. Cover signs, magnitudes, predictor clamps, step changes,
  byte boundaries, the 3-bit tail, both nibbles, continuation blocks and request intervals.
  Physical testing is optional; details are in `docs/hardware/sound-blaster-dsp.md`
- [ ] Establish exact DSP busy, OPL2 port busy, FIFO, stereo cadence and DMA terminal-count timing
  with generated software comparisons. Physical testing is optional
- [ ] Connect the card events to a host ISA DMA and PIC model, including DMA pages, channel state,
  bus arbitration and PIC edge/EOI behavior
- [ ] Apply mixer source selection, analog gain, tone and filter behavior when a game-facing path
  requires those outputs
- [ ] Gravis UltraSound GF1 and InterWave voice, memory and interrupt behavior
- [ ] MPU-401 event timing and adapters for MT-32 and General MIDI synthesizers
- [ ] AWE SoundFont Manager and selected Windows card APIs, when a game's call sites require them

## Engines ([animation.md](docs/engines/animation.md))

- [x] Pose changes after blending: inverse kinematics, look-at, hand placement

## Observing the original ([observe/](docs/observe/README.md))

- [x] Taint: following a value, several sources, slices (`tools/emu/mem_trace.py`)
- [x] Symbolic execution for small checks
- [x] Time-travel debugging
- [x] Tracing calls in the running game (`tools/live/frida_trace.py`, installed by setup)
- [x] Adding code to the original: Frida, proxy DLLs, code caves
- [x] A proxy between client and server
- [x] From memory to the screen: the view and projection matrices
- [x] A tool that marks positions read from memory on a screenshot, from the camera's matrices (`tools/triage/screenmarks.py`)
- [x] `mem_trace.py`: follow values through registers, not only memory copies
- [x] `frida_trace.py`: log reads and writes of chosen memory in the running game (`--watch`)

## Media and protection ([media/](docs/media/README.md))

- [x] Working with an old protection check, and how to find each kind
- [x] Imports found by hash (`hashnames.py` with ROR-13)
- [x] Pressed and recordable discs, hub codes, CD-Text, discs that will not read
- [x] Overlays: data reloaded with a part, functions copied into two parts

## Tools

- [x] `tools/pe/relocs.py`: pointers in data from base relocations
- [x] `tools/pe/peinfo.py`: delay-loaded imports, relocation count
- [x] `tools/triage/compare_trees.py --similar`: shared content, renamed files
- [x] `tools/triage/hashnames.py`: ROR-13
- [x] `tools/formats/catalog.py`: a first resource viewer (every file, decoder results,
  pictures, format coverage), with the `re-browse` skill and [the guide](docs/working/viewer.md)
- [x] `tools/triage/coverage.py --module`: coverage of one overlay or module whose code is numbered from a shared base
- [x] `ExportDisasm.java` (`ghidra-run.sh disasm`): straight-line disassembly of code Ghidra did not make a function
- [x] Ports that run inside the original, with instruction counts and declines ([the guide](docs/working/hybrid-port.md))
- [x] `gameframe agents --emit`: the agents for Codex, Gemini CLI, Copilot and OpenCode ([the matrix](docs/working/agents.md#agents-in-every-client))
- [x] `new-project.sh --copy` and `update-project.sh` copy the agents for clients other than Claude Code
- [x] `tools/emu/pe32_emu.py`: 32-bit (x86) PE routines under Unicorn, with register inputs, MMX, and named import stubs
- [x] `tools/formats/shadetable.py`: shade, fog and light tables of palette games
- [x] `xorscan.py --records`, `--key-in`: a key XORed into one field of every directory record, found from zero padding and in the executable
- [x] `rawimage.py` and `catalog.py --dac shift`: 6-bit palettes shifted left by 2, as many loaders do; short palettes filled with magenta
- [x] `constants.py`: Park-Miller's 16807 reported when its modulus or Schrage's constants are beside it
- [x] `new-site.sh --manual` and a theme to colour from the game
- [x] `pe32_emu`: a section with a file pointer of 0 (Watcom's `.bss`) is zeroed, and the x87
  starts at 0x037f, as Windows does (Hardwar, POD; [the entry](docs/knowhow/pe-bss-file-pointer-zero.md))

Proposed by the POD project (its `docs/gameframe-lessons.md`) and not yet in GameFrame. Each
needs the test named there before it moves:

- [x] `ExportCallArgs` that knows Watcom's register order (EAX, EDX, EBX, ECX), so a constant
  passed in a register is reported: `ghidra-run.sh BIN callargs OUT regs=watcom FUNC`
- [x] `ghidra-run fill-gaps`: make the functions `exportcov` finds, then export again, so
  `decomp.c` and `functions.tsv` hold them (no separate `decomp-extra.c` is needed)
- [x] A replay harness for `re-verify`: accept a sample when any port state in its window
  matches (`tools/verify/replay.py`); the port drives itself from the recorded input
- [x] A reader for `memrec` recordings: `tools/live/memlog.py` (summary, CSV, one sample);
  printing values the game's way stays in the project
- [x] `memscan.py`: every copy of a value, before and after a mod is installed, is two
  `DUMP:=VALUE` rules ([the entry](docs/knowhow/context-mod-value-replaced-at-run-time.md))
- [x] A writer for block containers: re-laying entries and checksums is in
  [Writers, editors and mods](docs/working/mods.md); the writer itself belongs to each format
- [x] An editor stage in `re-browse` (section 5): offsets the reader records, one edit type,
  patches with byte undo, and a command line that makes the same edits
- [x] An axes check: `tools/triage/axes.py` names the up axis, its sign and the handedness
  from a recorded path over the level's vertices
- [x] `winput`: `point X Y wait 1000 button` clicks in games that draw their own cursor
- [x] From POD's replacement libraries: frame scalers (the `gameframe-scale` crate: nearest,
  Scale2x, CRT) and a probe-library generator (`tools/pe/probegen.py`)
- [ ] A render timing log for replacement libraries (frames per second, time per kind of
  command): small and specific to each renderer, so left to the library for now

## Reading the story ([lore.md](docs/working/lore.md))

- [x] The case for the lore as evidence for design decisions, with Hardwar as the worked example
- [x] A lore step in `re-inventory` and `re-start`, and the `docs/lore.md` template
- [x] `tools/triage/sentences.py`: a program's sentences, for reading its story
- [ ] Reading a PDF manual's text from the tools: needs a PDF library beyond `requirements.txt`
- [ ] Gremlin Digital Video (`.GDV`): cut-scenes of Gremlin-published games, not yet decoded

## Know-how ([docs/knowhow/](docs/knowhow/README.md), [the design](docs/working/knowhow.md))

- [x] A taxonomy of failure modes by era, platform, runtime, hardware, API, compiler, genre and symptom
- [x] The entry format, `gameframe advise` and its `--check`
- [x] A hook that names matching entries after a shell command in Claude Code
- [x] Seed entries from running a 1997 Windows game under Wine on Apple silicon, and the
  context for Windows 95 DirectDraw games, Wine on macOS and Watcom programs
- [x] PE tools fall back to `SizeOfRawData` when a section's VirtualSize is 0 (`peinfo`, `vtable`)
- [x] The same hook for Codex, Gemini CLI, Cursor and Copilot (their post-tool events)
- [x] An MCP server for `advise` (`gameframe serve`), for clients without shell hooks
- [x] A Wine-side input tool (`tools/live/winput.py`): keys as scancodes, mouse moves and
  clicks with `SendInput`, built from our own source
- [x] Context entries for DOS with DOSBox-X, 3D-card builds and 16-bit Windows
- [x] `peinfo --imports`, `--exports` and `--strings` work again (a variable shadowed the flags)
- [x] Thirty-five entries from POD's wrapper, mods, race trace, race port and track editor, and
  its emulated-PC route
- [x] `tools/triage/refs.py`: every instruction that refers to an address, from the code bytes
- [x] `tools/live/memrec.py`: memory snapshots to diff, and recordings of a running game
  (`--wait` for a phase)
- [x] `discinfo.py --iso`: a BIN/CUE's data track as an ISO for an emulator's CD-ROM drive
- [x] `tools/live/emupc.py`: the files for running the original in an emulated PC
- [x] `tools/live/shot.py`: captures that are never overwritten, with their text
- [x] `tools/live/modfolder.py`: mods installed into, listed in and removed from an install folder
- [x] `winput`: held keys released at exit, `release`, `point` and `button`, `--hold`
- [x] The know-how hook reads only what a command printed (a client's notes on changed files
  had matched entries' own text)
- [ ] Queries that `advise` missed, to decide on embeddings (none recorded yet)

Proposed by POD, now built:

- [x] Export coverage and the likely starts in its gaps (`tools/triage/exportcov.py`), for the
  batch decompile
- [x] Per-address errors in `ExportAddressBatch`, and no new function at an address that
  nothing refers to unless it follows a `ret` or padding
- [x] An x87 control word in `pe32_emu` (`fpu_cw=`), and `fpu_modes`, which lists the cases
  whose result depends on it; the sample's `fix_mul` uses the 1.5 × 2^68 `fadd`
- [x] Stop at a call and read its arguments in `pe32_emu` (`until=`); the sample's `report`
  calls `wsprintfA`
- [x] A reader for `memrec` recordings (`tools/live/memlog.py`: summary, CSV, 16.16 as decimals)
- [x] A lock for a shared emulator (`tools/live/emulock.py`)
- [x] A round-trip check (`validate.py --write`, with `encode` in the decoder)
- [x] A closed-loop cut for recorded paths (`tools/live/loopcut.py`)

Proposed by POD and not built yet, each with the reason:

- [x] Watcom register order (EAX, EDX, EBX, ECX) in `ExportCallArgs` (`regs=watcom`), tested on
  a register call in the sample DLL, built with zig and inline assembly
- [x] Layout helpers in `pe32_emu` (`array`, `put`, `get`), tested on a linked list and a graph
- [x] A replay harness for `re-verify`: `tools/verify/replay.py`, a window for samples that
  straddle frames, tested on a toy game; rebuilding frames at recorded clocks stays in the port
- [x] Printing recorded values the game's way (a game's own time format): game-specific, so it
  stays beside the port; `memlog.py` says so
- [x] A multi-entry block-container writer: the method is in [Writers, editors and
  mods](docs/working/mods.md); the writer belongs to each format
- [x] A derived-data finder for geometry (`tools/formats/derived.py`), tested on generated
  records with stored bounds and normals
- [x] Guides: [Recording the original](docs/observe/recording.md),
  [Replacement libraries](docs/hardware/replacement-libraries.md),
  [Writers, editors and mods](docs/working/mods.md), and an emulated-PC section in
  [Running the original](docs/observe/running-the-original.md)
- [x] An editor stage in `re-browse` (section 5) and an axes check (`tools/triage/axes.py`,
  tested on a generated level and its mirror)
- [x] `scripts/zcc.sh`, the `zig cc` linker for 32-bit Windows DLLs from Rust, tested with a
  two-export crate whose stdcall export runs under `pe32_emu`
- [x] From the wrapper: the scalers (the `gameframe-scale` crate) and the probe-library
  generator (`tools/pe/probegen.py`, its stubs run under `pe32_emu`)
- [ ] The other wrapper pieces (port-I/O shim, DirectDraw in memory, Glide 2 on OpenGL,
  rasteriser, render-thread timing): large. Each needs a Windows test program run under Wine.
  GameFrame's tests cannot run those programs yet
- [ ] A Wine runner for `tests/run.sh` (Wine on Linux CI), so the Windows-side tools above and
  `winput` can be tested for real

From POD's move to POD Gold and the vehicle creator (its `docs/gameframe-lessons.md`):

- [x] `memrec --name` finds the game's own process, never the shell or launcher that started
  it, tested with both ahead of it
- [x] Chance from the frame clock in `re-randomiser`, the FPU's control word in recordings,
  sequence measurements, the version move in `re-start`, and probe clients that take any JSON
  answer, as guidance with their know-how entries
- [ ] Byte-level build matching beside `match_functions.py`: instructions with `.reloc` words,
  call displacements and padding masked; globals paired through aligned references; a grade
  per pair (same bytes, same code, same logic, changed). Test: two builds of a sample with
  different compiler settings, every function matched, one changed constant reported alone
- [ ] Build detection in `memrec`: an address spec per build, chosen by the PE link time read
  from the process. Test: a test program built twice, each recording reads the right addresses
- [ ] A palette from a twin picture, beside the image decoders: infer an 8-bit picture's
  palette from the same picture in 16 bits. Test: a generated pair recovers the palette
- [ ] A line follower that drives a port along a recorded path through its probe, to compare
  two versions of a vehicle or a level

## Out of scope

These were considered and left out, with the reason:

- Scanner and parser theory, SSA construction, data-flow algorithms: they build compilers,
  and reading their output needs only the results above.
- HLA syntax, macros and compile-time language: specific to one assembler.
- Pentium pairing, cycle counting, cache tuning: they change speed, not results. Games that
  depend on the CPU's speed are covered in [DOS games](docs/DOS.md).
- Lock-free concurrency: threads and jobs are covered in
  [engine runtime](docs/engines/engine-runtime.md).
- Cheats (aimbots, ESP, wallhacks, triggerbots, recoil removal): they change play.
- Malware persistence, registry tricks, Office, PDF and HTML macro analysis: not games.
- Keygens, shellcode, ROP and stack smashing: security work, not porting.
- Tool walkthroughs (CD/DVD Inspector, libbfd, Immunity, OllyDbg): the methods are covered
  with the tools GameFrame uses.
- Sound card start-up handshakes: the port replaces the sound output. The timing that matters
  (interrupts used as a clock) is covered in [sound](docs/hardware/sound.md).
