#!/usr/bin/env bash
# Start a reverse-engineering project for one game, with GameFrame's toolkit and playbook.
#
#   scripts/new-project.sh DIR "Game Name" [PATH/TO/GAME] [--copy] [--agent LIST]
#
# Creates DIR as a git repository with:
#   AGENTS.md (+ CLAUDE.md)   the rules, with the game's name; edit the goal paragraph first
#   README.md, LICENSE-APACHE, LICENSE-CC0-1.0   front page (build/run per part); code Apache 2.0, content CC0 1.0
#   .claude/settings.json     enables GameFrame's Claude Code plugin: the re-* skills, the agents,
#                             the gates and the MCP servers come from it and stay current
#   gameframe                 runs GameFrame's toolkit on this project (./gameframe help); it finds
#                             GameFrame in a clone or in the plugin's install
#   docs/FINDINGS.md, docs/CHECKLIST.md, docs/versions.md, labels/names.tsv, labels/fields.tsv
#   target/, analysis/, derived/   for the game, Ghidra output and ground truth: ignored by git
# The guides stay in GameFrame (`./gameframe root` prints where), and on its website.
# --copy instead copies GameFrame's toolkit, skills, agents, gates and guides into the project,
# for agents that cannot use the Claude Code plugin. The skills go in skills/, with the links
# .claude/skills and .agents/skills to it, so every agent finds the one copy.
# scripts/update-project.sh refreshes such a copy, or moves it to the plugin with --plugin.
# --agent LIST names the coding agents to set the project up for, separated by commas: claude,
# codex, gemini, copilot, cursor, opencode, or all (the default). Each gets its skill link, its
# agents, its gate hooks and its MCP servers (scripts/clients.py). Without claude there is no
# .claude/settings.json or CLAUDE.md. In a plugin project the other agents get a copy of the
# skills in skills/, since the plugin serves only Claude Code.
# With PATH/TO/GAME it copies the game's executables and DLLs into target/ (data files are
# left where they are: point the tools at them) and prints a first look at each binary.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
copy=0; agent=all; args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --copy) copy=1 ;;
    --agent) [[ $# -ge 2 ]] || { echo "--agent needs a list" >&2; exit 2; }; agent="$2"; shift ;;
    --agent=*) agent="${1#--agent=}" ;;
    *) args+=("$1") ;;
  esac
  shift
done
set -- ${args[@]+"${args[@]}"}
[[ $# -ge 2 ]] || { sed -n '2,26p' "$0"; exit 2; }
# The chosen agents, checked before anything is written.
agents="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import clients; print(",".join(clients.parse(sys.argv[2])))' "$ROOT/scripts" "$agent")"
claude=0; [[ ",$agents," == *",claude,"* ]] && claude=1
dir="$1"; name="$2"; game="${3:-}"
[[ -e "$dir" && -n "$(ls -A "$dir" 2>/dev/null)" ]] && { echo "$dir exists and is not empty" >&2; exit 1; }
mkdir -p "$dir"/{target,analysis,derived,docs,labels,.claude}
if [[ $copy == 1 ]]; then
  cp -R "$ROOT/tools" "$ROOT/scripts" "$dir/"
  # The site template that scripts/new-site.sh reads (re-site).
  mkdir -p "$dir/templates" && cp -R "$ROOT/templates/site" "$dir/templates/site"
  # One copy of the skills in skills/, and links for the agents that look elsewhere:
  # .claude/skills (Claude Code, Copilot, Cursor) and .agents/skills (Codex, Gemini CLI, others).
  # The links and each agent's own files come from scripts/clients.py, below.
  cp -R "$ROOT/skills" "$dir/skills"
  # The agents' source: gameframe agents reads it, and Claude Code and Cursor run it.
  cp -R "$ROOT/agents" "$dir/.claude/agents"
  # The permissions and the gates (scripts/hooks/gate.py): docs/working/gates.md.
  [[ $claude == 1 ]] && cp "$ROOT/.claude/settings.json" "$dir/.claude/settings.json"
  cp "$ROOT/requirements.txt" "$ROOT/requirements-live.txt" "$ROOT/.mcp.json" "$ROOT/gameframe" "$dir/"
  # The reference documents the skills point to (docs/DOS.md, docs/ABI.md, ...).
  for f in ABI GAME-PATTERNS DOS LEGAL OUTPUTS SETUP COMPILED-CODE TYPES; do cp "$ROOT/docs/$f.md" "$dir/docs/"; done
  # The guides, and the diagrams and pictures they show (docs/assets/).
  for d in games observe engines hardware consoles handhelds computers arcade media working teams case-studies preservation assets; do
    cp -R "$ROOT/docs/$d" "$dir/docs/"
  done
else
  # The plugin brings the skills, agents, gates and MCP servers; the launcher runs the toolkit.
  [[ $claude == 1 ]] && cp "$ROOT/templates/project/settings.json" "$dir/.claude/settings.json"
  cp "$ROOT/templates/project/gameframe" "$dir/gameframe"
fi
# Each chosen agent's skill link, agents, gate hooks and MCP servers.
python3 "$ROOT/scripts/clients.py" write "$dir" --agent "$agents" $([[ $copy == 1 ]] || echo --plugin) >/dev/null
# Code is Apache 2.0; documentation and other prose are CC0 1.0 (as in GameFrame).
cp "$ROOT/templates/project/licences/LICENSE-APACHE" "$ROOT/templates/project/licences/LICENSE-CC0-1.0" "$dir/"
cp "$ROOT/templates/project/gitignore" "$dir/.gitignore"
cp "$ROOT/templates/project/labels/"*.tsv "$dir/labels/"
for f in AGENTS.md README.md docs/FINDINGS.md docs/CHECKLIST.md docs/versions.md; do
  sed -e "s/GAME_NAME/$name/g" -e "s/DATE/$(date +%Y-%m-%d)/" "$ROOT/templates/project/$f" > "$dir/$f"
done
[[ $claude == 1 ]] && printf 'The instructions for this repository are in AGENTS.md:\n\n@AGENTS.md\n' > "$dir/CLAUDE.md"
# target/ holds the game: Cargo must build elsewhere, or `cargo clean` deletes the game.
mkdir -p "$dir/.cargo"
printf '[build]\n# target/ holds your copy of the game: never let Cargo build (or clean) there.\ntarget-dir = "build"\n' > "$dir/.cargo/config.toml"
[[ -f "$ROOT/.env" ]] && cp "$ROOT/.env" "$dir/.env"
[[ -d "$dir/tools" ]] && { find "$dir/tools" -name '__pycache__' -prune -exec rm -r {} + 2>/dev/null || true; }
if [[ -n "$game" ]]; then
  find "$game" -maxdepth 2 -type f \( -iname '*.exe' -o -iname '*.dll' -o -iname '*.ovl' \) -exec cp {} "$dir/target/" \;
  echo "Copied $(ls "$dir/target" | wc -l | tr -d ' ') binaries into $dir/target/"
fi
rmdir "$dir/.claude" 2>/dev/null || true   # left empty without Claude Code
# The git pre-commit hook: the commit gate for people and every agent (docs/working/gates.md).
mkdir -p "$dir/.githooks" && cp "$ROOT/scripts/hooks/git/pre-commit" "$dir/.githooks/pre-commit"
(cd "$dir" && git init -q && git add -A && git commit -qm "Start reverse engineering $name" && echo "Created $dir (git repository, first commit made)")
git -C "$dir" config core.hooksPath .githooks
if [[ -n "$game" && -x "$PYTHON" ]]; then
  for b in "$dir"/target/*; do
    if "$PYTHON" "$ROOT/tools/pe/peinfo.py" "$b" >/dev/null 2>&1; then "$PYTHON" "$ROOT/tools/pe/peinfo.py" "$b" | head -4
    else "$PYTHON" "$ROOT/tools/dos/dosinfo.py" "$b" 2>/dev/null | head -6; fi
  done
fi
cat <<NEXT

Next:
  1. Edit the goal paragraph in $dir/AGENTS.md.
  2. cd $dir && $([[ $copy == 1 ]] && echo scripts/setup.sh || echo ./gameframe doctor) --install
  3. Ask your agent to start with the re-start skill (it routes every step from there).
NEXT
