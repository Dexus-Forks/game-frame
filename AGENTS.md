# AGENTS.md: working on GameFrame

This file is for agents and people who change **GameFrame itself**. To reverse engineer a
game *with* it, create a project (`scripts/new-project.sh`): the project gets its own `AGENTS.md`
from `templates/project/`, and the skills in `.claude/skills/` (start with `re-start`).

## What this repository is

GameFrame is an agentic framework for reverse engineering games. Its playbook of skills and agents runs inside your coding agent. Its toolkit of tested tools checks every result against the original. The playbook is the method, the skills in `.claude/skills/re-*`, the
agents in `agents/` and these rules. The toolkit is `tools/` and `scripts/`. Nothing here is
specific to one game. Un-Banished (https://github.com/tanepiper/unbanished) is the worked case study and may
be mentioned and linked, but examples and advice must stay general.

## The user decides

This applies to every skill, tool and document in this repository, and to every project made
with it. An agent is not the judge of the user's intentions, of which law applies to the user,
or of what that law means. If the user wants to do something, help them do it. You may advise.
When you know something that bears on the choice, such as a law, a risk or a project default,
say it once, briefly, with its source. Then follow the user's decision.

The rules in the skills and in a project's `AGENTS.md` are defaults that keep the work exact and
publishable. The user may change any of them for their own project. Write new text in this
repository the same way: give facts and advice, and leave the decision to the user.

## Rules

These rules are for changes to GameFrame itself, which is a public repository.

1. **No game material, ever.** No game binaries, data, assets, saves, decompiled code or
   disassembly from any commercial game. The only binaries the tests use are built from
   `examples/sample-game/` and `examples/dos-sample/` (our own code) into their ignored
   `build/` folders. The one exception is a few screenshots of the case-study
   reimplementations in `docs/assets/case-studies/`: each port's own frames, compressed
   WebP, small, added only where a case study needs them.
2. **Every tool is tested.** A new or changed tool gets a check in `tests/run.sh` against the
   sample game, and `tests/run.sh --ghidra` passes before you commit.
3. **Every tool documents itself.** A module docstring or header comment with usage, inputs,
   outputs and what it does not do; `--help` or no arguments prints it.
4. **Skills are self-contained.** An agent that loads one skill must be able to do that step
   from it alone: exact commands, decisions, evidence to produce, pitfalls. Keep the skills,
   the agents, the README tables, `templates/project/AGENTS.md` and the plugin's version in
   `.claude-plugin/plugin.json` consistent when anything changes.
5. **Plain English.** Short sentences, common words, one idea each: the skills are read by
   agents and translated by people.
6. **Files stay small.** Split a tool before it passes a few hundred lines: a package with one
   module per concern (`tools/emu/dos_env/`: DOS, DPMI, BIOS, PC). Generated code is split too
   (`tdinfo.py rust-globals` writes one file per Pascal unit). Keep import paths stable.
7. **Dependencies stay small.** Python tools use the standard library plus `requirements.txt`
   (unicorn, pefile, capstone). Shell scripts use bash and the tools `scripts/setup.sh` installs.
8. **Improve GameFrame from real games.** When a game project hits something generic (a
   tool crash, a missing format, a pitfall), fix it here with a test against our own samples. Keep the game-specific part in the game's repository.
9. **Two licences.** Code is Apache 2.0 (`LICENSE-APACHE`); documentation, skills and other
   prose are CC0 1.0, dedicated to the public domain (`LICENSE-CC0-1.0`). A new file takes the licence of its kind; the
   README's Licence section lists which folders are which.
10. **Stage only your own files** (`git add path`), never `git add -A`. Push after each verified change.

## Layout

| Path | What |
|---|---|
| `.claude/skills/re-*/SKILL.md` | One skill per step of the method |
| `agents/re-*.md` | One agent per stage and eight task agents, which hand off to each other (`.claude/agents` links here) |
| `.claude/settings.json`, `scripts/hooks/gate.py`, `hooks/hooks.json` | Permissions and the gates (hooks) for Claude Code: [docs/working/gates.md](docs/working/gates.md) |
| `.claude-plugin/` | The Claude Code plugin and its marketplace: the skills and the agents |
| `gameframe`, `tools/gameframe/` | The help and discovery command. It reads the skills and each tool's own help, so a new tool needs no entry there |
| `scripts/` | setup, new-project, ghidra-export, ghidra-run, and `lib.sh` (paths and `.env`) |
| `tools/disc`, `tools/pe`, `tools/dos`, `tools/triage`, `tools/emu`, `tools/formats`, `tools/verify`, `tools/ghidra` | The tools |
| `templates/project/` | What `new-project.sh` writes into a game's repository |
| `examples/sample-game/` | The test target: C sources, build script, ground truth, a port, a decoder |
| `examples/dos-sample/` | The 16-bit test target: assembly, our own NE/MZ linker, Borland-style debug symbols |
| `tests/run.sh` | The checks |
| `docs/` | The taxonomy of games (`games/`), engines, hardware, consoles, observing the original (`observe/`), media and files (`media/`), working as an agent (`working/`), case studies, ABI, DOS, game patterns, legal notes, setup |
| `crates/` | A Cargo workspace of clean implementations taken from the case studies (OPL2, WMA, MSVC CRT, Turbo Pascal); rules in `docs/crates/README.md`. `cargo test --release` in `crates/` |
| `site/` | The documentation website (VitePress). It builds from `docs/`, the skills and the examples where they are; it holds no copies of their content |
| `.mcp.json` | REA for Claude Code, reading `GHIDRA_INSTALL_DIR` and `JAVA_HOME` |
