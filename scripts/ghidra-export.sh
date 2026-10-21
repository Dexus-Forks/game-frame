#!/usr/bin/env bash
# Analyse one or more binaries with Ghidra (no window) and export them as text an agent can
# search: decomp.c, functions.tsv, strings.txt, imports.txt, exports.txt per binary.
#
#   scripts/ghidra-export.sh [--project DIR] [--out DIR] BINARY [BINARY ...]
#
# Defaults: the Ghidra project in analysis/ghidra-project, the export in analysis/export/<binary>/.
# Import the game's executable and its own DLLs together, so calls between them are resolved.
# A large game takes 10-30 minutes; it prints only progress lines and writes the full log to
# analysis/headless.log. Set GHIDRA_CPUS (default 6) and MAXMEM (default 8G) to tune it.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
project="$ROOT/analysis/ghidra-project"; out="$ROOT/analysis/export"
while [[ "${1:-}" == --* ]]; do
  case "$1" in
    --project) project="$2"; shift 2 ;;
    --out) out="$2"; shift 2 ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
  esac
done
[[ $# -gt 0 ]] || { sed -n '2,12p' "$0"; exit 2; }
ghidra="$(find_ghidra)"; java="$(find_java21)"
[[ -n "$ghidra" ]] || { echo "Ghidra not found: run scripts/setup.sh, or set GHIDRA_INSTALL_DIR" >&2; exit 1; }
[[ -n "$java" ]] || { echo "JDK 21 not found: run scripts/setup.sh, or set JAVA_HOME" >&2; exit 1; }
bins=(); for b in "$@"; do bins+=("$(cd "$(dirname "$b")" && pwd)/$(basename "$b")"); done
mkdir -p "$project" "$out"
echo "Analysing ${#bins[@]} binaries into $project (log: $(dirname "$project")/headless.log)"
JAVA_HOME="$java" MAXMEM="${MAXMEM:-8G}" "$ghidra/support/analyzeHeadless" "$project" re \
  -overwrite -max-cpu "${GHIDRA_CPUS:-6}" -import "${bins[@]}" \
  -scriptPath "$ROOT/tools/ghidra" -postScript ExportAll.java "$out" \
  2>&1 | tee "$(dirname "$project")/headless.log" \
  | grep -E --line-buffered "EXPORT DONE|Analysis succeeded|ERROR.*Exception" || true
for b in "${bins[@]}"; do
  f="$out/$(basename "$b")/decomp.c"
  [[ -s "$f" ]] && echo "exported $(basename "$b"): $(grep -c '^// ==== ' "$f") functions in $f" \
    || { echo "no export for $(basename "$b"): see the log" >&2; exit 1; }
done
