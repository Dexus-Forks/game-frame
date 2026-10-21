# AGENTS.md: reverse engineering GAME_NAME

Rules for every agent (Claude Code, Codex, Cursor and others) and every person working here.
The method and the tools come from the GameFrame harness; its skills are in
`.claude/skills/` (agents without skill support: read each `SKILL.md`).

## The goal

GOAL: write one paragraph here. For example: "a 1:1 reimplementation of GAME_NAME VERSION in
LANGUAGE, reading the player's own data files", or "document the save format", or "a mod
loader". Say what "done" means and how exact it must be.

## Non-negotiable rules

1. **Never commit game material.** The game's binaries and data stay in `target/`, Ghidra
   output and decompiled code in `analysis/`, emulator output and extracted assets in
   `derived/`. All three are ignored by git; never force-add past `.gitignore`. Do not paste
   decompiled code, disassembly or data dumps into tracked files: describe findings in prose,
   tables, addresses (`FUN_1400be770`), field offsets (`+0x44`) and constants.
2. **Evidence before code.** Every finding and every reimplemented behaviour names the original
   function it came from, and says how it is known: *observed* (read in code or data),
   *measured* (emulated or run), *inferred* (a reasoned guess) or *unknown* (an open question).
3. **Measure before claiming exactness.** Numeric, random or procedural code is checked against
   the original under the emulator (`re-emulate`), bit for bit.
4. **Mark every intentional difference** from the original in a code comment containing
   `Deviation:` (and a piece left out with `not ported`), then regenerate the ledger:
   `tools/triage/deviations.py --out docs/DEVIATIONS.md SOURCE_DIR`.
5. **Keep the core exact and the additions apart.** The part that decides what happens (rules,
   clock, random numbers, saves) does not draw, play sound or use the network. Anything the
   original does not have (multiplayer, mod code, debug tools) is an addition behind a build
   option, reaches the core only through commands, events and queries, and can be switched off
   for a run (`re-port`, "Additions the original does not have").
6. **Write it down.** Findings go into `docs/`, function names into `labels/names.tsv`, struct
   fields into `labels/fields.tsv`, progress into `docs/CHECKLIST.md`. An undocumented finding
   is re-derived by the next session.
7. **Several sessions may work at once.** Stage only your own files (`git add path`), never
   `git add -A`, and never revert work you did not do.
8. **Ask before downloading** anything; say the file, the source and the size.
9. **Licence and instructions.** Our code is Apache 2.0 (LICENSE-APACHE; `license =
   "Apache-2.0"` in manifests); documentation and other prose are CC BY 4.0
   (LICENSE-CC-BY-4.0). Every part (core, player,
   viewer, tools) has build and run instructions in its README and a row in README.md.
10. **Keep files small and modular.** One concern per module; split a file before it passes a
   few hundred lines, generated code included (one file per original unit or table).

## Where things are

| Path | What |
|---|---|
| `target/` | Your copy of the game (ignored) |
| `analysis/export/<binary>/` | Ghidra's text export: decomp.c, functions.tsv, strings.txt, imports, exports (ignored) |
| `derived/` | Ground truth from the emulator, extracted files (ignored) |
| `docs/FINDINGS.md` | The overview: binaries, engine, formats, systems, with links |
| `docs/CHECKLIST.md` | What is traced, measured and reimplemented, and what is next |
| `labels/names.tsv`, `labels/fields.tsv` | Names for original functions and struct fields, read by the tools |
| `tools/`, `scripts/` | The harness's tools: https://github.com/tanepiper/game-frame |
| `docs/DOS.md`, `docs/ABI.md`, `docs/GAME-PATTERNS.md`, `docs/OUTPUTS.md` | The harness's reference documents (DOS games: start with DOS.md; what to ship: OUTPUTS.md) |
| `docs/games/`, `docs/observe/`, `docs/engines/`, `docs/hardware/`, `docs/consoles/`, `docs/working/` | The harness's taxonomy of games (with this game's family and its rule set) and how to observe the original: inputs, interface, object model |
| `.mcp.json` | MCP servers for the agent: REA, and `dosbox` (DOSBox-X's debugger, for DOS games) |

## The loop

inventory → observe → decompile → trace → (format) → emulate → reimplement → verify → land. Start each
session by reading `docs/CHECKLIST.md` and searching `docs/` and `labels/` for the addresses
and names you are about to work on.

Work a subsystem at a time and verify by risk (`re-start`, "Spend effort where it pays";
`re-verify` section 0): fuzz only maths, random draws and branchy code; check the rest
against recorded frames.

When something here would help any game (a tool fix, a new format reader, a pitfall), add it
to the harness too, with a test against its own samples.
