---
name: re-inventory
description: First look at a game's files (Windows PE, or DOS MZ/NE/LE executables) - which binaries hold the game's own code, which engine and middleware it uses, whether it is native, managed, scripted or packed, which data containers and save files exist - and record it in docs/FINDINGS.md. Use at the start of a project, or when a new binary or file type turns up.
---

# Inventory a game

The goal is a short, evidenced map of the game before any decompiling: what to analyse, with
which tool, and what to leave alone. A wrong choice here (decompiling a packed binary, or
analysing the C++ runtime of a Unity game instead of its C# code) wastes days.

## 0. Get the files out of their installer

A purchased game often arrives as an installer (GOG's Inno Setup `.exe`, a Steam depot, an
NSIS or InstallShield setup, an `.msi`, a `.dmg` or `.pkg`, GOG's Linux `.sh`). Never run it:
extract it.

```bash
scripts/extract-installer.sh --detect "~/Downloads/setup_game.exe"     # what it is, which tool
scripts/extract-installer.sh "~/Downloads/setup_game.exe" target/gog
python3 tools/triage/compare_trees.py target/game target/gog         # same version?
python3 tools/triage/compare_trees.py target/game /dev/null --hashes > docs/HASHES.txt
```

- A small macOS `.app` named after the game is often only GOG Galaxy's web installer: no game.
- Store releases bundle extras worth reading: DOSBox or ScummVM configs (the CPU speed and
  sound hardware the publisher tuned the game for), manuals and cheat sheets as PDF, saves.
- When you have two releases, compare them before choosing one. Identical executables mean
  every address and finding carries over; record the hashes (they are not game material) so
  every later finding names the exact version it was made on.

## 1. List the files

```bash
find target -maxdepth 3 -type f | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -30   # by extension
find target -maxdepth 3 -type f -size +5M -exec ls -lh {} \; | sort -k5 -h               # the big ones
```

Group them: executables and libraries (`.exe .dll .so .dylib`), data containers (large files
with custom extensions: `.pak .pkg .dat .arc .bin .wad .pck .assets`), configuration (`.ini
.cfg .xml .json .lua`), saves (often in the user's Documents or AppData, not the game folder).

## 2. Look at every binary

```bash
for b in target/*.exe target/*.dll; do .venv/bin/python tools/pe/peinfo.py "$b"; echo; done
```

For each, note: 32- or 64-bit, EXE or DLL, PDB path (a leaked build path names the engine and
project), compiler (a Rich header means MSVC), imports, exports, RTTI count (C++ class names are
recoverable), engine fingerprints, and packer warnings.

Decide which binaries are **the game's own code**: usually the EXE and any DLL with a game-like
name and many exports or imports between them. Leave out operating-system DLLs, the C/C++
runtime (`msvcp140.dll`, `vcruntime140.dll`), and known middleware (FMOD, Wwise, Bink, PhysX,
Havok, Steam/GOG SDKs): document what they are and move on.

Engines often ship a runtime DLL that exports C++ class names (`?Rand@Random@System@@...`). Its
exports are free, accurate names: decompile it together with the EXE.

### DOS and 16-bit programs

`file` saying "MS-DOS executable" (or peinfo refusing the file) means a DOS program. Use:

```bash
for b in target/*.EXE target/*.exe target/*.OVL; do python3 tools/dos/dosinfo.py "$b"; echo; done
```

It names the format (`MZ` real mode, `NE` 16-bit protected mode, `LE`/`LX` 32-bit extender),
gives Ghidra's selector for every NE segment, lists imports and exports, and fingerprints the
compiler, extender, packer and sound libraries. Two lines matter most:

- **`debug symbols: Turbo Debugger`**: the original names of every procedure, global and type
  are in the file. Run `python3 tools/dos/tdinfo.py BINARY summary` now; see docs/DOS.md
  section 3. This can save weeks.
- **A packer** (LZEXE, PKLITE, EXEPACK, DIET): unpack before anything else.

Launchers are common: a small MZ that runs the real program (`tyrian.exe` running
`file0001.exe`). The DOSBox-X file log (`scripts/dosbox-run.sh --fileio`) shows which program
and which data files are opened, in order. Runtime files (`RTM.EXE`, `DPMI16BI.OVL`,
`DOS4GW.EXE`) are the compiler's DOS extender, not game code. The whole DOS route is in
docs/DOS.md.

## 3. Choose the route

| What peinfo shows | What it means | Route |
|---|---|---|
| Native x64/x86, MSVC or MinGW, no packer | Ordinary compiled C/C++ | This harness: `re-ghidra`, `re-trace`, `re-emulate` |
| `mscoree.dll` / `_CorExeMain`, or `Assembly-CSharp.dll` | .NET managed code (XNA, MonoGame, Unity Mono) | Decompile C# with ILSpy or dnSpyEx, or REA `inspect_managed_artifact`; the game logic is in the managed DLLs |
| `GameAssembly.dll` + `global-metadata.dat` | Unity IL2CPP: C# compiled to C++ | Recover names with Il2CppDumper (or Cpp2IL), then Ghidra with the generated script |
| `UE4`/`UnrealEngine` strings, `.pak` files | Unreal Engine | Most logic is in the EXE plus Blueprint bytecode in `.uasset`; tools such as FModel and UE4SS read the assets |
| `lua_pcall`, `.lua`/`.luac` files | Lua scripting on a native engine | Decompile the scripts (unluac, luadec) for game rules; the native side is the engine |
| GameMaker / RPG Maker / Godot strings | A known engine with its own format | Use the engine's community tools for the data; the native binary is the stock runtime |
| dosinfo: `MZ` or `NE`, Borland/Microsoft/Watcom runtime | 16-bit DOS or Windows 3.x code | This harness's DOS route: `dosinfo.py`, `tdinfo.py`, Ghidra, `dos_emu`, DOSBox-X (docs/DOS.md) |
| dosinfo: `LE`/`LX` (DOS/4GW, PMODE/W) | 32-bit DOS-extender program | Ghidra with an LE/LX loader extension; run it in DOSBox-X |
| High-entropy executable section, `.vmp0`, `.themida`, UPX | Packed or protected | Stop. Unpack first (UPX: `upx -d`), or work from a memory dump with the user's agreement; Denuvo and VMProtect are out of scope for static analysis |

For an Android or iOS game, an Electron or browser game, or an archive you cannot identify, use
REA's routing (`re-rea`): it has dedicated tools for each.

## 4. Data containers

For each large file type:

```bash
xxd target/data.pak | head -8                         # magic number, header fields, offsets
.venv/bin/python -c "import math,collections,sys;d=open(sys.argv[1],'rb').read(1<<20);c=collections.Counter(d);print(-sum(n/len(d)*math.log2(n/len(d)) for n in c.values()))" target/data.pak
```

Entropy near 8.0 means compressed or encrypted; 4 to 6 means plain structured data. Look for
known signatures in the first bytes and inside: `PK\x03\x04` (ZIP), `\x78\x9c`/`\x78\xda`
(zlib), `\x28\xb5\x2f\xfd` (zstd), `\x04\x22\x4d\x18` (LZ4 frame), `RIFF`, `OggS`, `DDS `,
`\x89PNG`. Find the code that opens the file (search its name or extension in the strings), then
decode it with `re-format`.

## 5. Record

Fill `docs/FINDINGS.md` (correct any first-look numbers the tools overturn, such as an NE
segment with no bytes in the file, which is uninitialised data): the binaries table (file, role, architecture, compiler, notes), the
engine and middleware with the evidence for each (the import or string that showed it), the
data containers, and the open questions. Add the first lines to `docs/CHECKLIST.md`.
