# Changelog

Every release of GameFrame, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the versions follow
[Semantic Versioning](https://semver.org/). The newest version here is the one in
`.claude-plugin/plugin.json`, and `tests/run.sh` checks that the two match.

## [Unreleased]

### Added

- Know-how: a taxonomy of failure modes and context, one Markdown entry per failure, each with
  its symptoms, cause, evidence, status (solved, workaround, dead end, open) and what was tried
  ([the design](docs/working/knowhow.md), [the entries](docs/knowhow/README.md)).
  `gameframe advise "<error text or topic>"` finds entries by error signature, facet, alias and
  word, reads a log on stdin, and checks every entry with `--check`. It reads a project's own
  `docs/knowhow/` too.
- A Claude Code hook, `scripts/hooks/advise.py`, that names the matching entries after a shell
  command (`PostToolUse` and `PostToolUseFailure`), once per session, and the context entry for
  Wine when a command starts it.
- Eighteen seed entries from the POD: Planet of Death project: privileged port probes under
  Wine, DirectDraw mode tables, windowed mode on a 16-bit desktop, 16-bit back surfaces, "insert
  the CD", ini paths, VirtualSize 0, `SuspendThread`, DirectInput focus, input inside Wine,
  `DYLD_*` on macOS, `wineserver -w`, leftover processes, winedbg, and the open `wow64cpu` fault
  under Rosetta with every attempt; and context for Windows 95 DirectDraw games, Wine on macOS
  and Watcom programs.
- The skills (`re-start`, `re-observe`, `re-land`'s new "Know-how" step), every agent and the
  project template suggest `gameframe advise` when a command fails or the work turns to a new
  era, runtime or hardware.

- Two hardware guides from the POD: Planet of Death (1997) project:
  [MMX and other SIMD code](docs/hardware/simd.md) (finding hand-written MMX routines,
  matching each to its plain twin through its callers, measuring both, porting exactly) and
  [The card makers' 3D APIs](docs/hardware/card-apis.md) (Glide 2.x, ATI3DCIF and the S3D
  Toolkit: reading their state calls, how textures reach each card, games with one build per
  card, and renderer profiles for viewers).
- Two crates: `gameframe-mmx`, the MMX instructions bit for bit, and `gameframe-gpu-1997`, the
  texture formats and API constants of Glide 2.x (with NCC), ATI3DCIF and the S3D Toolkit,
  with one texel decoder.
- Two specialist task agents: `re-vectoriser` (SIMD code and CPU builds) and `re-accelerator`
  (3D card APIs and per-card builds).
- `tools/emu/pe32_emu.py` runs routines of 32-bit (x86) Windows games under Unicorn, with
  inputs in registers or on the stack, MMX included, and stops at any import without a
  handler, naming it. The sample game builds a 32-bit DLL (`mixer32.c`) to test it.
- `tools/triage/insns.py` lists the pattern for MMX code among its groups.

### Changed

- `peinfo` and `vtable` use a section's raw size when its VirtualSize is 0, as Watcom's linker
  writes it; `peinfo` marks such a section.

- `re-site`, `re-publisher`, `re-land` and `docs/OUTPUTS.md` now allow screenshots of the
  original game and of the project's viewer in documents and on the site, a few at a time,
  each captioned with its source. The game's files, extracted assets in bulk and decompiled
  code still stay out of git.
### Fixed

- The Claude Code commit and push gates check the command's own git verbs. Claude Code's
  `if: Bash(git commit *)` also matches commands it cannot parse (a heredoc, a shell variable,
  a for loop), so the gates fired on commands that neither commit nor push.

## [1.0.1] - 2026-10-23

GameFrame 1.0.1 sets up projects for the coding agents they name, and adds a website with a
diary of the work.

### Skills and agents

- A new skill, `re-site`, builds and keeps a project's website: a VitePress site in
  `sites/<name>/`, or one `site/`. It holds a diary with one entry per milestone and the
  project's documents shown through `@include`, so each has one copy. Entries are written for
  readers, as findings with their evidence, never as commit lists or tool output.
- A new task agent, `re-publisher`, writes the diary entry for each milestone, adds pages for
  new documents and checks the site before `re-lander` commits it.
- `scripts/new-site.sh` starts a site from `templates/site/`, with a GitHub Pages workflow that
  builds every site in the project.
- `tools/site/check.py` checks a site for game files, decompiled code, missing includes, diary
  entries without a title or summary, and entries that read like a commit log.
- New projects get milestones in their `AGENTS.md` and checklist. A viewer of every asset comes
  before the game, since each format then becomes a reader, a document and a viewer panel that
  the rest of the project reuses. `re-start` gives the same goals.

### Projects

- `new-project.sh --agent LIST` sets a project up for the coding agents it names: `claude`,
  `codex`, `gemini`, `copilot`, `cursor`, `opencode`, or `all` (the default). Each agent gets its
  skill link, its agents, its gate hooks and its MCP servers. Without `claude` the project has no
  `.claude/settings.json` or `CLAUDE.md`.
- `update-project.sh --agent LIST` adds agents to a project. Without it, the script refreshes
  the agents whose files the project already has.
- A plugin project now works in the other agents too. They get a copy of the skills in `skills/`
  with the `.agents/skills` link, and Cursor gets its agents in `.cursor/agents`.
- `./gameframe gate KIND` runs GameFrame's gates from a plugin project, which has no `scripts/`.
  Its Codex, Gemini CLI, Cursor and Copilot hooks call it.
- `scripts/clients.py` holds the list of files each agent needs. Both project scripts use it.
- A copied project's hook files now come from GameFrame too, so `new-project.sh --copy` gives
  Codex, Cursor and Copilot their hooks.

## [1.0.0] - 2026-10-23

GameFrame 1.0.0 runs its playbook in any coding agent. Claude Code, Codex, Gemini CLI, GitHub
Copilot, Cursor and OpenCode can each use the skills, the agents, the gates and the MCP servers.

### Skills

- The skills live in one `skills/` folder. `.claude/skills` and `.agents/skills` link to it, so
  every agent finds the same copy.
- The skills' wording no longer assumes Claude Code.

### Agents

- `agents/re-*.md` stays the one source for the 9 stage agents and 9 task agents.
- `gameframe agents --emit all` writes copies for Codex (`.codex/agents`), Gemini CLI
  (`.gemini/agents`), Copilot (`.github/agents`) and OpenCode (`.opencode/agents`). Each copy
  keeps its agent's limits, as an allowlist, a deny map or a sentence in its instructions.
- A session that cannot start subagents takes each role itself, from the agent's file.
- The support matrix in [docs/working/agents.md](docs/working/agents.md#agents-in-every-client)
  shows where each client reads rules, skills, subagents, hooks and MCP servers.

### Gates

- A git pre-commit hook blocks game material and decompiled code for every client and for people.
- Hook adapters for Claude Code, Codex, Gemini CLI, Cursor and Copilot. Outside Claude Code,
  agents name themselves to the gates with `GAMEFRAME_AGENT=<name>`.

### MCP and rules

- `gameframe mcp --emit all` writes each client's MCP configuration from `.mcp.json`, and points
  each client at `AGENTS.md`.

### Projects

- `new-project.sh --copy` and `update-project.sh` copy the skills folder and every client's
  agents. Older projects' `.claude/skills` move to `skills/`.

### Plugin

- The Claude Code plugin now carries version `1.0.0`. Claude Code updates it when the version
  changes, so plugin projects move from release to release.

[Unreleased]: https://github.com/tanepiper/game-frame/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/tanepiper/game-frame/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/tanepiper/game-frame/releases/tag/v1.0.0
