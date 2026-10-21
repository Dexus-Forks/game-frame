#!/usr/bin/env bash
# Prove every tool in the harness against the sample game. Run after changing any tool.
#
#   tests/run.sh            the fast checks (about 20 s): PE tools, emulator, call traces,
#                           ground truth against the port, format validation, project scaffold
#   tests/run.sh --ghidra   also the Ghidra export and decomp_query (about a minute more)
#
# Needs scripts/setup.sh --install (and zig, to build the sample game).
set -uo pipefail
. "$(dirname "$0")/../scripts/lib.sh"
cd "$ROOT"
S=examples/sample-game; B=$S/build
pass=0; fail=0
ok() { echo "  ok    $1"; pass=$((pass + 1)); }
bad() { echo "  FAIL  $1"; [[ -n "${2:-}" ]] && echo "$2" | sed 's/^/        /'; fail=$((fail + 1)); }
expect() { # name expected-substring command...
  local name="$1" want="$2"; shift 2
  local out; out="$("$@" 2>&1)"
  if [[ "$out" == *"$want"* ]]; then ok "$name"; else bad "$name" "$out"; fi
}

echo "Building the sample game"
$S/build.sh >/dev/null 2>&1 && ok "sample game builds (zig cc, Windows x64)" || { bad "sample game builds: is zig installed?"; exit 1; }

echo "PE tools"
expect "peinfo reads the EXE" "EXE, x64 (64-bit), image base 0x140000000" "$PYTHON" tools/pe/peinfo.py $B/game.exe
expect "peinfo lists the engine DLL import" "engine.dll" "$PYTHON" tools/pe/peinfo.py $B/game.exe
table=$("$PYTHON" -c "import pefile;p=pefile.PE('$B/game.exe');print(hex(p.OPTIONAL_HEADER.ImageBase+[e.address for e in p.DIRECTORY_ENTRY_EXPORT.symbols if e.name==b'growth_table'][0]))")
expect "readdata reads a float table" "0.05000000074505806, 0.125, 0.3333333134651184, 1.0" "$PYTHON" tools/pe/readdata.py $B/game.exe f32 "$table" --count 4
vt=$("$PYTHON" -c "import pefile;p=pefile.PE('$B/game.exe');print(hex(p.OPTIONAL_HEADER.ImageBase+[e.address for e in p.DIRECTORY_ENTRY_EXPORT.symbols if e.name==b'tree_vtbl'][0]))")
expect "vtable lists two slots" "+0x008" "$PYTHON" tools/pe/vtable.py $B/game.exe "$vt"

echo "Emulator"
expect "RNG matches MSVC rand() from seed 1" "[41, 18467, 6334]" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/emu')
from pe_emu import Emu
emu = Emu('examples/sample-game/build/game.exe', ['examples/sample-game/build/engine.dll'])
r = emu.malloc(4); emu.call(emu.export('engine.dll', 'rng_seed'), r, 1)
print([emu.call(emu.export('engine.dll', 'rng_next'), r) for _ in range(3)])
PY
expect "data loader runs on emulated memory" "3 (0.5, 1.25, 2.0)" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/emu')
from pe_emu import Emu
emu = Emu('examples/sample-game/build/game.exe', ['examples/sample-game/build/engine.dll'])
data = open('examples/sample-game/build/sample.dat', 'rb').read(); w = emu.malloc(64)
n = emu.call_i32(emu.export('game.exe', 'load_records'), emu.bytes_in(data), len(data), w)
print(n, emu.f32s(w, 3))
PY
cp $B/engine.dll $B/copy.dll
expect "overlapping DLLs are relocated" "relocated True" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/emu')
from pe_emu import Emu
b = 'examples/sample-game/build/'
emu = Emu(b + 'game.exe', [b + 'engine.dll', b + 'copy.dll'])
m = emu.module('copy.dll'); r = emu.malloc(4)
emu.call(emu.export('copy.dll', 'rng_seed'), r, 1)
ok = emu.call(emu.export('copy.dll', 'rng_next'), r) == 41
print('relocated', m.base != m.preferred and ok)
PY
expect "an unhandled import names itself" "unhandled import" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/emu')
from pe_emu import Emu
b = 'examples/sample-game/build/'
emu = Emu(b + 'game.exe', [b + 'engine.dll'], stubs={'malloc': None})
try:
    emu.call(emu.export('engine.dll', 'mem_alloc'), 16)
except RuntimeError as e:
    print(e)
PY

echo "Ground truth and the port"
"$PYTHON" $S/ground_truth.py $B/truth >/dev/null && "$PYTHON" $S/port.py $B/port >/dev/null \
  && "$PYTHON" $S/port.py --naive $B/naive >/dev/null
same=0; drift=0; draws=0
for s in $(seq 1 20); do
  "$PYTHON" tools/verify/bitcompare.py f32 $B/truth/seed-$s.f32 $B/port/seed-$s.f32 >/dev/null && same=$((same + 1))
  "$PYTHON" tools/verify/bitcompare.py f32 $B/truth/seed-$s.f32 $B/naive/seed-$s.f32 >/dev/null || drift=$((drift + 1))
  "$PYTHON" tools/verify/seqdiff.py $B/truth/seed-$s.trace $B/port/seed-$s.trace >/dev/null && draws=$((draws + 1))
done
[[ $same == 20 ]] && ok "the port matches the original bit for bit (20 seeds x 600 steps)" || bad "port matches on $same of 20 seeds"
[[ $draws == 20 ]] && ok "the port draws random numbers in the original's order (20 seeds)" || bad "draw order matches on $draws of 20 seeds"
[[ $drift == 20 ]] && ok "bitcompare catches the naive double-precision port (20 seeds)" || bad "naive port caught on only $drift of 20 seeds"
expect "call traces name the call site" "game.exe+0x" head -1 $B/truth/seed-1.trace

echo "Formats"
expect "the sample decoder consumes every byte" "1 of 1 files decode completely" "$PYTHON" tools/formats/validate.py $S/decode_sample.py "$B/*.dat"
printf 'SMPL\x01\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x3f\x03oakEXTRA' > $B/broken.dat
expect "validate fails a file with unread bytes" "5 bytes left unread" "$PYTHON" tools/formats/validate.py $S/decode_sample.py $B/broken.dat
rm -f $B/broken.dat

echo "Project scaffold"
tmp="$(mktemp -d)/re-test"
scripts/new-project.sh "$tmp" "Test Game" $B >/dev/null 2>&1
[[ -f "$tmp/AGENTS.md" && -f "$tmp/.claude/skills/re-start/SKILL.md" && -f "$tmp/target/game.exe" && -d "$tmp/.git" ]] \
  && grep -q "Test Game" "$tmp/AGENTS.md" && ! (cd "$tmp" && git ls-files | grep -q '^target/') \
  && ok "new-project creates a repository, keeps target/ out of git" || bad "new-project scaffold" "$(ls -la "$tmp" 2>&1)"

if [[ "${1:-}" == --ghidra ]]; then
  echo "Ghidra"
  scripts/ghidra-export.sh $B/game.exe $B/engine.dll >/dev/null 2>&1 && ok "ghidra-export exports both binaries" || bad "ghidra-export"
  expect "decomp_query shows a function" "rng_next" "$PYTHON" tools/triage/decomp_query.py --binary game.exe fn simulate
  expect "decomp_query finds string users" "main" "$PYTHON" tools/triage/decomp_query.py --binary game.exe strings "after 10"
  scripts/ghidra-run.sh game.exe decompile analysis/notes/test.txt 0x140001090 >/dev/null 2>&1
  expect "ghidra-run decompiles by address" "FUNCTION 140001090 simulate" cat analysis/notes/test.txt
fi

echo
echo "$pass passed, $fail failed"
[[ $fail == 0 ]]
