#!/usr/bin/env bash
# Build the sample game for Windows x64 (an EXE and its engine DLL) with Zig's C compiler, from
# any host. The output goes to build/ (ignored by git). Needs `zig` (scripts/setup.sh installs it).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p build
zig cc -target x86_64-windows-gnu -O2 -shared engine.c -o build/engine.dll
zig cc -target x86_64-windows-gnu -O2 game.c build/engine.lib -o build/game.exe
python3 make_data.py build/sample.dat
echo "built: $(ls build | tr '\n' ' ')"
