#!/usr/bin/env bash
# Start a website for a project: a VitePress site that logs the work and shows what it found.
#
#   scripts/new-site.sh NAME "Title" ["One-sentence description"] [--single] [--manual] [--repo URL]
#   ./gameframe run new-site NAME "Title" ...          the same, from a project made with GameFrame
#
# Writes sites/NAME/ in the project (with --single, site/ instead, for a project that will only
# ever have one site):
#   .vitepress/config.mts   the title, the navigation, the diary in the sidebar, local search
#   .vitepress/theme/       the default theme with the project's colours: set them from the game
#   public/                 the diary's pictures and diagrams (WebP and SVG), from the first entry
#   index.md                the front page; replace each TODO(site) line
#   findings.md, status.md  docs/FINDINGS.md and docs/CHECKLIST.md, shown with @include (one copy)
#   diary/                  the index of dated entries (YYYY-MM-DD-slug.md); write the first one
#   package.json, .gitignore
# and .github/workflows/sites.yml, which builds every site and publishes it on GitHub Pages,
# when the project does not have it yet. With --manual the workflow runs only when started by
# hand (the Actions tab or `gh workflow run sites.yml`), not on every push, to save Actions
# minutes. The re-site skill says what goes on each page.
# tools/site/check.py checks a site: includes, game material, diary entries, leftover TODOs.
# It writes nothing if the folder exists. It does not run npm: run `npm install` in the folder.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
single=0; manual=0; repo=""; args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --single) single=1 ;;
    --manual) manual=1 ;;
    --repo) [[ $# -ge 2 ]] || { echo "--repo needs a URL" >&2; exit 2; }; repo="$2"; shift ;;
    *) args+=("$1") ;;
  esac
  shift
done
set -- ${args[@]+"${args[@]}"}
[[ $# -ge 2 ]] || { sed -n '2,21p' "$0"; exit 2; }
name="$1"; title="$2"; description="${3:-A log of reverse engineering $2: what was tried, what was found, and how each finding is known.}"
[[ "$name" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { echo "NAME is lower case letters, digits and hyphens: $name" >&2; exit 2; }
if [[ $single == 1 ]]; then dir="$PROJECT/site"; docs="../docs"; else dir="$PROJECT/sites/$name"; docs="../../docs"; fi
[[ -e "$dir" ]] && { echo "$dir exists: edit that site instead" >&2; exit 1; }
if [[ -z "$repo" ]]; then
  repo="$(git -C "$PROJECT" remote get-url origin 2>/dev/null || true)"
  repo="${repo%.git}"; repo="${repo/git@github.com:/https://github.com/}"
fi
T="$ROOT/templates/site"
mkdir -p "$dir/.vitepress/theme" "$dir/diary" "$dir/public"
# Values go into double-quoted strings (YAML and JavaScript), then through sed.
esc() { printf '%s' "$1" | sed -e 's/[\\"]/\\&/g' -e 's/[\/&|]/\\&/g'; }
for f in .vitepress/config.mts index.md findings.md status.md diary/index.md diary/posts.data.mts package.json; do
  sed -e "s|SITE_TITLE|$(esc "$title")|g" -e "s|SITE_DESCRIPTION|$(esc "$description")|g" \
      -e "s|SITE_NAME|$name|g" -e "s|REPO_URL|$(esc "$repo")|g" -e "s|DOCS/|$docs/|g" "$T/$f" > "$dir/$f"
done
cp "$T/gitignore" "$dir/.gitignore"
cp "$T/.vitepress/theme/index.ts" "$T/.vitepress/theme/custom.css" "$dir/.vitepress/theme/"
wf="$PROJECT/.github/workflows/sites.yml"
if [[ ! -f "$wf" ]]; then
  mkdir -p "$(dirname "$wf")"
  if [[ $manual == 1 ]]; then   # no push trigger: it runs when started by hand
    sed -e '/^  push:/,/^    paths:/d' -e 's/^on:$/# Manual only: start it from the Actions tab or with `gh workflow run sites.yml`.\non:/' "$T/sites.yml" > "$wf"
  else
    cp "$T/sites.yml" "$wf"
  fi
  echo "Wrote ${wf#$PROJECT/}"
elif [[ $manual == 1 ]]; then
  echo "${wf#$PROJECT/} exists: remove its push trigger by hand to run it manually only" >&2
fi
echo "Wrote ${dir#$PROJECT/}/. Next: replace the TODO(site) lines in index.md, write the first diary entry,"
echo "then: cd ${dir#$PROJECT/} && npm install && npm run dev"
