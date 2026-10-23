#!/usr/bin/env bash
# Start a reverse-engineering project for one game, with GameFrame's toolkit and playbook.
#
#   scripts/new-project.sh DIR "Game Name" [PATH/TO/GAME] [--copy]
#
# Creates DIR as a git repository with:
#   AGENTS.md (+ CLAUDE.md)   the rules, with the game's name; edit the goal paragraph first
#   README.md, LICENSE-APACHE, LICENSE-CC0-1.0   front page (build/run per part); code Apache 2.0, content CC0 1.0
#   .claude/settings.json     enables GameFrame's Claude Code plugin: the re-* skills, the agents,
#                             the gates and the MCP servers come from it and stay current
#   gameframe                 runs GameFrame's toolkit on this project (./gameframe help); it finds
#                             GameFrame in a clone or in the plugin's install
#   docs/FINDINGS.md, docs/CHECKLIST.md, labels/names.tsv, labels/fields.tsv
#   target/, analysis/, derived/   for the game, Ghidra output and ground truth: ignored by git
# The guides stay in GameFrame (`./gameframe root` prints where), and on its website.
# --copy instead copies GameFrame's toolkit, skills, agents, gates and guides into the project,
# for agents that cannot use the Claude Code plugin. The skills go in skills/, with the links
# .claude/skills and .agents/skills to it, so every agent finds the one copy.
# scripts/update-project.sh refreshes such a copy, or moves it to the plugin with --plugin.
# With PATH/TO/GAME it copies the game's executables and DLLs into target/ (data files are
# left where they are: point the tools at them) and prints a first look at each binary.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
copy=0; args=()
for a in "$@"; do [[ "$a" == --copy ]] && copy=1 || args+=("$a"); done
set -- ${args[@]+"${args[@]}"}
[[ $# -ge 2 ]] || { sed -n '2,21p' "$0"; exit 2; }
dir="$1"; name="$2"; game="${3:-}"
[[ -e "$dir" && -n "$(ls -A "$dir" 2>/dev/null)" ]] && { echo "$dir exists and is not empty" >&2; exit 1; }
mkdir -p "$dir"/{target,analysis,derived,docs,labels,.claude}
if [[ $copy == 1 ]]; then
  cp -R "$ROOT/tools" "$ROOT/scripts" "$dir/"
  # One copy of the skills in skills/, and links for the agents that look elsewhere:
  # .claude/skills (Claude Code, Copilot, Cursor) and .agents/skills (Codex, Gemini CLI, others).
  cp -R "$ROOT/skills" "$dir/skills"
  mkdir -p "$dir/.agents"
  for l in .claude/skills .agents/skills; do
    # Without symbolic links (Windows without developer mode) the link is a second copy.
    ln -s ../skills "$dir/$l" 2>/dev/null && [[ -d "$dir/$l/re-start" ]] || { rm -rf "$dir/$l"; cp -R "$ROOT/skills" "$dir/$l"; }
  done
  cp -R "$ROOT/agents" "$dir/.claude/agents"
  # The agents' copies for Codex, Gemini CLI, Copilot and OpenCode (gameframe agents --emit).
  for d in .codex/agents .gemini/agents .github/agents .opencode/agents; do mkdir -p "$dir/$d" && cp "$ROOT/$d"/re-* "$dir/$d/"; done
  # The permissions and the gates (scripts/hooks/gate.py): docs/working/gates.md.
  cp "$ROOT/.claude/settings.json" "$dir/.claude/settings.json"
  cp "$ROOT/requirements.txt" "$ROOT/requirements-live.txt" "$ROOT/.mcp.json" "$ROOT/gameframe" "$dir/"
  # The reference documents the skills point to (docs/DOS.md, docs/ABI.md, ...).
  for f in ABI GAME-PATTERNS DOS LEGAL OUTPUTS SETUP COMPILED-CODE TYPES; do cp "$ROOT/docs/$f.md" "$dir/docs/"; done
  # The guides, and the diagrams and pictures they show (docs/assets/).
  for d in games observe engines hardware consoles handhelds computers arcade media working teams case-studies preservation assets; do
    cp -R "$ROOT/docs/$d" "$dir/docs/"
  done
else
  # The plugin brings the skills, agents, gates and MCP servers; the launcher runs the toolkit.
  cp "$ROOT/templates/project/settings.json" "$dir/.claude/settings.json"
  cp "$ROOT/templates/project/gameframe" "$dir/gameframe"
fi
# Code is Apache 2.0; documentation and other prose are CC0 1.0 (as in GameFrame).
cp "$ROOT/templates/project/licences/LICENSE-APACHE" "$ROOT/templates/project/licences/LICENSE-CC0-1.0" "$dir/"
cp "$ROOT/templates/project/gitignore" "$dir/.gitignore"
cp "$ROOT/templates/project/labels/"*.tsv "$dir/labels/"
for f in AGENTS.md README.md docs/FINDINGS.md docs/CHECKLIST.md; do
  sed -e "s/GAME_NAME/$name/g" -e "s/DATE/$(date +%Y-%m-%d)/" "$ROOT/templates/project/$f" > "$dir/$f"
done
printf 'The instructions for this repository are in AGENTS.md:\n\n@AGENTS.md\n' > "$dir/CLAUDE.md"
# target/ holds the game: Cargo must build elsewhere, or `cargo clean` deletes the game.
mkdir -p "$dir/.cargo"
printf '[build]\n# target/ holds your copy of the game: never let Cargo build (or clean) there.\ntarget-dir = "build"\n' > "$dir/.cargo/config.toml"
[[ -f "$ROOT/.env" ]] && cp "$ROOT/.env" "$dir/.env"
[[ -d "$dir/tools" ]] && { find "$dir/tools" -name '__pycache__' -prune -exec rm -r {} + 2>/dev/null || true; }
if [[ -n "$game" ]]; then
  find "$game" -maxdepth 2 -type f \( -iname '*.exe' -o -iname '*.dll' -o -iname '*.ovl' \) -exec cp {} "$dir/target/" \;
  echo "Copied $(ls "$dir/target" | wc -l | tr -d ' ') binaries into $dir/target/"
fi
(cd "$dir" && git init -q && git add -A && git commit -qm "Start reverse engineering $name" && echo "Created $dir (git repository, first commit made)")
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
