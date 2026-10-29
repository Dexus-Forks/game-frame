#!/usr/bin/env bash
# Start a reverse-engineering project for one game, with GameFrame's toolkit and playbook.
#
#   scripts/new-project.sh DIR "Game Name" [PATH/TO/GAME] [--id ID] [--local] [--copy] [--agent LIST]
#
# Creates DIR as a git repository with:
#   AGENTS.md (+ CLAUDE.md)   the rules, with the game's name; edit the goal paragraph first
#   README.md, LICENSE-APACHE, LICENSE-CC0-1.0   front page (build/run per part); code Apache 2.0, content CC0 1.0
#   .claude/settings.json     enables GameFrame's Claude Code plugin: the re-* skills, the agents,
#                             the gates and the MCP servers come from it and stay current
#   gameframe                 runs GameFrame's toolkit on this project (./gameframe help); it finds
#                             GameFrame in a clone or in the plugin's install
#   docs/FINDINGS.md, docs/GAME_PROFILE.md, docs/CHECKLIST.md, docs/lore.md, docs/versions.md,
#   labels/names.tsv, labels/fields.tsv
#   game, media, saves, analysis, derived   links into the GameFrame home's games/ID/ (the game,
#                             its discs, saves, Ghidra output, ground truth): ignored by git, and outside it
#   game/GAME-FILES-NOT-BUILD-OUTPUT.txt   says game/ is the game, not build output
#   .cargo/config.toml        Cargo builds into build/
# DIR with no slash, such as `pod`, means $GAMEFRAME_HOME/projects/pod (~/gameframe/projects/pod).
# --id ID names the game in the home (games/ID/); the default is DIR's folder name in lower case.
# --local makes game/, media/, saves/, analysis/ and derived/ real folders inside the project instead:
# for a cloud machine or Windows without links. `gameframe home adopt` moves them out later.
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
# With PATH/TO/GAME it adds the game to the home with `gameframe game add`: a disc image or an
# installer is copied into games/ID/media/ and extracted into game/, and a folder is copied into
# game/. Then it prints a first look at each binary.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
copy=0; agent=all; args=(); id=""; local_dirs=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --copy) copy=1 ;;
    --agent) [[ $# -ge 2 ]] || { echo "--agent needs a list" >&2; exit 2; }; agent="$2"; shift ;;
    --agent=*) agent="${1#--agent=}" ;;
    --id) [[ $# -ge 2 ]] || { echo "--id needs a name" >&2; exit 2; }; id="$2"; shift ;;
    --local) local_dirs=1 ;;
    *) args+=("$1") ;;
  esac
  shift
done
set -- ${args[@]+"${args[@]}"}
[[ $# -ge 2 ]] || { sed -n '2,/^set -euo/p' "$0" | sed '$d'; exit 2; }
# The chosen agents, checked before anything is written.
agents="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import clients; print(",".join(clients.parse(sys.argv[2])))' "$ROOT/scripts" "$agent")"
claude=0; [[ ",$agents," == *",claude,"* ]] && claude=1
dir="$1"; name="$2"; game="${3:-}"
# A bare name is a project in the home.
[[ "$dir" == */* ]] || dir="$GAMEFRAME_HOME/projects/$dir"
[[ -n "$id" ]] || id="$(basename "$dir" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_\n-' '-')"
[[ -e "$dir" && -n "$(ls -A "$dir" 2>/dev/null)" ]] && { echo "$dir exists and is not empty" >&2; exit 1; }
mkdir -p "$dir"/{docs,labels,.claude}
gf() { GAMEFRAME_HOME="$GAMEFRAME_HOME" python3 "$ROOT/tools/gameframe" "$@"; }
if [[ $local_dirs == 1 ]]; then mkdir -p "$dir"/{game,media,saves,analysis,derived}
else gf game link "$id" "$dir" >/dev/null; fi
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
for f in AGENTS.md README.md docs/FINDINGS.md docs/GAME_PROFILE.md docs/CHECKLIST.md docs/lore.md docs/versions.md; do
  sed -e "s/GAME_NAME/$name/g" -e "s/DATE/$(date +%Y-%m-%d)/" "$ROOT/templates/project/$f" > "$dir/$f"
done
[[ $claude == 1 ]] && printf 'The instructions for this repository are in AGENTS.md:\n\n@AGENTS.md\n' > "$dir/CLAUDE.md"
# Cargo builds into build/, one folder for every kind of build output.
mkdir -p "$dir/.cargo"
printf '[build]\n# Build output goes in build/. game/ is your copy of the game.\ntarget-dir = "build"\n' > "$dir/.cargo/config.toml"
# A note in game/ itself, for people and cleaners that come across it.
python3 "$ROOT/scripts/hooks/gates/game_folder.py" mark "$dir"
# Settings for every project live in the home's config.env; .env holds what differs.
[[ -f "$GAMEFRAME_HOME/config.env" || ! -f "$ROOT/.env" ]] || cp "$ROOT/.env" "$dir/.env"
# Pin a plugin project to the one GameFrame checkout that made it (docs/working/environments.md):
# ./gameframe reads GAMEFRAME_ROOT from .env, also from the project's git worktrees. A copied
# project carries its own tools and needs no pin.
[[ $copy == 1 ]] || printf '\n# The GameFrame checkout this project uses (./gameframe reads it)\nGAMEFRAME_ROOT=%s\n' "$ROOT" >> "$dir/.env"
for copied in tools scripts; do
  [[ -d "$dir/$copied" ]] && { find "$dir/$copied" -name '__pycache__' -prune -exec rm -r {} + 2>/dev/null || true; }
done
if [[ -n "$game" && $local_dirs == 1 ]]; then
  if [[ -d "$game" ]]; then cp -R "$game"/. "$dir/game/"; else bash "$ROOT/scripts/extract-installer.sh" "$game" "$dir/game" >/dev/null; fi
  echo "Copied the game into $dir/game/"
elif [[ -n "$game" ]]; then
  gf game add "$id" "$game"
fi
rmdir "$dir/.claude" 2>/dev/null || true   # left empty without Claude Code
# The git pre-commit hook: the commit gate for people and every agent (docs/working/gates.md).
mkdir -p "$dir/.githooks" && cp "$ROOT/scripts/hooks/git/pre-commit" "$dir/.githooks/pre-commit"
(cd "$dir" && git init -q && git add -A && git commit -qm "Start reverse engineering $name" && echo "Created $dir (git repository, first commit made)")
git -C "$dir" config core.hooksPath .githooks
if [[ -n "$game" && -x "$PYTHON" ]]; then
  for b in "$dir"/game/*; do
    [[ -f "$b" && "$b" =~ \.([Ee][Xx][Ee]|[Dd][Ll][Ll]|[Oo][Vv][Ll]|[Cc][Oo][Mm])$ ]] || continue
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
