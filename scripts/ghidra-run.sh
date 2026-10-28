#!/usr/bin/env bash
# Run one of the tools/ghidra scripts on a binary in an existing, analysed Ghidra project,
# without analysing it again (about a minute of start-up, so batch addresses in one run).
#
#   scripts/ghidra-run.sh BINARY SCRIPT OUTPUT [ARGS ...]
#
#   scripts/ghidra-run.sh game.exe decompile analysis/notes/rng.txt 0x140001000 0x140001080
#   scripts/ghidra-run.sh game.exe named     analysis/notes/random.txt Random Rand
#   scripts/ghidra-run.sh game.exe refs      analysis/notes/refs.txt 0x140009000
#   scripts/ghidra-run.sh game.exe vtable    analysis/notes/vt.txt Tree 0x140008000 16
#   scripts/ghidra-run.sh game.exe labels    analysis/notes/labels.txt labels/names.tsv
#   scripts/ghidra-run.sh game.exe callargs  analysis/notes/sounds.tsv PlaySound rng_next
#   scripts/ghidra-run.sh GAME.EXE callargs analysis/notes/srand.tsv regs=watcom srand   # register arguments too
#   scripts/ghidra-run.sh game.bin blocks    analysis/notes/blocks.txt derived/loadmap.tsv
#   scripts/ghidra-run.sh game.exe disasm    analysis/notes/rng.asm 0x140001000 0x140001200-0x140001240
#   scripts/ghidra-run.sh GAME.EXE fill-gaps analysis/notes/fill.txt analysis/notes/starts.txt [EXPORT_DIR]
#
# SCRIPT is decompile (ExportAddressBatch: also creates functions Ghidra missed, such as
# callbacks reached only through tables), named (ExportNamedFunctions), refs (ExportAddressRefs:
# every reference to an address, with the referring function decompiled) or vtable
# (ExportVtableSlots: LABEL 0xTABLE [SLOTS] ...) or labels (ApplyLabels: name functions and
# globals from a TSV, creating missing functions; it saves the project, so export again after
# it with scripts/ghidra-export.sh --labels, or re-run decompile), callargs (ExportCallArgs:
# every call to the named functions with the constant arguments at each call) or blocks
# (AddMemoryBlocks: add raw pieces of memory from a load map; it saves the project, so
# analyse again) or disasm (ExportDisasm: the instructions of the functions containing the
# addresses, of START-END ranges decoded in order, or of every function with no addresses) or
# fill-gaps (the likely starts that tools/triage/exportcov.py --starts wrote, one address a
# line, 0x... or seg:off for 16-bit code: makes a function at each, as decompile does, writes the report to OUTPUT, then exports
# the binary again into EXPORT_DIR, default analysis/export, so decomp.c, functions.tsv and
# bodies.tsv hold the new functions; a start that fails is reported and the rest go on, and a
# start it refuses (nothing refers to it and no ret or padding comes before it) is counted and
# named apart from the ones it made; run exportcov again after it to see what is left). Use --project DIR for another project. Ghidra locks a project: run one job
# per project at a time.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
project="$PROJECT/analysis/ghidra-project"
[[ "${1:-}" == --project ]] && { project="$2"; shift 2; }
[[ $# -ge 3 ]] || { sed -n '2,34p' "$0"; exit 2; }
binary="$(basename "$1")"; script="$2"; output="$3"; shift 3
case "$script" in
  decompile|fill-gaps) java_script=ExportAddressBatch.java ;;
  named) java_script=ExportNamedFunctions.java ;;
  refs) java_script=ExportAddressRefs.java ;;
  vtable) java_script=ExportVtableSlots.java ;;
  labels) java_script=ApplyLabels.java ;;
  callargs) java_script=ExportCallArgs.java ;;
  blocks) java_script=AddMemoryBlocks.java ;;
  disasm) java_script=ExportDisasm.java ;;
  *.java) java_script="$script" ;;
  *) echo "unknown script $script" >&2; exit 2 ;;
esac
ghidra="$(find_ghidra)"; java="$(find_java21)"
[[ -n "$ghidra" && -n "$java" ]] || { echo "Ghidra or JDK 21 missing: run scripts/setup.sh" >&2; exit 1; }
mkdir -p "$(dirname "$output")"
output="$(cd "$(dirname "$output")" && pwd)/$(basename "$output")"
if [[ "$script" == fill-gaps ]]; then
  [[ $# -ge 1 && -f "$1" ]] || { echo "fill-gaps needs the starts file exportcov --starts wrote" >&2; exit 2; }
  starts=($(grep -oE '^\s*(0x[0-9a-fA-F]+|[0-9a-fA-F]{4}:[0-9a-fA-F]{4})' "$1" | tr -d ' \t' || true))
  export_dir="${2:-$PROJECT/analysis/export}"
  [[ ${#starts[@]} -gt 0 ]] || { echo "no addresses in $1: nothing to fill" >&2; exit 1; }
  set -- "${starts[@]}"
fi
JAVA_HOME="$java" "$ghidra/support/analyzeHeadless" "$(ghidra_project_dir "$project")" re -process "$binary" -noanalysis \
  -scriptPath "$ROOT/tools/ghidra" -postScript "$java_script" "$output" "$@" 2>&1 \
  | grep -E "written to|ERROR|Exception" || true
[[ -s "$output" ]] || { echo "no output: is $binary in $project?" >&2; exit 1; }
if [[ "$script" == fill-gaps ]]; then
  # A start is made when the report names its function's entry; "FUNCTION <none>" is a start
  # ExportAddressBatch refused (nothing refers to it, no ret or padding before it).
  made="$(grep -c '^===== ADDRESS .* FUNCTION [0-9a-fA-F]' "$output" || true)"
  refused="$(grep '^===== ADDRESS .* FUNCTION <none>' "$output" | awk '{ print $3 }' || true)"
  failed="$(grep -c '^===== ADDRESS .* ERROR' "$output" || true)"
  echo "fill-gaps: $made of ${#starts[@]} starts decompiled, $failed failed, $(grep -c . <<< "$refused" || true) refused; exporting $binary again"
  [[ -z "$refused" ]] || echo "fill-gaps: refused (not a likely start; give the start of the function around it):" $refused
  mkdir -p "$export_dir"; export_dir="$(cd "$export_dir" && pwd)"
  JAVA_HOME="$java" "$ghidra/support/analyzeHeadless" "$(ghidra_project_dir "$project")" re -process "$binary" -noanalysis \
    -scriptPath "$ROOT/tools/ghidra" -postScript ExportAll.java "$export_dir" 2>&1 | grep -E "EXPORT DONE|ERROR|Exception" || true
fi
echo "$output"
