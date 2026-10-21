#!/usr/bin/env bash
# Remake the terminal pictures in docs/assets/captures/ from GameFrame's own tools, run on
# the sample game. Nothing here touches a commercial game.
#
#   site/captures/make.sh
#
# Runs tests/run.sh twice (with and without --ghidra, about two minutes).
# Needs: tests/run.sh --ghidra run first, so the sample game, its ground truth, the port and
# the Ghidra export exist (examples/sample-game/build/, analysis/export/).
# Writes: docs/assets/captures/*.svg (see term_svg.py for the picture).
# It does not run the tests itself, and leaves the pictures as they are if a tool fails.
set -euo pipefail
. "$(dirname "$0")/../../scripts/lib.sh"
cd "$ROOT"
B=examples/sample-game/build
OUT=docs/assets/captures
SVG="$PYTHON site/captures/term_svg.py"
mkdir -p "$OUT"
[[ -f $B/game.exe && -d analysis/export/game.exe ]] || { echo "run tests/run.sh --ghidra first"; exit 1; }

# The test run: the first sections and the summary.
log="$(tests/run.sh 2>&1)"
{ echo '$ tests/run.sh'; echo "$log" | sed -n 1,12p; echo '...'; echo "$log" | grep -v '^$' | tail -4; } \
  | $SVG "tests/run.sh" "$OUT/run-tests.svg"

# The home page's hero: the checks that prove the port, from a full run.
full="$(tests/run.sh --ghidra 2>&1)"
{ echo '$ tests/run.sh --ghidra'; echo '...'; echo "$full" | grep -A3 '^Ground truth'; echo '...'
  echo "$full" | tail -1; } | $SVG "tests/run.sh" "$OUT/hero.svg"

# What is this binary?
pe="$("$PYTHON" tools/pe/peinfo.py $B/game.exe)"
{ echo "\$ python tools/pe/peinfo.py $B/game.exe"; echo "$pe" | sed -n 1,13p; } \
  | $SVG "peinfo.py" "$OUT/peinfo.svg"

# One function from the Ghidra export.
fn="$("$PYTHON" tools/triage/decomp_query.py --binary game.exe fn simulate)"
{ echo '$ python tools/triage/decomp_query.py --binary game.exe fn simulate'
  echo "$fn" | sed -n '1,5p;14,40p' | grep -v '^\s*/\* 0x' | sed '5a\
  ...'; } \
  | $SVG "decomp_query.py" "$OUT/decomp-query.svg"

# The port against the original, and a naive port that drifts (bitcompare then exits 1).
naive="$("$PYTHON" tools/verify/bitcompare.py f32 $B/truth/seed-1.f32 $B/naive/seed-1.f32 || true)"
{ echo "\$ cd $B"
  echo "\$ python ../../../tools/verify/bitcompare.py f32 truth/seed-1.f32 port/seed-1.f32"
  "$PYTHON" tools/verify/bitcompare.py f32 $B/truth/seed-1.f32 $B/port/seed-1.f32
  echo "\$ python ../../../tools/verify/seqdiff.py truth/seed-1.trace port/seed-1.trace"
  "$PYTHON" tools/verify/seqdiff.py $B/truth/seed-1.trace $B/port/seed-1.trace
  echo "\$ python ../../../tools/verify/bitcompare.py f32 truth/seed-1.f32 naive/seed-1.f32"
  echo "$naive" | head -4; echo '...'; echo "$naive" | tail -1; } \
  | $SVG "bitcompare.py and seqdiff.py" "$OUT/bitcompare.svg"

ls -l "$OUT"
