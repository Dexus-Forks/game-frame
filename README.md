# reverse-engineer-games

A harness for AI agents (and people) to reverse engineer games from their shipped files:
understand how a game works, document its formats and rules, and reimplement it so it behaves
exactly like the original.

It packages a method, step-by-step agent skills, and tested tools around
[Ghidra](https://ghidra-sre.org), the [Unicorn](https://www.unicorn-engine.org) emulator,
[REA](https://github.com/morluto/rea) (Reverse Engineer Anything) and, for DOS games,
[DOSBox-X](https://dosbox-x.com) and its debugger. It grew out of
[Un-Banished](https://github.com/tanepiper/unbanished), a 1:1 reimplementation of the game
Banished built this way; see [docs/CASE-STUDY.md](docs/CASE-STUDY.md).

## Quick start

```bash
git clone https://github.com/tanepiper/reverse-engineer-games.git
cd reverse-engineer-games
scripts/setup.sh --install          # Ghidra, JDK 21, objdump, Node, Zig, and .venv (unicorn, pefile, capstone)
tests/run.sh --ghidra               # proves every tool on the sample game (about 1½ minutes)

scripts/new-project.sh ~/re/mygame "My Game" "/path/to/My Game"   # a project for one game
```

Then open the new project with your agent and ask it to start with the **re-start** skill.
Claude Code finds the skills in `.claude/skills/` automatically; other agents read `AGENTS.md`
and the `SKILL.md` files.

## The method

```
inventory -> decompile -> trace -> format -> emulate -> reimplement -> verify -> land
                            ^                                             |
                            +---------------- when a check fails ---------+
```

Two rules hold it together. **Every claim rests on evidence** you can point to (an address, a
value, a measurement) and is labelled *observed*, *measured*, *inferred* or *unknown*. **Every
number that matters is measured**: the original function runs under the emulator, and the
reimplementation must match it bit for bit.

## Skills

Each skill is a self-contained guide for one step: when to use it, exact commands, what evidence
to produce, and the pitfalls that cost time.

| Skill | Step |
|---|---|
| [re-start](.claude/skills/re-start/SKILL.md) | Rules, the loop, a first-session checklist, routing to the other skills |
| [re-setup](.claude/skills/re-setup/SKILL.md) | Install and check the toolchain; register REA for your agent |
| [re-inventory](.claude/skills/re-inventory/SKILL.md) | Which files hold the game's code; native, managed, scripted or packed; engine and middleware; data containers |
| [re-rea](.claude/skills/re-rea/SKILL.md) | When to use REA's MCP tools, and how they fit with the text export |
| [re-ghidra](.claude/skills/re-ghidra/SKILL.md) | Headless export, queries, one-off decompiles, vtables, constants, checking the disassembly |
| [re-trace](.claude/skills/re-trace/SKILL.md) | From a string, import, export or class name to the function, its fields and constants |
| [re-format](.claude/skills/re-format/SKILL.md) | Decode archives, resources and saves; prove decoders on every file |
| [re-emulate](.claude/skills/re-emulate/SKILL.md) | Run original functions under Unicorn for ground truth; trace random draws |
| [re-port](.claude/skills/re-port/SKILL.md) | Reimplement exactly: precision, order of operations, draw order, iteration order |
| [re-verify](.claude/skills/re-verify/SKILL.md) | Bit-exact parity, draw order, determinism, saves, pictures, version against version |
| [re-land](.claude/skills/re-land/SKILL.md) | Document in evidence style, label, commit safely |

## Scripts

| Script | What it does |
|---|---|
| `scripts/setup.sh [--install] [--rea]` | Checks every tool; installs what is missing; shows REA's registration plan |
| `scripts/new-project.sh DIR "Name" [GAME]` | Creates a project for one game: rules, skills, tools, ignored folders for the game and its derivatives |
| `scripts/ghidra-export.sh [--labels FILE] BINARY...` | Headless Ghidra analysis and a text export: pseudocode, functions, strings, imports, exports; names applied from a labels file |
| `scripts/ghidra-run.sh BINARY decompile\|named\|refs\|vtable\|labels OUT ARGS` | One-off jobs in the analysed project, without re-analysis |
| `scripts/extract-installer.sh [--detect] INSTALLER OUT` | Unpacks an installer or package without running it: Inno Setup (GOG), NSIS, InstallShield, MSI, archives, `.dmg`, `.pkg`, GOG's `.sh` |
| `scripts/dosbox-run.sh [--fileio] [--break] [--mcp PORT] DIR PROGRAM` | Runs a DOS program in DOSBox-X without a window: exit code, file I/O log, debugger control |
| `tests/run.sh [--ghidra]` | 71 checks of every tool against the two sample games |

## Tools

| Tool | What it answers |
|---|---|
| `tools/triage/compare_trees.py` | Two copies or releases compared file by file (SHA-256, case-insensitive paths); hashes to publish |
| `tools/pe/peinfo.py` | What is this binary: architecture, compiler, PDB path, imports, exports, RTTI, engine and packer fingerprints |
| `tools/pe/readdata.py` | The real value of a `DAT_` constant or table |
| `tools/pe/vtable.py` | The functions in a table of function pointers |
| `tools/triage/decomp_query.py` | A function with your labels; callers; callees; who uses a string; who writes a field; where the work is |
| `tools/emu/pe_emu.py` | Call any function of a Windows x64 EXE or DLL with chosen inputs and read the result |
| `tools/emu/call_trace.py` | The calls an emulated function makes (random draws), in order, with call sites |
| `tools/formats/reader.py`, `validate.py` | Prototype a format decoder; prove it consumes every byte of every file |
| `tools/verify/bitcompare.py` | Ground truth against reimplementation, bit for bit, with ULP distance |
| `tools/verify/seqdiff.py` | The first random draw (or any event) where two traces differ |
| `tools/ghidra/*.java` | The Ghidra scripts the shell scripts run (`ApplyLabels.java` names functions and creates missed ones) |
| `tools/dos/dosinfo.py` | What is this DOS binary: MZ, NE or LE; segments with Ghidra's selectors; imports; compiler, extender, packer fingerprints; debug symbols; Pascal strings |
| `tools/dos/tdinfo.py` | Borland Turbo Debugger symbols: original procedure, global and type names, parameters, record layouts, source lines, labels for Ghidra, Rust readers for records |
| `tools/dos/nedis.py` | 16-bit disassembly with relocations applied: far calls and DS globals by name |
| `tools/dos/readseg.py` | Typed values at `seg:off`, including Turbo Pascal `Real` and Pascal strings |
| `tools/emu/dos_emu/` | Call any function of a 16-bit NE (protected mode) or MZ (real mode) program, at Ghidra's addresses |
| `tools/emu/dos_env/` | A small DOS, BIOS, DPMI host and PC for dos_emu: run a whole DOS program headless, with files, timer interrupts, keys, and screenshots |
| `tools/dos/fileio_trace.py` | From a DOSBox-X file log: every file's reads in order, with record patterns |
| `tools/dos/dbx_control.py` | Drive DOSBox-X's debugger from a script: breakpoints, registers, memory |

## Worked example

[examples/sample-game](examples/sample-game) is a tiny Windows x64 game (an EXE and an engine
DLL, built from C with Zig on any host) with the parts real games have: a fixed-step update, a
random number generator, a constant table, objects with a vtable, and a data file. It comes with
a ground-truth script, a reimplementation that matches the original bit for bit on 20 seeds,
and a deliberately naive reimplementation (doubles instead of floats) that the checks catch.

[examples/dos-sample](examples/dos-sample) does the same for DOS: 16-bit Pascal-style code
assembled with Zig and linked by our own script into an NE program (with Borland debug
symbols), and a real-mode MZ program, checked under the emulator and under DOSBox-X.

## Documents

- [docs/DOS.md](docs/DOS.md): the DOS route: MZ/NE/LE, Ghidra's segment numbering, debug
  symbols, Pascal pitfalls, the 16-bit emulator, DOSBox-X and its debugger (also for agents,
  through the `dosbox` MCP server).
- [docs/GAME-PATTERNS.md](docs/GAME-PATTERNS.md): main loops, random generators, entities,
  resources, saves, rendering, as they appear in machine code.
- [docs/ABI.md](docs/ABI.md): calling conventions, C++ layout, floating-point and integer details
  that decide exactness.
- [docs/LEGAL.md](docs/LEGAL.md): your own copy, nothing redistributed, new code from
  documented behaviour.
- [docs/SETUP.md](docs/SETUP.md): the toolchain, and tools for targets beyond native code.
- [docs/CASE-STUDY.md](docs/CASE-STUDY.md): how Un-Banished used all of this.

## Scope

Native Windows x64 games and 16-bit DOS games (MZ real mode, NE protected mode, Borland
Pascal and C) are covered end to end; 32-bit LE/LX DOS-extender games are recognised and run
in DOSBox-X, but have no loader or emulator here yet. 32-bit, Linux and macOS binaries work with
Ghidra and the PE tools where they apply; the emulator needs a variant for them (`re-emulate`
lists what changes). Managed (.NET, Unity), scripted and packed games are routed to the right
tools by `re-inventory`.

Use it on games you own, keep the game's files and anything derived from them out of
repositories, and write new code from documented behaviour: see [docs/LEGAL.md](docs/LEGAL.md).

## Licence

MIT. See [LICENSE](LICENSE).
