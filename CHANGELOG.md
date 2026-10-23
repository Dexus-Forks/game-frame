# Changelog

Every release of GameFrame, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the versions follow
[Semantic Versioning](https://semver.org/). The newest version here is the one in
`.claude-plugin/plugin.json`, and `tests/run.sh` checks that the two match.

## [Unreleased]

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
