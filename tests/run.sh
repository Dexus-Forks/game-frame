#!/usr/bin/env bash
# Prove every tool in the harness against the sample game. Run after changing any tool.
#
#   tests/run.sh            the fast checks (about 20 s): PE tools, emulator, call traces,
#                           ground truth against the port, format validation, the 16-bit DOS
#                           tools and emulator, project scaffold
#   tests/run.sh --ghidra   also the Ghidra exports (Windows x64 and 16-bit NE), labels and
#                           decomp_query (about a minute more)
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

echo "16-bit DOS tools (examples/dos-sample)"
D=examples/dos-sample/build
python3 examples/dos-sample/make_exe.py $D >/dev/null 2>&1 && ok "DOS sample builds (zig, .code16; our own NE and MZ linker)" || bad "DOS sample builds"
expect "dosinfo reads the NE header" "NE: module 'DOSSAMPL'" "$PYTHON" tools/dos/dosinfo.py $D/sample.exe
expect "dosinfo gives Ghidra's selector per segment" "2  1008  file" "$PYTHON" tools/dos/dosinfo.py $D/sample.exe
expect "dosinfo lists imports by ordinal" "KERNEL.3" "$PYTHON" tools/dos/dosinfo.py $D/sample.exe --imports
expect "dosinfo lists exports at seg:off" "1000:0028  RNG_NEXT" "$PYTHON" tools/dos/dosinfo.py $D/sample.exe --exports
expect "dosinfo finds Pascal strings with their address" "1008:0011  [12] 'DOS SAMPLE 1'" "$PYTHON" tools/dos/dosinfo.py $D/sample.exe --pstrings 5
expect "dosinfo reads a real-mode MZ" "3 relocations" "$PYTHON" tools/dos/dosinfo.py $D/sample-mz.exe
expect "dosinfo finds Turbo Debugger symbols" "debug symbols: Turbo Debugger" "$PYTHON" tools/dos/dosinfo.py $D/sample-td.exe
expect "readseg reads a typed constant" "1008:0000  4660" "$PYTHON" tools/dos/readseg.py $D/sample.exe u32 DAT_1008_0000
expect "readseg reads a Pascal string" "'DOS SAMPLE 1'" "$PYTHON" tools/dos/readseg.py $D/sample.exe pstr 1008:0010
expect "readseg decodes a Turbo Pascal Real" "[1.0, -2.5, 0.0]" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/dos')
from readseg import real48
print([real48(b) for b in (bytes([129, 0, 0, 0, 0, 0]), bytes([130, 0, 0, 0, 0, 0xA0]), bytes(6))])
PY
expect "tdinfo lists procedures with sizes and parameters" "param     [bp+6] range: Word" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe procs rng_next
expect "tdinfo lays out a record" "+0x0004  total: Word  (2)" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe types
expect "tdinfo writes labels" "1008:0000	RandSeed" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe labels
expect "tdinfo maps code to source lines" "1000:0028	SAMPLE.S:11" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe lines
expect "tdinfo writes a Rust reader for a record" "seed: r.i32()?," "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust SampleState
expect "tdinfo writes the Rust Pascal reader and writer" "pub struct PascalWriter" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust-reader
expect "tdinfo writes record writers" "w.u16(self.total);" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust SampleState
expect "tdinfo writes Globals at their DS offsets" "w.seek(0x4); w.u16(self.total);" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust-globals
"$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe labels > $D/labels.tsv
expect "nedis applies relocations and names far calls" "lcall   KERNEL.3" "$PYTHON" tools/dos/nedis.py $D/sample.exe 1000:008c 6 --labels $D/labels.tsv
expect "nedis names data in DS" "[RandSeed]" "$PYTHON" tools/dos/nedis.py $D/sample.exe 1000:0028 8 --labels $D/labels.tsv
fr=$(awk '$1=="field_read"{print $3}' $D/symbols.tsv)
expect "nedis leaves register-relative operands alone ([si + 4] is not Total)" "[si + 4]" "$PYTHON" tools/dos/nedis.py $D/sample.exe "1000:${fr#0x}" 4 --labels $D/labels.tsv

echo "16-bit emulator"
expect "dos_emu runs the NE random generator (matches the reference)" "True [722, 842, 312, 2, 309]" "$PYTHON" - <<'PY'
import sys; sys.path.insert(0, 'tools/emu')
from dos_emu import DosEmu, Long
ref, seed = [], 1234
for _ in range(5):
    seed = (seed * 0x08088405 + 1) & 0xFFFFFFFF
    ref.append(((seed >> 16) * 1000) >> 16)
emu = DosEmu('examples/dos-sample/build/sample.exe')
emu.call('RNG_SEED', Long(1234))
got = [emu.call('1000:0028', 1000) for _ in range(5)]
print(got == ref, got)
PY
expect "dos_emu applies selector relocations (Ghidra's numbering)" "0x1008" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
print(hex(DosEmu('$D/sample.exe').call('GET_DATA_SEG')))"
expect "dos_emu gives scripts whole segments" "1020 104" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe'); s = e.new_segment(b'hello'); print(f'{s:04x}', e.call('CHECKSUM', (s, 0), 1))"
expect "dos_emu runs a fragment between two addresses" "ax=42" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe'); f = int(open('$D/symbols.tsv').read().split('field_read')[1].split()[1], 16)
s = e.new_segment(b'....' + bytes([42, 0]))
e.run_range(f'1000:{f:04x}', f'1000:{f + 3:04x}', regs={'ds': s, 'si': 0}); print(f'ax={e.reg(\"ax\")}')"
expect "dos_emu passes far pointers" "532" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe'); print(e.call('CHECKSUM', e.bytes_in(b'hello'), 5))"
expect "dos_emu traces random draws with call sites" "rng_next(50)	1000:005d" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu, Long
e = DosEmu('$D/sample.exe'); e.call('RNG_SEED', Long(99)); e.trace({'RNG_NEXT': 'rng_next'}, args=1)
e.call('SUM_DRAWS', 4, 50); print(e.traces[0])"
expect "an unhandled import names itself" "unhandled import KERNEL.3 called from 1000:0091" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
try: DosEmu('$D/sample.exe').call('ASK_HOST')
except RuntimeError as e: print(e)"
expect "import stubs and interrupt handlers run" "0xa03 5" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe', stubs={'KERNEL.3': lambda m: m.set(ax=0x0a03)}, interrupts={0x21: lambda m: m.set(ax=5)})
print(hex(e.call('ASK_HOST')), e.call('DOS_VERSION'))"
expect "an unhandled interrupt names itself" "unhandled interrupt 21h (AH=30h) at 1000:0094" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
try: DosEmu('$D/sample.exe').call('DOS_VERSION')
except RuntimeError as e: print(e)"
expect "dos_emu runs real-mode MZ code at Ghidra's addresses (data after the code)" "True [722, 842, 312]" "$PYTHON" -c "
import sys, os; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu, Long
m = DosEmu('$D/sample-mz.exe'); ds = m.call('1000:0088')
code_paras = -(-(int(open('$D/symbols.tsv').read().split('get_key')[1].split()[1], 16) + 3) // 16)
m.call('1000:000f', Long(1234), regs={'ds': ds})
print(ds == 0x1000 + code_paras, [m.call('1000:0028', 1000, regs={'ds': ds}) for _ in range(3)])"

echo "Headless DOS environment (dos_env)"
printf 'HELLO DOS ENV\n' > $D/SAMPLE.TXT
expect "dos_env serves DOS file reads from a folder (names without case)" "14 b'HELLO DOS ENV'" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D'); print(e.call('READ_FILE'), e.read((0x1008, 0x90), 13))"
expect "dos_env delivers timer interrupts to the program's own INT 8 handler" "3 3" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D'); print(e.call('WAIT_TICKS'), env.irq0_count)"
expect "dos_env types queued keys and stops when none is left" "0x1c0d the program waits for a key" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D'); env.keys = [0x1C0D]; k = e.call('GET_KEY')
try: e.call('GET_KEY')
except RuntimeError as x: print(hex(k), x)"
expect "dos_env's DPMI gives selectors at a base, and the constant selectors point at VGA" "a0000 b8123" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D'); s = e.new_selector(base=0xB8123)
print(hex(e.linear(e.new_selector(base=0xA0000)))[2:], hex(e.linear(s))[2:])"

echo "DOSBox-X runtime"
log="$(mktemp)"
cat > "$log" <<'LOG'
      10 DEBUG FILES:file open command 2 file C:\GAME\DATA.DAT
      11 DEBUG FILES:Reading 4 bytes from DATA.DAT
      12 DEBUG FILES:Reading 2 bytes from DATA.DAT
      13 DEBUG FILES:Reading 1 bytes from DATA.DAT
      14 DEBUG FILES:Reading 2 bytes from DATA.DAT
      15 DEBUG FILES:Reading 1 bytes from DATA.DAT
      16 DEBUG FILES:Reading 2 bytes from DATA.DAT
      17 DEBUG FILES:Reading 1 bytes from DATA.DAT
      18 DEBUG FILES:Seeking to 100 bytes from position type (0) in DATA.DAT
      19 DEBUG FILES:Reading 16 bytes from DATA.DAT
LOG
expect "fileio_trace groups a table of records" "0x000004: [2, 1] x 3 (stride 3)" "$PYTHON" tools/dos/fileio_trace.py "$log"
expect "fileio_trace follows seeks" "0x000064: 16" "$PYTHON" tools/dos/fileio_trace.py "$log" --file data.dat
rm -f "$log"
if command -v dosbox-x >/dev/null; then
  g="$(mktemp -d)"; cp $D/sample-mz.exe "$g/SAMPLEMZ.EXE"
  expect "dosbox-run runs a DOS program and reads its exit code (= dos_emu's first draw)" "exit code 23" \
    scripts/dosbox-run.sh --out "$g/out" --time 20 "$g" SAMPLEMZ.EXE
  expect "dbx_control drives the debugger: break at exit, read AL and the new seed" "AL=23 seed=3b032b05" "$PYTHON" - "$g" <<'PY'
import os, sys; sys.path.insert(0, 'tools/dos')
from dbx_control import Debugger
g = sys.argv[1]
with Debugger(port=58993, timeout=30) as d:
    d.launch([os.path.abspath('scripts/dosbox-run.sh'), '--out', g + '/dbx', '--time', '40', '--break', '--mcp', '58993', g, 'SAMPLEMZ.EXE'])
    d.wait_connected(); d.wait_break()          # DEBUGBOX: stopped at the first instruction
    d.exec('BPINT 21 4C'); d.run()
    regs = d.wait_break()                       # stopped at INT 21h AH=4Ch (exit)
    seed = int.from_bytes(d.memdump('DS', 0, 4), 'little')
    d.run()
print(f'AL={regs["STEP_AX"] & 0xFF} seed={seed:08x}')
PY
  rm -rf "$g"
else
  echo "  skip  dosbox-run (dosbox-x is not installed: scripts/setup.sh --install)"
fi

echo "Installers and releases"
I="$(mktemp -d)"
cp $B/game.exe "$I/setup.exe" && printf 'Inno Setup Setup Data (5.6.2)' >> "$I/setup.exe"
expect "extract-installer recognises an Inno Setup installer" "Inno Setup -> innoextract" scripts/extract-installer.sh --detect "$I/setup.exe"
mkdir -p "$I/web.app/Contents/MacOS" && : > "$I/web.app/Contents/MacOS/GalaxyWebInstaller"
expect "extract-installer flags a web-installer app" "probably no game files inside" scripts/extract-installer.sh --detect "$I/web.app"
mkdir -p "$I/src/data" && echo level > "$I/src/data/LEVEL1.DAT" && cp $S/../dos-sample/build/sample-mz.exe "$I/src/GAME.EXE"
(cd "$I/src" && zip -qr ../game.zip .)
printf '#!/bin/sh\n# MojoSetup installer\nexit 0\n' > "$I/game.sh" && cat "$I/game.zip" >> "$I/game.sh"
expect "extract-installer unpacks a zip" "extracted into" scripts/extract-installer.sh "$I/game.zip" "$I/a"
expect "extract-installer unpacks a GOG-style .sh (zip after a script)" "GOG Linux installer" scripts/extract-installer.sh "$I/game.sh" "$I/b"
echo changed > "$I/b/data/LEVEL1.DAT"
expect "compare_trees finds the changed file, ignoring case" "DIFFERENT  DATA/LEVEL1.DAT" "$PYTHON" tools/triage/compare_trees.py "$I/a" "$I/b"
expect "compare_trees counts identical files" "1 identical, 1 different" "$PYTHON" tools/triage/compare_trees.py "$I/a" "$I/b"
rm -rf "$I"

echo "Project scaffold"
tmp="$(mktemp -d)/re-test"
scripts/new-project.sh "$tmp" "Test Game" $B >/dev/null 2>&1
[[ -f "$tmp/AGENTS.md" && -f "$tmp/docs/DOS.md" && -f "$tmp/.cargo/config.toml" && -f "$tmp/.claude/skills/re-start/SKILL.md" && -f "$tmp/target/game.exe" && -d "$tmp/.git" ]] \
  && grep -q "Test Game" "$tmp/AGENTS.md" && ! (cd "$tmp" && git ls-files | grep -q '^target/') \
  && ok "new-project creates a repository, keeps target/ out of git" || bad "new-project scaffold" "$(ls -la "$tmp" 2>&1)"

if [[ "${1:-}" == --ghidra ]]; then
  echo "Ghidra"
  scripts/ghidra-export.sh $B/game.exe $B/engine.dll >/dev/null 2>&1 && ok "ghidra-export exports both binaries" || bad "ghidra-export"
  expect "decomp_query shows a function" "rng_next" "$PYTHON" tools/triage/decomp_query.py --binary game.exe fn simulate
  expect "decomp_query finds string users" "main" "$PYTHON" tools/triage/decomp_query.py --binary game.exe strings "after 10"
  scripts/ghidra-run.sh game.exe decompile analysis/notes/test.txt 0x140001090 >/dev/null 2>&1
  expect "ghidra-run decompiles by address" "FUNCTION 140001090 simulate" cat analysis/notes/test.txt
  "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe labels > analysis/notes/dos-labels.tsv
  scripts/ghidra-export.sh --project analysis/ghidra-dos --out analysis/export-dos --labels analysis/notes/dos-labels.tsv \
    $D/sample-td.exe $D/sample-mz.exe >/dev/null 2>&1 && ok "ghidra-export exports a 16-bit NE and an MZ" || bad "ghidra-export (16-bit)"
  expect "Ghidra numbers NE segments 1000, 1008" "1000:0028" cat analysis/export-dos/sample-td.exe/functions.tsv
  expect "ApplyLabels creates a function Ghidra missed" "1000:0092	5	dos_version" cat analysis/export-dos/sample-td.exe/functions.tsv
  expect "decomp_query reads seg:off exports with labels" "RandSeed" "$PYTHON" tools/triage/decomp_query.py --export analysis/export-dos --binary sample-td.exe fn 1000:0028
  expect "decomp_query lists callees by seg:off" "1000:0028" "$PYTHON" tools/triage/decomp_query.py --export analysis/export-dos --binary sample-td.exe callees sum_draws
fi

echo
echo "$pass passed, $fail failed"
[[ $fail == 0 ]]
