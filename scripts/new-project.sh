#!/usr/bin/env bash
# Start a reverse-engineering project for one game, with this harness's tools and skills.
#
#   scripts/new-project.sh DIR "Game Name" [PATH/TO/GAME]
#
# Creates DIR as a git repository with:
#   AGENTS.md (+ CLAUDE.md)   the rules, with the game's name; edit the goal paragraph first
#   .claude/skills/           the re-* skills (every step of the method)
#   tools/, scripts/          the harness's tools, requirements.txt, .mcp.json (REA)
#   docs/FINDINGS.md, docs/CHECKLIST.md, labels/names.tsv, labels/fields.tsv
#   target/, analysis/, derived/   for the game, Ghidra output and ground truth: ignored by git
# With PATH/TO/GAME it copies the game's executables and DLLs into target/ (data files are
# left where they are: point the tools at them) and prints a first look at each binary.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
[[ $# -ge 2 ]] || { sed -n '2,15p' "$0"; exit 2; }
dir="$1"; name="$2"; game="${3:-}"
[[ -e "$dir" && -n "$(ls -A "$dir" 2>/dev/null)" ]] && { echo "$dir exists and is not empty" >&2; exit 1; }
mkdir -p "$dir"/{target,analysis,derived,docs,labels,.claude}
cp -R "$ROOT/tools" "$ROOT/scripts" "$dir/"
cp -R "$ROOT/.claude/skills" "$dir/.claude/"
cp "$ROOT/requirements.txt" "$ROOT/.mcp.json" "$dir/"
cp "$ROOT/templates/project/gitignore" "$dir/.gitignore"
cp "$ROOT/templates/project/labels/"*.tsv "$dir/labels/"
for f in AGENTS.md docs/FINDINGS.md docs/CHECKLIST.md; do
  sed -e "s/GAME_NAME/$name/g" -e "s/DATE/$(date +%Y-%m-%d)/" "$ROOT/templates/project/$f" > "$dir/$f"
done
printf 'The instructions for this repository are in AGENTS.md:\n\n@AGENTS.md\n' > "$dir/CLAUDE.md"
[[ -f "$ROOT/.env" ]] && cp "$ROOT/.env" "$dir/.env"
find "$dir/tools" -name '__pycache__' -prune -exec rm -r {} + 2>/dev/null || true
if [[ -n "$game" ]]; then
  find "$game" -maxdepth 2 -type f \( -iname '*.exe' -o -iname '*.dll' \) -exec cp {} "$dir/target/" \;
  echo "Copied $(ls "$dir/target" | wc -l | tr -d ' ') binaries into $dir/target/"
fi
(cd "$dir" && git init -q && git add -A && git commit -qm "Start reverse engineering $name" && echo "Created $dir (git repository, first commit made)")
if [[ -n "$game" && -x "$PYTHON" ]]; then
  for b in "$dir"/target/*; do "$PYTHON" "$ROOT/tools/pe/peinfo.py" "$b" | head -4; done
fi
cat <<NEXT

Next:
  1. Edit the goal paragraph in $dir/AGENTS.md.
  2. cd $dir && scripts/setup.sh --install
  3. Ask your agent to start with the re-start skill (it routes every step from there).
NEXT
