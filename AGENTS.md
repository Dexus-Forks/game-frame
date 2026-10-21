# AGENTS.md: working on the reverse-engineer-games harness

This file is for agents and people who change **the harness itself**. To reverse engineer a
game *with* it, create a project (`scripts/new-project.sh`): the project gets its own `AGENTS.md`
from `templates/project/`, and the skills in `.claude/skills/` (start with `re-start`).

## What this repository is

A generic method, skills and tools for reverse engineering games. Nothing here is specific to
one game. Un-Banished (https://github.com/tanepiper/unbanished) is the worked case study and may
be mentioned and linked, but examples and advice must stay general.

## Rules

1. **No game material, ever.** No game binaries, data, assets, saves, decompiled code or
   disassembly from any commercial game. The only binaries the tests use are built from
   `examples/sample-game/` and `examples/dos-sample/` (our own code) into their ignored
   `build/` folders.
2. **Every tool is tested.** A new or changed tool gets a check in `tests/run.sh` against the
   sample game, and `tests/run.sh --ghidra` passes before you commit.
3. **Every tool documents itself.** A module docstring or header comment with usage, inputs,
   outputs and what it does not do; `--help` or no arguments prints it.
4. **Skills are self-contained.** An agent that loads one skill must be able to do that step
   from it alone: exact commands, decisions, evidence to produce, pitfalls. Keep the skills,
   the README tables and `templates/project/AGENTS.md` consistent when anything changes.
5. **Plain English.** Short sentences, common words, one idea each: the skills are read by
   agents and translated by people.
6. **Files stay small.** Split a tool before it passes a few hundred lines: a package with one
   module per concern (`tools/emu/dos_env/`: DOS, DPMI, BIOS, PC). Generated code is split too
   (`tdinfo.py rust-globals` writes one file per Pascal unit). Keep import paths stable.
7. **Dependencies stay small.** Python tools use the standard library plus `requirements.txt`
   (unicorn, pefile, capstone). Shell scripts use bash and the tools `scripts/setup.sh` installs.
8. **Improve the harness from real games.** When a game project hits something generic (a
   tool crash, a missing format, a pitfall), fix it here with a test against our own samples,
   and keep the game-specific part in the game's repository.
9. **Stage only your own files** (`git add path`), never `git add -A`, and push after each
   verified change.

## Layout

| Path | What |
|---|---|
| `.claude/skills/re-*/SKILL.md` | One skill per step of the method |
| `scripts/` | setup, new-project, ghidra-export, ghidra-run, and `lib.sh` (paths and `.env`) |
| `tools/pe`, `tools/dos`, `tools/triage`, `tools/emu`, `tools/formats`, `tools/verify`, `tools/ghidra` | The tools |
| `templates/project/` | What `new-project.sh` writes into a game's repository |
| `examples/sample-game/` | The test target: C sources, build script, ground truth, a port, a decoder |
| `examples/dos-sample/` | The 16-bit test target: assembly, our own NE/MZ linker, Borland-style debug symbols |
| `tests/run.sh` | The checks |
| `docs/` | ABI, game patterns, legal notes, setup, the case study |
| `.mcp.json` | REA for Claude Code, reading `GHIDRA_INSTALL_DIR` and `JAVA_HOME` |
