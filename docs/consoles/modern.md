# Seventh generation onward

From the Xbox 360 on, every platform ships signed executables in a vendor format such as XEX,
SELF, RPX or NSO, and most of them are encrypted. A decrypted dump and the right Ghidra loader
come before any analysis. This page also covers the handhelds of the same years: the Nintendo
DS, 3DS, PSP and PS Vita.

## The platforms

| Platform | Year | CPU | Executable | Ghidra module or loader | Emulator with a debugger |
|---|---|---|---|---|---|
| Nintendo DS, DSi | 21 November 2004 | ARM946E-S at 67 MHz and ARM7TDMI at 33 MHz | ARM9 and ARM7 binaries, overlays, NitroFS | [NTRGhidra](https://github.com/pedro-javierf/NTRGhidra) | melonDS, DeSmuME, no$gba |
| PSP | 12 December 2004 (JP) | Allegrex, MIPS32 with VFPU, 222 MHz default | EBOOT.BIN and PRX, encrypted "~PSP" ELF | [ghidra-allegrex](https://github.com/kotcrab/ghidra-allegrex) | PPSSPP |
| Xbox 360 | 22 November 2005 | Xenon, 3 PowerPC cores at 3.2 GHz, VMX128, big-endian | XEX2, signed, usually encrypted, wraps PE | [XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV) | Xenia Canary |
| PlayStation 3 | November 2006 | Cell: PPE plus six SPEs for games | SELF and SPRX (EBOOT.BIN) | [GhidraSPU](https://github.com/aerosoul94/GhidraSPU) for SPU; no SELF loader confirmed | RPCS3 |
| Wii | 19 November 2006 | Broadway PowerPC at 729 MHz; Starlet ARM9 runs IOS | DOL and REL; discs AES-encrypted | [Ghidra-GameCube-Loader](https://github.com/Cuyler36/Ghidra-GameCube-Loader), paired-singles module | Dolphin |
| Nintendo 3DS | 26 February 2011 (JP) | ARM11 MPCore, 2 cores at 268 MHz; ARM9 for security | NCSD/CCI, CIA, NCCH with ExeFS; 3DSX homebrew | [ghidra-3ds](https://github.com/Stary2001/ghidra-3ds), [ghidra-3dsx-loader](https://github.com/danylokos/ghidra-3dsx-loader) | Azahar |
| PS Vita | 17 December 2011 (JP) | Quad Cortex-A9 | SELF wrapping ARM ELF; SUPRX modules | [GhidraVitaLoader](https://github.com/xerpi/GhidraVitaLoader), [VitaLoaderRedux](https://github.com/CreepNT/VitaLoaderRedux) | Vita3K |
| Wii U | 18 November 2012 | Espresso, 3 PowerPC cores at 1.24 GHz | RPX and RPL ELF variants | [GhidraRPXLoader](https://github.com/cemu-project/GhidraRPXLoader) | Cemu |
| PlayStation 4 | 15 November 2013 | 8 Jaguar x86-64 cores at 1.6 GHz | SELF with x86-64 ELF | [GhidraPS4Loader, GhidraOrbis](https://p.codekk.com/detail/64e02e470c2d5a3fbe2aa984) | shadPS4 |
| Xbox One | 22 November 2013 | 8 Jaguar cores at 1.75 GHz | Encrypted XVC containers | None known | None known |
| Nintendo Switch | 3 March 2017 | 4 Cortex-A57 at 1.02 GHz | NSO in encrypted NCA; NRO homebrew | [Ghidra-Switch-Loader](https://github.com/Adubbz/Ghidra-Switch-Loader) | Ryubing, Eden, Citron |
| PlayStation 5 | 12 November 2020 | Zen 2, 8 cores to 3.5 GHz | | None known | None known |
| Xbox Series X and S | 10 November 2020 | Zen 2, 8 cores to 3.8 GHz | | None known | None known |

No emulator is known to run commercial Xbox One, PS5 or Series games. The PS4, Xbox One, PS5
and Series are x86-64, and many of their games also have PC releases, which may be the easier
target. Switch 2 (5 June 2025) is not covered here.

## Keys and encrypted executables

- Switch NCA contents are encrypted. Emulators need prod.keys, title keys and firmware dumped
  from the user's own console (not confirmed) ([switchbrew: NCA](https://switchbrew.org/wiki/NCA)).
- PS3 SELF files are signed and encrypted. RPCS3 needs the official firmware and a disc key
  (not confirmed) ([psdevwiki: SELF](https://www.psdevwiki.com/ps3/SELF_-_SPRX);
  [RPCS3 quickstart](https://rpcs3.net/quickstart)).
- XEX2 files are decrypted with the console-family key
  ([XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV)).
- PS4 loaders need a decrypted ELF without the Sony header.
- 3DS content is AES-encrypted, with keys in the ARM9 boot ROM ([3dbrew](https://www.3dbrew.org/)).
- Wii discs are AES-encrypted.

Recompilation projects such as Unleashed Recompiled and the GoldenEye 360 recompilation ship no
game files and need the user's own copy
([tbreak](https://tbreak.com/goldeneye-007-xbox-360-pc-recompilation/)). That practice matches
GameFrame's rule of no game material in the repository.

On 4 March 2024 Tropic Haze, yuzu's developer, settled with Nintendo for $2.4 million. A Rhode
Island federal court entered a permanent injunction against offering yuzu or software that
circumvents Nintendo's protections. The yuzu domain went to Nintendo
([Android Authority](https://androidauthority.com/nintendo-switch-yuzu-settlement-3422388);
[Nintendo World Report](https://www.nintendoworldreport.com/news/66510/creators-of-yuzu-emulator-settle-lawsuit-with-nintendo)).
Citra came down at the same time ([Wikipedia: Citra](https://en.wikipedia.org/wiki/Citra_(emulator))).
The 2024 to 2026 cases ended in settlements. No court ruling on console emulators is known from
that period. The user decides how to obtain and use their own keys and dumps.

## Emulator status

Repositories can disappear, so pin an exact emulator
build and keep a local copy.

- Dolphin has a full debugger, shared with the GameCube ([Dolphin](https://dolphin-emu.org/)).
- RPCS3 has a PPU and SPU debugger (not confirmed) ([RPCS3](https://rpcs3.net/)).
- PPSSPP has a built-in debugger and a WebSocket remote debugger (not confirmed)
  ([PPSSPP](https://www.ppsspp.org/)).
- Vita3K's status is not confirmed ([Vita3K](https://vita3k.org/)).
- Cemu has been open source since 2022. Its organisation maintains the RPX loader.
- Xenia Canary is the active Xbox 360 branch. Secondary coverage reports near-daily commits into
  July 2026, against a slower original repository
  ([tech-insider](https://tech-insider.org/?p=20173); [Snapcraft: xenia-canary](https://snapcraft.io/xenia-canary)).
- Citra shut down in March 2024. Lime3DS and PabloMK7's fork merged into Azahar in late 2024,
  with first releases in early 2025 ([Azahar](https://azahar-emu.org/);
  [tech-insider](https://tech-insider.org/?p=19264)).
- shadPS4 runs Bloodborne. Reported versions are v0.16.0 in June 2026 and v0.19.0 in October
  2026 ([Wikipedia: ShadPS4](https://en.wikipedia.org/wiki/ShadPS4);
  [itechguides](https://www.itechguides.com/what-the-bloodborne-pc-port-is-and-how-it-works/)).
- On the Switch, yuzu ended in March 2024. Ryubing, a Ryujinx fork at ryujinx.app, moved to
  maintenance in February 2026. Eden and Citron are described as the active forks, and suyu was
  an early fork. A February 2026 wave of takedown notices reportedly hit at least 13 Switch
  projects, including Citron, Eden and Ryubing. These Switch facts come from secondary 2026
  coverage ([tech-insider: Eden vs Ryubing](https://tech-insider.org/eden-vs-ryubing-best-switch-emulator-2026/);
  [tech-insider](https://tech-insider.org/?p=23388)).

## Nintendo DS and DSi

### Hardware

The DS has an ARM946E-S at 67 MHz and an ARM7TDMI at 33 MHz. A game ships ARM9 and ARM7
binaries, overlays and a NitroFS file system ([Nintendo DS](https://en.wikipedia.org/wiki/Nintendo_DS);
[GBATEK](https://problemkaputt.de/gbatek.htm)).

### Tools

[NTRGhidra](https://github.com/pedro-javierf/NTRGhidra) loads DS binaries. Load the ARM9 and the
ARM7 code as separate programs. melonDS, DeSmuME and no$gba run the games.

### Projects to learn from

pret's pokeplatinum and pokeheartgold are active C decompilations. An SDL3 port fork is in
progress ([gittrend: pokeplatinum](https://gittrend.io/repo/pret/pokeplatinum);
[pokeplatinum-portable](https://github.com/arcanite24/pokeplatinum-portable)).

## PSP

### Hardware

The Allegrex is a MIPS32 core with a VFPU, at 222 MHz by default. Games ship as EBOOT.BIN and
PRX files, which are encrypted "~PSP" ELFs
([PlayStation Portable](https://en.wikipedia.org/wiki/PlayStation_Portable)).

### Tools

Ghidra has no Allegrex language built in. [ghidra-allegrex](https://github.com/kotcrab/ghidra-allegrex)
adds it. Capstone has no Allegrex mode. splat supports PSP binaries
([GitHub: splat](https://github.com/ethteck/splat)). PPSSPP runs the games.

### Projects to learn from

The Castlevania: Symphony of the Night decompilation includes a PSP build.

## Xbox 360

### Hardware

Xenon has three in-order big-endian PowerPC cores at 3.2 GHz with VMX128
([Wikipedia: Xenon](https://en.wikipedia.org/wiki/Xenon_(processor))). Executables are XEX2 files
that wrap a PE. They are signed and usually encrypted
([Xbox 360 technical specifications](https://en.wikipedia.org/wiki/Xbox_360_technical_specifications)).

### Tools

[XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV) loads XEX2 files. Ghidra's PowerPC
module has no VMX128, and Unicorn has no VMX128.

### Projects to learn from

hedge-dev's XenonRecomp (PowerPC to C++) and XenosRecomp (shaders to HLSL) produced Unleashed
Recompiled, a native PC port of Sonic Unleashed revealed in March 2025. It ships without game
files and was inspired by N64: Recompiled
([Unleashed Recompiled README](https://cdn.jsdelivr.net/gh/hedge-dev/unleashedrecomp@main/README.md);
[Time Extension, March 2025](https://www.timeextension.com/news/2025/03/this-new-unofficial-pc-port-of-sonic-unleashed-could-be-the-best-version-of-the-game-yet)).
ReXGlue (2026) is an open SDK built on XenonRecomp and Xenia, with early ports such as Blue
Dragon and Ninja Gaiden 2. It is "in early development"
([Time Extension, March 2026](https://www.timeextension.com/news/2026/03/i-hold-my-work-to-a-strict-standard-the-driving-force-behind-xbox-360-recomp-tool-rexglue-speaks-out);
[Notebookcheck](https://www.notebookcheck.net/ReXGlue-SDK-debut-sparks-Xbox-360-to-PC-ports-for-Blue-Dragon-more.1231442.0.html)).

## PlayStation 3

### Hardware

The Cell pairs one PPE with SPUs. Games get six SPEs. Each SPU has its own 128-bit instruction
set, 256 KiB of local store and DMA-only access to main memory
([Wikipedia: Cell](https://en.wikipedia.org/wiki/Cell_(processor));
[psdevwiki](https://www.psdevwiki.com/ps3/)). Executables are SELF and SPRX files.

### Tools

[GhidraSPU](https://github.com/aerosoul94/GhidraSPU) adds the SPU. No SELF loader is
confirmed. RPCS3 runs the games.

### Projects to learn from

PS3Recomp is a PS3 static recompiler. One guide says its first playable games appeared in
October 2026 without public downloads. The claim of "more than three thousand games" lacks a
primary source ([Generation Amiga, June 2026](https://www.generationamiga.com/2026/06/15/ps3recomp-the-new-tool-bringing-native-playstation-3-ports-closer-to-pc/);
[Held Games](https://heldgames.com/tags/ps3recomp)).

### For an exact reimplementation

Cell SPU single precision rounds toward zero, treats denormals as zero and has no INF or NaN
(not confirmed) ([IBM Cell SDK 3.0](https://arcb.csc.ncsu.edu/~mueller/cluster/ps3/SDK3.0/docs/accessibility/sdkpt/cbet_3codemapeqv.html)).
A port of SPU float code that uses host IEEE floats will drift, so it needs a software float
model. Export the PPE and SPU code separately.

## Wii and Wii U

### Hardware

The Wii's Broadway is a PowerPC at 729 MHz. A Starlet ARM9 runs IOS. Executables are DOL and REL
files, and discs are AES-encrypted ([Wii](https://en.wikipedia.org/wiki/Wii)). The Wii U's
Espresso has three PowerPC cores at 1.24 GHz. Its executables are RPX and RPL ELF variants
([Wii U](https://en.wikipedia.org/wiki/Wii_U)). Broadway and Espresso descend from the
GameCube's Gekko ([Wikipedia: Gekko](https://en.wikipedia.org/wiki/Gekko_(processor))).

### Tools

The Wii uses the GameCube's tools: [Ghidra-GameCube-Loader](https://github.com/Cuyler36/Ghidra-GameCube-Loader)
for DOL and REL, and [ghidra-gekko-broadway-lang](https://github.com/aldelaro5/ghidra-gekko-broadway-lang)
for paired singles ([WiiBrew: Ghidra](https://wiibrew.org/wiki/Ghidra)). Dolphin runs the games.
[GhidraRPXLoader](https://github.com/cemu-project/GhidraRPXLoader) loads Wii U files, and Cemu
runs them.

### Projects to learn from

Super Mario Galaxy was reported 100% decompiled in late September 2026. It is the first Wii game
to get there. A PC port called AstroCore is in progress, and decomp-toolkit is the tool
([Android Authority](https://androidauthority.com/super-mario-galaxy-decompiled-pc-port-3717193);
[GitHub: Petari](https://github.com/Gota7/Petari); [decomp.dev](https://decomp.dev/)).

### For an exact reimplementation

Gekko paired-single results depend on the quantised load and store (GQR) setup. Confirm it
against Dolphin's code. The PowerPC consoles now share tooling, so big-endian PowerPC can be
treated as one family.

## Nintendo 3DS

### Hardware

The 3DS has an ARM11 MPCore with 2 cores at 268 MHz, and an ARM9 for security. Formats are
NCSD/CCI, CIA and NCCH with ExeFS. Homebrew uses 3DSX
([Nintendo 3DS](https://en.wikipedia.org/wiki/Nintendo_3DS); [3dbrew](https://www.3dbrew.org/)).

### Tools

[ghidra-3ds](https://github.com/Stary2001/ghidra-3ds) loads 3DS files. Its analysis is
described as "pretty broken".
[ghidra-3dsx-loader](https://github.com/danylokos/ghidra-3dsx-loader) loads 3DSX homebrew.
Azahar runs the games.

## PS Vita

### Hardware

The Vita has a quad Cortex-A9. Executables are SELF files wrapping an ARM ELF, with SUPRX modules
([PlayStation Vita](https://en.wikipedia.org/wiki/PlayStation_Vita)).

### Tools

[GhidraVitaLoader](https://github.com/xerpi/GhidraVitaLoader) and
[VitaLoaderRedux](https://github.com/CreepNT/VitaLoaderRedux) load Vita files. Vita3K runs the
games.

## PlayStation 4

### Tools

GhidraPS4Loader and GhidraOrbis load PS4 files
([codekk](https://p.codekk.com/detail/64e02e470c2d5a3fbe2aa984)). They need a decrypted ELF
without the Sony header. shadPS4 is the emulator
([PlayStation 4 technical specifications](https://en.wikipedia.org/wiki/PlayStation_4_technical_specifications)).

## Nintendo Switch

### Hardware

The Switch has 4 Cortex-A57 cores at 1.02 GHz. Games ship as NSO files inside encrypted NCA
containers. Homebrew uses NRO ([Nintendo Switch](https://en.wikipedia.org/wiki/Nintendo_Switch);
[switchbrew NSO](https://switchbrew.org/wiki/NSO)).

### Tools

[Ghidra-Switch-Loader](https://github.com/Adubbz/Ghidra-Switch-Loader) loads Switch executables.
Ghidra's AARCH64 language covers v8.5-A, "missing some 8.5 vector"
([AARCH64.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/AARCH64/data/languages/AARCH64.ldefs)).

### Projects to learn from

A Breath of the Wild v1.5.0 decompilation is in progress ([GitHub: botw](https://github.com/german77/botw)).

## Matching decompilations not yet done

No completed PSP, Vita, 3DS or PS4 matching decompilation is known.

Sources: [Report: game platforms and CPUs](../computers/research/report.md).
