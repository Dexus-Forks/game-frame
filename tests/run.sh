#!/usr/bin/env bash
# Prove every tool in GameFrame's toolkit against the sample game. Run after changing any tool.
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
g="$(mktemp -d)/globals"; "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust-globals "$g" >/dev/null
expect "tdinfo writes Globals per unit at their DS offsets" "w.seek(0x4); w.u16(self.total);" cat "$g/sample.rs"
expect "tdinfo writes a procedure's locals at their BP offsets" "[bp+6] range: Word" "$PYTHON" tools/dos/tdinfo.py $D/sample-td.exe rust-locals rng_next
expect "tdinfo joins the units into Globals" "pub sample: sample::Sample," cat "$g/mod.rs"
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
expect "run_to carries on with the registers as they are (run_range resets the stack)" "ax=42 sp=1234" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe'); f = int(open('$D/symbols.tsv').read().split('field_read')[1].split()[1], 16)
s = e.new_segment(b'....' + bytes([42, 0]))
e.run_range(f'1000:{f:04x}', f'1000:{f:04x}', regs={'ds': s, 'si': 0, 'sp': 0x1234})
e.run_to(f'1000:{f + 3:04x}'); print(f'ax={e.reg(\"ax\")} sp={e.reg(\"sp\"):04x}')"
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
expect "call() sets general registers for routines that take them in registers" "0x4242" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
e = DosEmu('$D/sample.exe', stubs={'KERNEL.3': lambda m: m.set(ax=m.reg('cx'))})
print(hex(e.call('ASK_HOST', regs={'cx': 0x4242})))"
expect "an unhandled interrupt names itself" "unhandled interrupt 21h (AH=30h) at 1000:0094" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu
try: DosEmu('$D/sample.exe').call('DOS_VERSION')
except RuntimeError as e: print(e)"
expect "dos_emu runs real-mode MZ code at Ghidra's addresses (data after the code)" "True [722, 842, 312]" "$PYTHON" -c "
import sys, os; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu, Long
m = DosEmu('$D/sample-mz.exe'); ds = m.call('1000:0088')
last = max(int(l.split()[2], 16) for l in open('$D/symbols.tsv'))
m.call('1000:000f', Long(1234), regs={'ds': ds})
print(ds > 0x1000 + last // 16, [m.call('1000:0028', 1000, regs={'ds': ds}) for _ in range(3)])"

echo "Headless DOS environment (dos_env)"
printf 'HELLO DOS ENV\n' > $D/SAMPLE.TXT
expect "dos_env serves DOS file reads from a folder (names without case)" "14 b'HELLO DOS ENV'" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D'); print(e.call('READ_FILE'), e.read((0x1008, 0x90), 13))"
w="$(mktemp -d)"
expect "dos_env never writes the game folder: writes go to the overlay" "unchanged XYZ" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/emu'); from dos_emu import DosEmu; from dos_env import DosEnv
e = DosEmu('$D/sample.exe'); env = DosEnv(e, root='$D', overlay='$w')
e.write((0x1008, 0x90), b'XYZ'); e.call('READ_FILE') if False else None; e.call('WRITE_FILE')
print('unchanged' if open('$D/SAMPLE.TXT').read() == 'HELLO DOS ENV\\n' else 'CHANGED', open('$w/SAMPLE.TXT').read()[:3])"
rm -rf "$w"
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

echo "Fragments of a main loop (fragments.py)"
a() { printf '1000:%s' "$(awk -v n="$1" '$1 == n { sub(/^0x/, "", $3); print $3 }' $D/symbols.tsv)"; }
w="$(mktemp -d)"
expect "fragments captures fragments of a running main loop" "draw: 8 of 8 runs saved" "$PYTHON" tools/emu/fragments.py $D/sample.exe "$w" 8 \
  "draw=$(a draw)-$(a draw_even_end)|$(a draw_odd_end),heap=1" "event=$(a event)-$(a event_end),changed=a8" \
  "tail=$(a event_end)-$(a frame_top),every=2,max=3" --frame-top "$(a frame_top)" --start "$(a main_loop)" \
  --screen a0 --display a2 --heap a0,a4
expect "fragments keeps only runs that change a word, and one in N up to K" "event 2 tail 3" sh -c "
  echo event \$(ls '$w/event' | grep -c before.ds) tail \$(ls '$w/tail' | grep -c before.ds)"
expect "fragments saves state before and after: DS, screen, display, files, heap" "ds scr vga files heap 2 1" "$PYTHON" -c "
w = '$w/draw/00000'; r = lambda e: open(w + e, 'rb').read()
print(' '.join(k for k in ('ds', 'scr', 'vga', 'files') if r('.before.' + k) != r('.after.' + k)), 'heap', len(r('.before.heap')) // 65538, len(r('.after.heap')) // 65538)"
expect "fragments saves the open files with their positions" "5	1	sample.txt" cat "$w/draw/00000.after.files"
rm -rf "$w"

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
  g="$(mktemp -d)"; mkdir "$g/game"; cp $D/sample-mz.exe "$g/game/SAMPLEMZ.EXE"   # output stays outside the game folder it copies
  expect "dosbox-run runs a DOS program and reads its exit code (= dos_emu's first draw)" "exit code 23" \
    scripts/dosbox-run.sh --out "$g/out" --time 20 "$g/game" SAMPLEMZ.EXE
  if ! dosbox_has_debug_channel; then
    echo "  skip  dbx_control (DOSBox-X $(dosbox_version) has no debugger control channel; needs 2026.10+)"
  else
  expect "dbx_control drives the debugger: break at exit, read AL and the new seed" "AL=23 seed=3b032b05" "$PYTHON" - "$g" <<'PY'
import os, sys; sys.path.insert(0, 'tools/dos')
from dbx_control import Debugger
g = sys.argv[1]
with Debugger(port=0, timeout=30) as d:            # a free port: no stale DOSBox-X can connect
    d.launch([os.path.abspath('scripts/dosbox-run.sh'), '--out', g + '/dbx', '--time', '40', '--break', '--mcp', str(d.port), g + '/game', 'SAMPLEMZ.EXE'])
    d.wait_connected(); d.wait_break()          # DEBUGBOX: stopped at the first instruction
    d.exec('BPINT 21 4C'); d.run()
    regs = d.wait_break()                       # stopped at INT 21h AH=4Ch (exit)
    seed = int.from_bytes(d.memdump('DS', 0, 4), 'little')
    d.run()
print(f'AL={regs["STEP_AX"] & 0xFF} seed={seed:08x}')
PY
  fi
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

echo "Discs, file kinds, similarity, obfuscation, timestamps"
W="$(mktemp -d)"
"$PYTHON" tests/make_disc.py "$W"
expect "discinfo lists a data track and an audio track" "track  2  audio AUDIO       start 00:02:40  length 00:02:00" "$PYTHON" tools/disc/discinfo.py "$W/test.cue"
expect "discinfo reads the ISO 9660 creation date" "1995-11-03 14:30:00 UTC+01:00" "$PYTHON" tools/disc/discinfo.py "$W/test.iso"
expect "discinfo reads Joliet names and finds the hidden file" "Secret Data.dat" "$PYTHON" tools/disc/discinfo.py "$W/test.cue" --files
expect "extract-installer hands a CUE sheet to discinfo" "CD image -> discinfo" scripts/extract-installer.sh --detect "$W/test.cue"
"$PYTHON" tools/disc/discinfo.py "$W/test.cue" --extract "$W/x" --audio "$W/wav" >/dev/null
[[ "$(cat "$W/x/ReadMe File.txt")" == "hello from the disc" && $(wc -c < "$W/wav/track02.wav") -eq 352844 ]] \
  && ok "discinfo copies the files and writes the audio track as WAV" || bad "discinfo --extract --audio"
expect "filetypes names a PE executable" "executable (PE, Windows)" "$PYTHON" tools/triage/filetypes.py $B/game.exe
head -c 65536 /dev/urandom > "$W/noise.dat"
expect "filetypes flags a high-entropy file" "compressed or encrypted" "$PYTHON" tools/triage/filetypes.py "$W/noise.dat"
expect "similar scores a file against itself" "100" "$PYTHON" tools/triage/similar.py $B/game.exe $B/game.exe
"$PYTHON" - "$W" <<'PY'
import random, sys
d = sys.argv[1]; r = random.Random(7); data = bytes(r.randrange(256) for _ in range(100000))
open(f'{d}/v1.dat', 'wb').write(data); open(f'{d}/v2.dat', 'wb').write(data[:40000] + b'patch' * 20 + data[40000:])
sys.path.insert(0, 'tools/formats'); from xorscan import decode
open(f'{d}/hidden.txt', 'wb').write(decode(open('README.md', 'rb').read()[:4000], 'xor-key:1f2e3d4c'))
PY
s=$("$PYTHON" tools/triage/similar.py "$W/v1.dat" "$W/v2.dat" | awk '{print $1}')
(( s >= 90 && s < 100 )) && ok "similar sees a patched file as close but changed ($s)" || bad "similar on a patched file" "$s"
"$PYTHON" - "$W" <<'PY'
import random, struct, sys
d = sys.argv[1]; r = random.Random(5); mem = bytearray(r.randbytes(65536))
for i, gold in enumerate((500, 480, 480, 612)):
    for _ in range(200): mem[r.randrange(len(mem))] = r.randrange(256)
    struct.pack_into('<H', mem, 0x1235, gold); struct.pack_into('<I', mem, 0x4000, 0x21235)
    open(f'{d}/m{i}.bin', 'wb').write(mem)
PY
expect "memscan narrows dumps to the one address that holds the value" "0x00021235  u16  500 -> 480 -> 480 -> 612" "$PYTHON" tools/triage/memscan.py "$W/m0.bin:=500" "$W/m1.bin:=480" "$W/m2.bin:same" "$W/m3.bin:up" --base 0x20000
expect "memscan finds a pointer to the value" "0x00024000" "$PYTHON" tools/triage/memscan.py "$W/m3.bin" --pointers 0x21235 --base 0x20000
expect "xorscan finds a repeating XOR key" "xor-key:1f2e3d4c " "$PYTHON" tools/formats/xorscan.py "$W/hidden.txt" --top 1
expect "timestamps reads a FAT date" "dos       1995-11-03 14:30:10" "$PYTHON" tools/formats/timestamps.py 0x1F6373C5
expect "reader reads LEB128 and big-endian numbers" "624485 -123456 4660" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/formats'); from reader import Reader
r = Reader(bytes([0xe5, 0x8e, 0x26, 0xc0, 0xbb, 0x78, 0x12, 0x34])); print(r.uleb(), r.sleb(), r.u16be()); r.done()"
expect "discinfo --map finds data in a sector no file uses" "sectors 34-34 (1): ZIP archive" "$PYTHON" tools/disc/discinfo.py "$W/test.cue" --map
expect "discinfo finds a file the Joliet tree leaves out" "missing from the Joliet tree: NOJOLIET.TXT" "$PYTHON" tools/disc/discinfo.py "$W/test.cue"
expect "discinfo reads a PlayStation-style system area" "PlayStation licence text" "$PYTHON" tools/disc/discinfo.py "$W/xa.cue"
"$PYTHON" tools/disc/discinfo.py "$W/xa.cue" --extract "$W/xa" >/dev/null
[[ $(wc -c < "$W/xa/MOVIE.STR") -eq 4704 && -f "$W/xa/NOJOLIET.TXT" ]] \
  && ok "discinfo copies an XA Form 2 file as whole sectors, and files only ISO 9660 lists" || bad "discinfo XA extract"
rm -rf "$W"

echo "Constants, resources, stack strings, Flash, function matching"
W="$(mktemp -d)"
expect "constants finds the MSVC-style random generator in the engine" "MSVC rand() multiplier 214013" "$PYTHON" tools/triage/constants.py $B/engine.dll
printf 'int main(void){return 0;}\n' > "$W/m.c"
if zig cc -target x86_64-windows-gnu "$W/m.c" tests/fixtures/resources.rc -o "$W/res.exe" >/dev/null 2>&1; then
  expect "peres reads the version information" "OriginalFilename         SAMPLE.EXE" "$PYTHON" tools/pe/peres.py "$W/res.exe"
  expect "peres reads a string table" "17  'Insert disc 2'" "$PYTHON" tools/pe/peres.py "$W/res.exe" --strings
else bad "peres: zig could not compile tests/fixtures/resources.rc"; fi
if zig cc -target x86-windows-gnu -O1 -fno-sanitize=undefined tests/fixtures/stackstring.c -o "$W/ss.exe" >/dev/null 2>&1; then
  expect "stackstrings finds text built on the stack" "'SAVE'" "$PYTHON" tools/pe/stackstrings.py "$W/ss.exe"
else bad "stackstrings: zig could not compile tests/fixtures/stackstring.c"; fi
"$PYTHON" tests/make_swf.py "$W"
expect "swfinfo reads an LZMA SWF and sees ActionScript 3" "3 (DoABC: AVM2)" "$PYTHON" tools/formats/swfinfo.py "$W/game-zws.swf"
expect "swfinfo lists the classes linked to symbols" "GameMain" "$PYTHON" tools/formats/swfinfo.py "$W/game-cws.swf"
"$PYTHON" tools/formats/swfinfo.py "$W/projector.exe" --extract "$W/out.swf" >/dev/null
cmp -s "$W/out.swf" "$W/game-cws.swf" && ok "swfinfo extracts the movie from a projector" || bad "swfinfo --extract"
expect "filetypes names a compressed Flash movie" "Flash movie (SWF, zlib)" "$PYTHON" tools/triage/filetypes.py "$W/game-cws.swf"
"$PYTHON" - "$W" <<'PY'
import os, sys
d = sys.argv[1]
body = {'update': 'void FUN_{a}(int *p)\n{{\n  p[1] = p[1] + p[2] * 3;\n  if (p[1] > 100) {{\n    FUN_{b}(p, DAT_{c});\n  }}\n  return;\n}}\n',
        'spawn': 'void FUN_{b}(int *p, int n)\n{{\n  int i;\n  for (i = 0; i < n; i = i + 1) {{\n    p[i + 4] = i * 7 + 0x10;\n  }}\n  return;\n}}\n'}
for build, (a, b, c) in (('A', ('401000', '401100', '409000')), ('B', ('402000', '402300', '40a000'))):
    os.makedirs(f'{d}/{build}', exist_ok=True)
    with open(f'{d}/{build}/decomp.c', 'w') as f:
        f.write(f'// ==== {a} FUN_{a}\n' + body['update'].format(a=a, b=b, c=c))
        f.write(f'// ==== {b} FUN_{b}\n' + body['spawn'].format(b=b))
    open(f'{d}/{build}/functions.tsv', 'w').write(f'{a}\t60\tFUN_{a}\t\n{b}\t40\tFUN_{b}\t\n')
open(f'{d}/names.tsv', 'w').write('401000\tunit_update\tgrows by 3 a tick\n401100\tunit_spawn\t\n')
PY
"$PYTHON" tools/triage/match_functions.py "$W/A" "$W/B" --labels "$W/names.tsv" --out "$W/names-b.tsv" >/dev/null
expect "match_functions carries labels to the new build's addresses" "402300	unit_spawn	matched from A 401100 (exact 1.0)" cat "$W/names-b.tsv"
rm -rf "$W"

echo "Pictures, tables, signatures, pointers, markers, instructions"
W="$(mktemp -d)"
"$PYTHON" -c "
d = bytes((x ^ y) & 255 for y in range(200) for x in range(320)); open('$W/screen.bin', 'wb').write(d)"
"$PYTHON" tools/formats/rawimage.py "$W/screen.bin" "$W/screen.png" --mode linear8 >/dev/null
[[ "$(head -c 8 "$W/screen.png" | xxd -p)" == 89504e470d0a1a0a ]] && ok "rawimage writes a mode 13h screen as PNG" || bad "rawimage"
expect "rawimage decodes Mode X planes" "640 x 480, mode modex" "$PYTHON" tools/formats/rawimage.py "$W/screen.bin" "$W/x.png" --mode modex --scale 2
expect "binview draws a file as a picture" "512 x 512, 1 byte(s) per pixel" "$PYTHON" tools/triage/binview.py $B/game.exe "$W/bv.png" --layout hilbert
"$PYTHON" -c "
import math, os, struct
a = os.urandom(999) + b''.join(struct.pack('<h', round(math.sin(2 * math.pi * i / 1024) * 16384)) for i in range(1280))
open('$W/sine.bin', 'wb').write(a)"
expect "tables finds a sine table and its shape" "sine table: 1024 steps per circle, scale 16384, i16, nearest; 1024 entries match" "$PYTHON" tools/triage/tables.py "$W/sine.bin"
expect "tables sees the cosine table inside the sine table" "the cosine table is the same table from entry 256 on" "$PYTHON" tools/triage/tables.py "$W/sine.bin"
printf "140001000\ttree_update\n140001090\tsimulate\n" > "$W/names.tsv"
"$PYTHON" tools/triage/sigscan.py make $B/game.exe --names "$W/names.tsv" > "$W/game.sigs"
expect "sigscan makes signatures that wildcard addresses" "48 8D 15 ?? ?? ?? ??" cat "$W/game.sigs"
printf 'static int pad(int x){return x*7+1;}\nint first(int x){return pad(x)+pad(x+1);}\n' > "$W/pad.c"
cat "$W/pad.c" $S/game.c > "$W/game2.c"
if zig cc -target x86_64-windows-gnu -O2 "$W/game2.c" $B/engine.lib -o "$W/game2.exe" >/dev/null 2>&1; then
  expect "sigscan finds the named functions in a rebuilt game at new addresses" "simulate" "$PYTHON" tools/triage/sigscan.py scan "$W/game.sigs" "$W/game2.exe"
else bad "sigscan: zig could not build the second game"; fi
"$PYTHON" tests/make_omf.py "$W"
expect "omf lists a library's public names" "_puts                            _TEXT:0013" "$PYTHON" tools/dos/omf.py "$W/RT.LIB"
expect "omf turns a fixup into wildcards" "_puts: 55 8B EC FF 76 04 E8 ?? ??" "$PYTHON" tools/dos/omf.py "$W/RT.LIB" --sigs
"$PYTHON" tools/dos/omf.py "$W/RT.LIB" --sigs > "$W/rt.sigs"
expect "sigscan names a library function inside a DOS program" "_puts                            1000:0051" "$PYTHON" tools/triage/sigscan.py scan "$W/rt.sigs" "$W/GAME.EXE"
expect "insns lists the interrupts of a DOS program" "1000:0003  int 0x21" "$PYTHON" tools/triage/insns.py "$W/GAME.EXE" '^int '
"$PYTHON" - "$W" <<'PY'
import random, struct, sys
d = sys.argv[1]
for run, (w, p) in enumerate([(0x1001000, 0x1003000), (0x1005400, 0x1002200)]):
    r = random.Random(run)
    mem = bytearray(0x10000); heap = bytearray(0x10000)
    for i in range(0, 0x10000, 4):
        struct.pack_into('<I', mem, i, r.randrange(0, 0x3fffff)); struct.pack_into('<I', heap, i, r.randrange(0, 0x3fffff))
    struct.pack_into('<I', mem, 0x4000, w); struct.pack_into('<I', heap, w - 0x1000000 + 0x18, p)
    struct.pack_into('<I', heap, p - 0x1000000 + 0x24, 500)
    open(f'{d}/run{run}.bin', 'wb').write(mem + bytes(0x1000000 - 0x410000) + heap)
PY
expect "pointerpaths finds the chain that survives a restart" "[0x404000] -> +0x18 -> +0x24" "$PYTHON" tools/triage/pointerpaths.py "$W/run0.bin@0x400000:0x1003024" "$W/run1.bin@0x400000:0x1002224" --static 0x400000-0x40ffff
"$PYTHON" -c "
import struct; d = bytearray(64); struct.pack_into('<IH', d, 16, 1234, 77); open('$W/save.dat', 'wb').write(d)"
"$PYTHON" tools/formats/markers.py plant "$W/save.dat" "$W/save-m.dat" --from 16 --to 24 >/dev/null
"$PYTHON" -c "
m = bytearray(4096); m[100:104] = open('$W/save-m.dat', 'rb').read()[20:24]; open('$W/dump.bin', 'wb').write(m)"
expect "markers maps a file's field to its address in memory" "file 0x000014  marker 0xc0de0001  ->  0x20064" "$PYTHON" tools/formats/markers.py find "$W/save-m.dat.markers" "$W/dump.bin@0x20000"
expect "reader reads bit fields in both orders" "4 22 5 20" "$PYTHON" -c "
import sys; sys.path.insert(0, 'tools/formats'); from reader import Reader
a = Reader(bytes([0b10110100])); b = Reader(bytes([0b10110100])); print(a.bits(3), a.bits(5), b.bits(3, True), b.bits(5, True))"
rm -rf "$W"

echo "Ledger and coverage"
L="$(mktemp -d)"; mkdir -p "$L/src/build" "$L/export/game.exe"
printf '/// Original: `FUN_140001090`\nfn grow() {}\n// Deviation: the time comes in as an argument,\n// so tests can fix it.\nfn tick() {}\n// Not ported: the debug overlay.\n' > "$L/src/world.rs"
printf '# A helper. Original: `rng_next`\n' > "$L/src/tool.py"
echo "// Deviation: generated" > "$L/src/build/gen.rs"; echo "# Deviation notes" > "$L/src/NOTES.md"
printf '140001000\t40\trng_next\t\n140001090\t300\tsimulate\t\n140001200\t20\tFUN_140001200\t\n140001300\t5\tthunk_x\tthunk\n' > "$L/export/game.exe/functions.tsv"
expect "deviations joins a comment block and skips build/ and prose" "Deviation: 1, Not ported: 1." "$PYTHON" tools/triage/deviations.py "$L/src"
expect "deviations keeps the whole block" "argument, so tests can fix it." "$PYTHON" tools/triage/deviations.py "$L/src"
"$PYTHON" tools/triage/deviations.py --out "$L/DEV.md" "$L/src" >/dev/null
echo "// Deviation: new" >> "$L/src/tool.py"
expect "deviations --check finds a stale ledger" "out of date" "$PYTHON" tools/triage/deviations.py --out "$L/DEV.md" --check "$L/src"
expect "coverage counts addresses and names, not thunks" "2 of 3 functions cited (66.7 %), 340 of 360 bytes (94.4 % by size)" "$PYTHON" tools/triage/coverage.py --export "$L/export" "$L/src"
expect "coverage lists the largest uncited function" "140001200" "$PYTHON" tools/triage/coverage.py --export "$L/export" "$L/src"
rm -rf "$L"

echo "Project scaffold"
tmp="$(mktemp -d)/re-test"
scripts/new-project.sh "$tmp" "Test Game" $B >/dev/null 2>&1
[[ -f "$tmp/AGENTS.md" && -f "$tmp/README.md" && -f "$tmp/LICENSE-APACHE" && -f "$tmp/LICENSE-CC0-1.0" && ! -e "$tmp/LICENSE-CC-BY-4.0" && ! -e "$tmp/LICENSE-MIT" && -f "$tmp/docs/DOS.md" && -f "$tmp/docs/OUTPUTS.md" && -f "$tmp/docs/games/README.md" && -f "$tmp/docs/observe/inputs.md" && -f "$tmp/docs/working/agents.md" && -f "$tmp/docs/hardware/cpus.md" && -f "$tmp/.cargo/config.toml" && -f "$tmp/.claude/skills/re-start/SKILL.md" && -f "$tmp/.claude/agents/re-tracer.md" && -f "$tmp/.claude/settings.json" && -f "$tmp/scripts/hooks/gate.py" && -f "$tmp/target/game.exe" && -d "$tmp/.git" ]] \
  && grep -q "Test Game" "$tmp/AGENTS.md" && ! (cd "$tmp" && git ls-files | grep -q '^target/') \
  && ok "new-project creates a repository, keeps target/ out of git" || bad "new-project scaffold" "$(ls -la "$tmp" 2>&1)"
expect "gameframe works in a new project" "tools/pe/peinfo.py  (program)" "$tmp/gameframe" tool peinfo

echo "Gates (scripts/hooks/gate.py)"
out="$("$PYTHON" - <<'PY' 2>&1
import json, os, subprocess, tempfile
G = os.path.abspath('scripts/hooks/gate.py')
def run(mode, event):
    return subprocess.run(['python3', G, mode], input=json.dumps(event), capture_output=True, text=True).returncode
d = tempfile.mkdtemp()
git = lambda *a: subprocess.run(['git', *a], cwd=d, capture_output=True, check=True)
git('init', '-q'); os.makedirs(f'{d}/target'); os.makedirs(f'{d}/docs')
open(f'{d}/target/GAME.EXE', 'w').write('x'); git('add', '-f', 'target/GAME.EXE')
c = {'cwd': d, 'tool_input': {'command': 'git commit -m x'}}
assert run('commit', c) == 2, 'a commit of target/GAME.EXE is blocked'
git('rm', '-q', '--cached', 'target/GAME.EXE')
open(f'{d}/docs/a.md', 'w').write('FUN_1400be770 reads +0x44 (observed).\n'); git('add', 'docs/a.md')
assert run('commit', c) == 0, 'a commit of prose is allowed'
assert run('commit', dict(c, agent_type='gameframe:re-tracer')) == 2, 're-tracer may not commit'
assert run('commit', dict(c, agent_type='re-lander')) == 0, 're-lander may commit'
p = {'cwd': d, 'tool_input': {'command': 'git push'}}
assert run('push', dict(p, agent_type='re-porter')) == 2 and run('push', p) == 0, 'push gate'
pseudo = 'undefined8 uVar1;\nuVar1 = CONCAT44(a, b);\n'
w = lambda path: {'cwd': d, 'tool_input': {'file_path': f'{d}/{path}', 'content': pseudo}}
assert run('write', w('docs/f.md')) == 2 and run('write', w('analysis/f.c')) == 0, 'write gate'
s = subprocess.run(['python3', G, 'session'], input=json.dumps({'cwd': d}), capture_output=True, text=True)
assert json.loads(s.stdout)['hookSpecificOutput']['hookEventName'] == 'SessionStart', 'session context'
print('commit, push, write and session gates')
PY
)"
[[ $? == 0 ]] && ok "the gates block game material, decompiled code and non-lander commits ($out)" || bad "gates" "$out"

echo "Claude Code plugin"
out="$("$PYTHON" - <<'PY' 2>&1
import json, re, pathlib
plugin = json.loads(pathlib.Path('.claude-plugin/plugin.json').read_text())
market = json.loads(pathlib.Path('.claude-plugin/marketplace.json').read_text())
assert [p['name'] for p in market['plugins']] == [plugin['name']], 'the marketplace lists the plugin'
assert pathlib.Path(plugin['skills'], 're-start', 'SKILL.md').is_file(), 'the plugin finds the skills'
assert pathlib.Path('.claude/agents').resolve() == pathlib.Path('agents').resolve(), '.claude/agents links to agents/'
skills = {p.parent.name for p in pathlib.Path('.claude/skills').glob('*/SKILL.md')}
start = pathlib.Path('.claude/skills/re-start/SKILL.md').read_text()
agents = sorted(pathlib.Path('agents').glob('*.md'))
assert agents, 'no agents'
for a in agents:
    text = a.read_text()
    front = re.match(r'---\n(.*?)\n---\n', text, re.S).group(1)
    assert re.search(rf'^name: {a.stem}$', front, re.M), f'{a}: name matches the file'
    assert re.search(r'^description: .{40,}', front, re.M), f'{a}: has a description'
    desc = re.search(r'^description: (.*)$', front, re.M).group(1)
    assert desc.startswith('"') or ': ' not in desc, f'{a}: an unquoted description with ": " is not valid YAML'
    listed = re.findall(r'^  - (re-[a-z]+)$', front.split('skills:')[1], re.M) if '\nskills:' in '\n' + front else []
    assert listed, f'{a}: no skills to preload'
    for s in listed:
        assert s in skills, f'{a}: no skill {s}'
    body_skills = re.findall(r'`(re-[a-z]+)`', text.split('Your skills are preloaded:')[1].split('\n')[0])
    assert body_skills == listed, f'{a}: the body names {body_skills}, the front matter preloads {listed}'
    for n in re.findall(r'`(re-[a-z]+er)`', text):
        assert pathlib.Path('agents', n + '.md').is_file(), f'{a}: hands off to a missing agent {n}'
    assert f'Stage: {a.stem}' in text and '## Hand-off' in text, f'{a}: ends with a hand-off'
    assert f'| `{a.stem}` |' in start, f're-start does not list {a.stem}'
print(len(agents), 'agents')
PY
)"
[[ $? == 0 ]] && ok "the plugin lists the skills and agents; every agent loads real skills and hands off to a real agent ($out)" || bad "plugin and agents" "$out"
if command -v claude >/dev/null; then
  out="$(claude plugin validate --strict . 2>&1)"
  [[ $? == 0 ]] && ok "claude plugin validate --strict passes" || bad "claude plugin validate" "$out"
else
  echo "  skip  claude plugin validate (no claude command)"
fi

echo "Help and discovery (gameframe)"
expect "scripts/env.sh sets GAMEFRAME_ROOT and puts gameframe on PATH" "$(pwd)/gameframe" \
  bash -c 'cd / && . "'"$(pwd)"'/scripts/env.sh" >/dev/null && command -v gameframe'
G=./gameframe
expect "gameframe prints its help with no command" "find WORD" $G
expect "gameframe steps reads the loop from re-start" "re-emulate: What exactly does this original function return" $G steps
expect "gameframe skills lists every skill" "$(ls -d .claude/skills/re-*/ | wc -l | tr -d ' ')" sh -c "$G skills --json | '$PYTHON' -c 'import json,sys; print(len(json.load(sys.stdin)))'"
expect "gameframe skill prints one section" "## Pitfalls" $G skill emulate --section pitfalls
expect "gameframe skill names a missing section" "has no section matching" $G skill re-trace --section nothing-here
expect "gameframe tools shows each tool's own first line" "\"summary\": \"$(sed -n 2p tools/verify/seqdiff.py | sed 's/^"""//')\"" $G tools --json
expect "every tool and script carries its own help" "missing: []" sh -c "$G tools --json | '$PYTHON' -c 'import json,sys; print(\"missing:\", [t[\"name\"] for t in json.load(sys.stdin) if not t[\"summary\"]])'"
expect "gameframe tool prints the docstring unchanged" "Usage: decomp_query.py [--binary NAME]" $G tool decomp_query
expect "gameframe tool finds a script by name" "scripts/ghidra-export.sh  (script)" $G tool ghidra-export
expect "gameframe tool suggests a close name" "Did you mean: peinfo" $G tool peinf
expect "gameframe run gives the same output as the tool" "$("$PYTHON" tools/pe/peinfo.py $B/game.exe | head -1)" $G run peinfo $B/game.exe
expect "gameframe run passes the tool's exit code" "rc 1" sh -c "$G run bitcompare f32 $B/truth/seed-1.f32 $B/naive/seed-1.f32 >/dev/null; echo rc \$?"
expect "gameframe run refuses a library and says why" "a library to import" $G run pe_emu
expect "gameframe find names the skills and tools for a word" "tool vtable" $G find vtable

echo "Crates"
if command -v cargo >/dev/null; then
  out="$(cd crates && cargo test --release 2>&1)"
  [[ $? == 0 ]] && ok "the crates' tests pass ($(echo "$out" | grep -o '[0-9]* passed' | awk '{s+=$1} END {print s}') tests)" \
    || bad "crates" "$(echo "$out" | grep -E 'FAILED|panicked|error' | head -8)"
else
  echo "  skip  the crates (cargo is not installed)"
fi

echo "Documentation site"
svg="$(mktemp)"
printf '$ tests/run.sh\n  ok    a check <&>\n' | "$PYTHON" site/captures/term_svg.py "run" "$svg" \
  && grep -q '&lt;&amp;&gt;' "$svg" && grep -q '>ok</tspan>' "$svg" \
  && ok "term_svg draws terminal text as SVG, escaped and coloured" || bad "term_svg"
rm -f "$svg"
if [[ -d site/node_modules ]]; then
  out="$(npm --prefix site run build 2>&1)"
  [[ $? == 0 && -f site/.vitepress/dist/games/interaction.html && -f site/.vitepress/dist/skills/re-observe.html ]] \
    && ok "the site builds from docs/ and the skills, with no dead links" || bad "site build" "$(echo "$out" | grep -iv '^\s*at ' | tail -8)"
  dist=site/.vitepress/dist
  grep -q '^- \[.*\](/games/interaction.md): ' $dist/llms.txt 2>/dev/null \
    && grep -q '^# ' $dist/games/interaction.md && ! head -1 $dist/games/interaction.md | grep -q '^---' \
    && grep -q '](/observe/inputs.md)' $dist/games/interaction.md \
    && grep -q 'Skill `re-observe`' $dist/skills/re-observe.md && grep -q 'Source: /skills/re-observe' $dist/llms-full.txt \
    && ok "the site writes llms.txt, llms-full.txt and a Markdown copy of each page" || bad "llms.txt and .md copies" "$(head -12 $dist/llms.txt 2>&1)"
else
  echo "  skip  the site (run: npm --prefix site install)"
fi

if [[ "${1:-}" == --ghidra ]]; then
  echo "Ghidra"
  scripts/ghidra-export.sh $B/game.exe $B/engine.dll >/dev/null 2>&1 && ok "ghidra-export exports both binaries" || bad "ghidra-export"
  expect "decomp_query shows a function" "rng_next" "$PYTHON" tools/triage/decomp_query.py --binary game.exe fn simulate
  expect "decomp_query finds string users" "main" "$PYTHON" tools/triage/decomp_query.py --binary game.exe strings "after 10"
  C="$(mktemp -d)"; printf '// Original: `simulate`\n' > "$C/sim.rs"
  expect "coverage reads a real export" "1 of" "$PYTHON" tools/triage/coverage.py --binary game.exe "$C"
  rm -rf "$C"
  scripts/ghidra-run.sh game.exe decompile analysis/notes/test.txt 0x140001090 >/dev/null 2>&1
  expect "ghidra-run decompiles by address" "FUNCTION 140001090 simulate" cat analysis/notes/test.txt
  scripts/ghidra-run.sh game.exe callargs analysis/notes/callargs.tsv rng_seed printf >/dev/null 2>&1
  expect "ghidra-run callargs resolves a constant argument" "0x4d2 (1234)" cat analysis/notes/callargs.tsv
  expect "ghidra-run callargs resolves a string argument" '"after 10 s: %f' cat analysis/notes/callargs.tsv
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
