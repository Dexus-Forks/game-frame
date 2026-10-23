#!/usr/bin/env bash
# Build the sample game for Windows x64 (an EXE and its engine DLL, and a 32-bit DLL) with Zig's C compiler, from
# any host. The output goes to build/ (ignored by git). Needs `zig` (scripts/setup.sh installs it).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p build
zig cc -target x86_64-windows-gnu -O2 -shared engine.c -o build/engine.dll
zig cc -target x86_64-windows-gnu -O2 game.c build/engine.lib -o build/game.exe
# A 32-bit DLL for the x86 emulator (tools/emu/pe32_emu.py).
zig cc -target x86-windows-gnu -O1 -shared mixer32.c -o build/mixer32.dll
python3 make_data.py build/sample.dat
echo "built: $(ls build | tr '\n' ' ')"
