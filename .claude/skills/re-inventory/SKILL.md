---
name: re-inventory
description: First look at a game's files - which binaries hold the game's own code, which engine and middleware it uses, whether it is native, managed, scripted or packed, which data containers and save files exist - and record it in docs/FINDINGS.md. Use at the start of a project, or when a new binary or file type turns up.
---

# Inventory a game

The goal is a short, evidenced map of the game before any decompiling: what to analyse, with
which tool, and what to leave alone. A wrong choice here (decompiling a packed binary, or
analysing the C++ runtime of a Unity game instead of its C# code) wastes days.

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

## 3. Choose the route

| What peinfo shows | What it means | Route |
|---|---|---|
| Native x64/x86, MSVC or MinGW, no packer | Ordinary compiled C/C++ | This harness: `re-ghidra`, `re-trace`, `re-emulate` |
| `mscoree.dll` / `_CorExeMain`, or `Assembly-CSharp.dll` | .NET managed code (XNA, MonoGame, Unity Mono) | Decompile C# with ILSpy or dnSpyEx, or REA `inspect_managed_artifact`; the game logic is in the managed DLLs |
| `GameAssembly.dll` + `global-metadata.dat` | Unity IL2CPP: C# compiled to C++ | Recover names with Il2CppDumper (or Cpp2IL), then Ghidra with the generated script |
| `UE4`/`UnrealEngine` strings, `.pak` files | Unreal Engine | Most logic is in the EXE plus Blueprint bytecode in `.uasset`; tools such as FModel and UE4SS read the assets |
| `lua_pcall`, `.lua`/`.luac` files | Lua scripting on a native engine | Decompile the scripts (unluac, luadec) for game rules; the native side is the engine |
| GameMaker / RPG Maker / Godot strings | A known engine with its own format | Use the engine's community tools for the data; the native binary is the stock runtime |
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

Fill `docs/FINDINGS.md`: the binaries table (file, role, architecture, compiler, notes), the
engine and middleware with the evidence for each (the import or string that showed it), the
data containers, and the open questions. Add the first lines to `docs/CHECKLIST.md`.
