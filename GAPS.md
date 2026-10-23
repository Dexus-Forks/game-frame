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
- [ ] PowerVR SGL, Rendition RRedline and Matrox MSI in the same detail

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
- [ ] Queries that `advise` missed, to decide on embeddings (none recorded yet)

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
