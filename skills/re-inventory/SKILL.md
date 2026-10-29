---
name: re-inventory
description: First look at a game's files (Windows PE, or DOS MZ/NE/LE executables) - which binaries hold the game's own code, which engine and middleware it uses, whether it is native, managed, scripted or packed, which data containers and save files exist - and record it in docs/FINDINGS.md; then read the manual and the program's long strings into docs/lore.md, which says what each system is for. Use at the start of a project, or when a new binary or file type turns up.
---

# Inventory a game

Before any decompiling, make a short map of the game with evidence. It says what to analyse,
with which tool, and what to leave alone. A wrong choice here (decompiling a packed binary, or
analysing the C++ runtime of a Unity game instead of its C# code) wastes days.

## 0. Get the files out of their installer

A purchased game often arrives as an installer (GOG's Inno Setup `.exe`, a Steam depot, an
NSIS or InstallShield setup, an `.msi`, a `.dmg` or `.pkg`, GOG's Linux `.sh`) or on a CD (an
image as `.cue` and `.bin`, or `.iso`). Never run an installer: extract it.

`./gameframe game add ID FILE...` does it for you. It copies each disc image or installer into
the GameFrame home's `games/ID/media/`, records its size and SHA-256 in `games/ID/game.json`,
and extracts the first one into `games/ID/game/`. The project's `media/` and `game/` are links
to those folders. Give `--version NAME` for a second release or a patch, so each keeps its own
folder in `media/`. The commands below do the same steps by hand.

```bash
scripts/extract-installer.sh --detect "~/Downloads/setup_game.exe"     # what it is, which tool
scripts/extract-installer.sh "~/Downloads/setup_game.exe" derived/gog
python3 tools/triage/compare_trees.py game derived/gog         # same version?
python3 tools/triage/compare_trees.py game /dev/null --hashes > docs/HASHES.txt
python3 tools/triage/fixity.py bag game derived/bags/game-as-bought --info "External-Description=GOG 1.2"   # fixity
python3 tools/disc/discinfo.py media/GAME.CUE --files          # a CD: tracks, file systems, dates
python3 tools/triage/similar.py game derived/gog             # what changed between two releases
```

- A CD can hold more than its files. Audio tracks are often the game's music, which the port
  must play. A Mode 2 data track can hold video and sound in Form 2 sectors, which a cooked
  `.iso` loses. A hybrid disc can hold a Mac volume (HFS) beside the PC one. Read
  `docs/media/discs.md` before you choose which image to work from.

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

```bash
python3 tools/triage/filetypes.py target --summary      # kinds of file, by their bytes
python3 tools/triage/filetypes.py target                # each file: kind, entropy, a note
```

Group them: executables and libraries (`.exe .dll .so .dylib`), data containers (large files
with custom extensions: `.pak .pkg .dat .arc .bin .wad .pck .assets`), configuration (`.ini
.cfg .xml .json .lua`), saves (often in the user's Documents or AppData, not the game folder).

## 2. Look at every binary

```bash
for b in game/*.exe game/*.dll; do .venv/bin/python tools/pe/peinfo.py "$b"; echo; done
```

For each, note: 32- or 64-bit, EXE or DLL, PDB path (a leaked build path names the engine and
project), compiler (a Rich header means MSVC), imports, exports, RTTI count (C++ class names are
recoverable), engine fingerprints, and packer warnings.

Decide which binaries are **the game's own code**: usually the EXE and any DLL with a game-like
name and many exports or imports between them. Leave out operating-system DLLs, the C/C++
runtime (`msvcp140.dll`, `vcruntime140.dll`), and known middleware (FMOD, Wwise, Bink, PhysX,
Havok, Steam/GOG SDKs): document what they are and move on.

```bash
.venv/bin/python tools/pe/peres.py game/GAME.EXE            # version information, resources
python3 tools/triage/constants.py game/GAME.EXE             # random generators, CRCs, hashes, LZSS
.venv/bin/python tools/pe/stackstrings.py game/GAME.EXE     # text built on the stack
python3 tools/formats/swfinfo.py game/game.exe              # a Flash projector, or an SWF
python3 tools/triage/binview.py game/GAME.EXE derived/game.png --layout hilbert   # its layout, as a picture
```

For a DOS game, name the compiler's runtime before reading any code: build signatures from the
same compiler's libraries with `tools/dos/omf.py LIB --sigs`, then
`tools/triage/sigscan.py scan SIGS GAME.EXE --labels derived/runtime.tsv`
(`docs/COMPILED-CODE.md`, "Naming library code by signature").

The version information names the product and the build. The constants name the algorithms
inside and give their addresses. A Flash or Shockwave game needs a different route
(`docs/engines/flash-and-shockwave.md`).

Engines often ship a runtime DLL that exports C++ class names (`?Rand@Random@System@@...`). Its
exports are free, accurate names: decompile it together with the EXE.

### DOS and 16-bit programs

`file` saying "MS-DOS executable" (or peinfo refusing the file) means a DOS program. Use:

```bash
for b in game/*.EXE game/*.exe game/*.OVL; do python3 tools/dos/dosinfo.py "$b"; echo; done
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
| Native x64/x86, MSVC or MinGW, no packer | Ordinary compiled C/C++ | GameFrame: `re-ghidra`, `re-trace`, `re-emulate` |
| `mscoree.dll` / `_CorExeMain`, or `Assembly-CSharp.dll` | .NET managed code (XNA, MonoGame, Unity Mono) | Decompile C# with ILSpy or dnSpyEx, or REA `inspect_managed_artifact`; the game logic is in the managed DLLs |
| `GameAssembly.dll` + `global-metadata.dat` | Unity IL2CPP: C# compiled to C++ | Recover names with Il2CppDumper (or Cpp2IL), then Ghidra with the generated script |
| `UE4`/`UnrealEngine` strings, `.pak` files | Unreal Engine | Most logic is in the EXE plus Blueprint bytecode in `.uasset`; tools such as FModel and UE4SS read the assets |
| `lua_pcall`, `.lua`/`.luac` files | Lua scripting on a native engine | Decompile the scripts (unluac, luadec) for game rules; the native side is the engine |
| GameMaker / RPG Maker / Godot strings | A known engine with its own format | Use the engine's community tools for the data; the native binary is the stock runtime |
| dosinfo: `MZ` or `NE`, Borland/Microsoft/Watcom runtime | 16-bit DOS or Windows 3.x code | GameFrame's DOS route: `dosinfo.py`, `tdinfo.py`, Ghidra, `dos_emu`, DOSBox-X (docs/DOS.md); a Delphi program (`TPF0` resources): `dfm.py`, `delphi1.py` (docs/engines/delphi.md) |
| dosinfo: `LE`/`LX` (DOS/4GW, PMODE/W) | 32-bit DOS-extender program | Ghidra with an LE/LX loader extension; run it in DOSBox-X |
| A ROM or disc image for a console (`.nes`, `.sfc`, `.md`, `.z64`, `.bin`/`.cue`, `.iso`) | Console code at fixed addresses, often banked, no OS | The console route: `docs/consoles/` (platforms, tools, routes to a port) |
| An engine's archives or DLLs (`.WAD`, `.PAK`, `.GRP`, `.u`, `.MPQ`, `glide2x.dll`, `Engine.dll`) | A known engine or library | Read `docs/engines/recognising.md`; use the engine's documented formats and released source where it exists |
| High-entropy executable section, `.vmp0`, `.themida`, UPX | Packed or protected | Stop. Unpack first (UPX: `upx -d`), or work from a memory dump with the user's agreement; Denuvo and VMProtect are out of scope for static analysis |

For an Android or iOS game, an Electron or browser game, or an archive you cannot identify, use
REA's routing (`re-rea`): it has dedicated tools for each.

## 4. Data containers

For each large file type:

```bash
xxd game/data.pak | head -8                          # magic number, header fields, offsets
python3 tools/triage/filetypes.py game/data.pak --blocks 65536   # entropy, block by block
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

Then run the catalogue, the first resource viewer (`re-browse`), and copy its summary line
into `docs/CHECKLIST.md` as the starting format coverage:

```bash
python3 tools/formats/catalog.py game derived/catalog
```

Also record the hardware the game was written for: the minimum CPU, the video modes, the 3D
APIs, the sound devices and the input devices it supports (`docs/hardware/`). Its setup program
and its readme list them.

When the first inventory and game classification are in place, run `re-profile` to turn the
evidence into a quick start and identify any high-impact questions for the user.

### Every edition, and the reference

Before deep work, find every edition of the game and record them in `docs/versions.md` (the
project template; copy `templates/project/docs/versions.md` from GameFrame if the project is
older):

1. **Read this copy's identity** first:
   `.venv/bin/python tools/triage/buildid.py game/GAME --name "First edition" --write docs/versions.md`.
   It reads the disc's mastering date, each executable's link date and VERSIONINFO, version
   strings in the code, and the version lines of the readme and setup files.
2. **Is it the latest official version?** Search for later official releases: the
   publisher's or developer's own editions, patches and updates, regional and language
   releases, "Gold" or "GOTY" re-releases, and digital re-releases (GOG, Steam). Check
   MobyGames (the title's releases and patches), PCGamingWiki ("Essential improvements",
   "Patches", "Availability"), the publisher's and developer's old sites and FTP patch pages
   through the Wayback Machine, archive.org items for the title, the GOG and Steam pages, and
   magazine cover-disc listings. Fan patches, cracks and mods are context, recorded apart, and
   never the reference. Compare what you find with the build identity, and answer in
   `docs/versions.md`: is this the latest official build, and if not, which is and where it is
   documented. Record each source with its URL and the date you checked it. Ask the user
   before downloading anything. If a later official version exists, suggest moving the
   reference to it.
3. **List the editions**: the first release, regional releases with other languages, "Gold"
   and budget re-releases, official patches and updates, store builds, ports and demos, with
   the date of each. Look in the readme and manual, the publisher's old pages, archive sites,
   magazine cover discs and fan sites. Note the extras that came with each (tools, extra
   levels, fan files).
4. **Record each one**: its source, the checksums of its files or image, its languages, content
   and builds.
5. **Measure the data delta** between each pair:
   `.venv/bin/python tools/triage/versiondiff.py game/first/GAME.CUE game/latest/GAME.CUE`.
   It reads folders or disc images in place and prints the files added, removed and changed by
   folder, the PE link dates of the executables, and the languages. Paste it into the
   data-delta section.
6. **Pick the reference**: by default the last version released, which has the most fixes and
   improvements. Write down the reason when you pick another.

The code delta comes after the Ghidra export of each edition: `match_functions.py` matches
their functions and carries labels across. When a newer edition turns up mid-project, follow
"Moving to a newer version mid-project" in `docs/working/versions.md`: export it, match the
functions, record the ground truth again, verify the port again, and keep the old edition as
documented differences. Comparing builds is often the quickest way to find a rule: a patch note
names it, and the function that changed between two builds is it.

## 6. Classify the game

Look at the game itself, not only its files: play it or watch it played. Then place it in the
taxonomy (`docs/games/README.md`): who drives (a stream with controls, software over a
simulation, an environment, turns and choices), its time model, where its rules live, what
must repeat exactly, and its family. Read the family page and copy its rule set into the
project's `AGENTS.md`, adjusted to the game. Record in `docs/FINDINGS.md` how exact the
interface must be, and why. A mixed game gets one line per mode, and the state that switches
between them. Then read the story (section 7). Next: `re-observe`.

## 7. Read the story

A game's manual and the text inside its program often say what a system is for before the
code does. That tells you which systems the designers cared about, and so which must be exact
(`docs/working/lore.md`, with Hardwar as the worked example). Do this now, before the tracing
starts:

```bash
find target -iname '*.pdf' -o -iname '*.txt' -o -iname '*.doc' -o -iname '*.hlp' | head   # manual, README
./gameframe run sentences game/GAME.EXE --min-words 6 > analysis/notes/sentences.txt    # the program's long strings
./gameframe run sentences analysis/export/GAME.EXE/strings.txt --min-words 6              # the same from Ghidra's export
```

1. Read the manual, the README and any text the publisher shipped. Read the long strings in
   address order: messages, mission text and item descriptions often hold the plot and state
   rules in plain words (a production chain, a deadline, a limit).
2. Write `docs/lore.md` (the project's template): the setting in your own words, the rules the
   text states with the source of each, and the table "in the story → in the code →
   evidence". Mark each code side *not yet traced* until it is.
3. Copy what the lore says must be exact into `AGENTS.md`, "Rules for this kind of game": a
   resource the player plans around, a timed rule, a limit, a world the plot changes (the save
   must hold it), an ending that tests something.
4. Go back to `docs/lore.md` whenever a system's purpose is unclear or a constant has no
   reason, and fill in its code side as each system is traced.

The sentences list is the game's text: keep it in `analysis/`. In tracked files, name a string
by its address and say what it states. A manual describes the design, which can differ from
the build: a rule from it is a lead until the code shows it.
