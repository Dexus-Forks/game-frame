# Notes: more consoles and handhelds

Covers consoles and handhelds beyond the NES-to-GBA, 32/64-bit, sixth-generation and arcade platforms: hardware, emulators, Ghidra support, notable projects and legal notes. Hardware facts cite the platform's Wikipedia article, cross-checked against developer wikis where named. "(not confirmed)" marks a fact without a primary-source confirmation. "(secondary source)" marks one from news or blog coverage only.

## Hardware, media, executable format and SDK per platform

### Takeaway
The early consoles (Odyssey to Vectrex) have tiny ROMs, no OS and well-known 8-bit or 16-bit CPUs; the 32-bit era adds co-processors that carry real game code (Jaguar Tom/Jerry, 32X twin SH-2, Sega CD 68000); from the Xbox 360 on, every platform ships signed and encrypted executables in a vendor format (XEX, SELF, RPX, NSO), so a decrypted dump and the right loader come before any analysis.

### Cited Findings

#### First and second generation
- **Magnavox Odyssey**: released September 1972 in North America; designed by Ralph Baer. It has no CPU and no ROM: games are discrete transistor and diode logic, and the plug-in "game cards" only rewire jumpers between the console's circuits; screen overlays supplied the playfield. — [Wikipedia: Magnavox Odyssey](https://en.wikipedia.org/wiki/Magnavox_Odyssey)
- **Atari 2600 (VCS)**: released September 1977 in North America. CPU MOS 6507 (a 6502 with 13 address lines, no interrupts) at 1.19 MHz (NTSC). TIA chip does video and two-channel sound; RIOT (MOS 6532) gives 128 bytes of RAM, I/O and a timer. No frame buffer: the program writes TIA registers in time with each scanline ("racing the beam"). Cartridge address space is 4 KB; larger games use bank-switching schemes (F8, F6, E0, 3F and others). — [Wikipedia: Atari 2600](https://en.wikipedia.org/wiki/Atari_2600); [Wikipedia: Atari 2600 hardware](https://en.wikipedia.org/wiki/Atari_2600_hardware)
- **Intellivision** (Mattel): test market 1979, national US release 1980. CPU General Instrument CP1610, 16-bit, clocked at about 894.886 kHz (NTSC colour clock divided by 4). STIC (AY-3-8900-1) video, AY-3-8914 sound. ROM is 10-bit wide ("decles"); the built-in Executive ROM (4K x 10-bit) holds an OS-like runtime many games call into. — [Wikipedia: Intellivision](https://en.wikipedia.org/wiki/Intellivision); clock figure (not confirmed against jzIntv docs)
- **ColecoVision**: released August 1982 in North America. Zilog Z80A at 3.58 MHz; Texas Instruments TMS9928A video with 16 KB VRAM; SN76489A sound; 1 KB RAM; 8 KB BIOS ROM. — [Wikipedia: ColecoVision](https://en.wikipedia.org/wiki/ColecoVision)
- **Atari 5200**: released November 1982 in North America. Custom 6502C ("SALLY") at 1.79 MHz; ANTIC + GTIA graphics and POKEY sound, the same chips as the Atari 8-bit computers; 16 KB RAM; cartridges up to 32 KB. — [Wikipedia: Atari 5200](https://en.wikipedia.org/wiki/Atari_5200)
- **Atari 7800**: test market 1984, national release May 1986 (NA). 6502C "SALLY" at 1.79 MHz (slowed to 1.19 MHz when accessing TIA/RIOT); MARIA graphics chip; TIA for sound (some carts add a POKEY); 4 KB RAM. Plays 2600 cartridges. NTSC cartridges carry a digital signature checked by the BIOS; PAL units skip the check. — [Wikipedia: Atari 7800](https://en.wikipedia.org/wiki/Atari_7800)
- **Vectrex** (GCE, later Milton Bradley): released November 1982 in North America. Motorola 68A09 (6809) at 1.5 MHz; AY-3-8912 sound; 1 KB RAM; 8 KB system ROM with built-in game Mine Storm; built-in vector CRT; the CPU drives the beam through a DAC and analogue integrators. — [Wikipedia: Vectrex](https://en.wikipedia.org/wiki/Vectrex)

#### Other 8/16/32-bit platforms
- **Sega Game Gear**: released 6 October 1990 in Japan, 1991 in NA/EU. Z80 at 3.58 MHz; VDP derived from the Master System (4,096-colour palette, 160x144 visible); SN76489 sound with stereo; 8 KB RAM, 16 KB VRAM. Close enough to the Master System that many games are SMS ports. — [Wikipedia: Game Gear](https://en.wikipedia.org/wiki/Game_Gear)
- **Atari Lynx**: released September 1989 in NA. Custom "Mikey" chip contains a 65SC02 at 4 MHz plus sound and timers; "Suzy" chip is a blitter with hardware sprite scaling and a maths unit; 64 KB RAM; 160x102 screen, 4,096-colour palette. Cartridge boot uses an encrypted loader decrypted by the boot ROM, so emulators need the boot ROM image. — [Wikipedia: Atari Lynx](https://en.wikipedia.org/wiki/Atari_Lynx)
- **Neo Geo Pocket / Color** (SNK): mono Neo Geo Pocket October 1998 in Japan; Neo Geo Pocket Color March 1999 (JP). Toshiba TLCS-900/H at 6.144 MHz; Z80 at 3.072 MHz as sound CPU; T6W28 PSG (SN76489-like); 160x152 screen. — [Wikipedia: Neo Geo Pocket Color](https://en.wikipedia.org/wiki/Neo_Geo_Pocket_Color)
- **WonderSwan** (Bandai): released March 1999 in Japan only; WonderSwan Color December 2000; SwanCrystal 2002. NEC V30MZ (x86 16-bit compatible) at 3.072 MHz; 16 KB RAM (64 KB on Color); 224x144 screen; 4-channel wavetable sound. — [Wikipedia: WonderSwan](https://en.wikipedia.org/wiki/WonderSwan)
- **Virtual Boy** (Nintendo): released July 1995 in Japan, August 1995 in NA. NEC V810 (32-bit RISC) at 20 MHz; red LED scanning displays at 384x224 per eye; VIP video chip, VSU sound. — [Wikipedia: Virtual Boy](https://en.wikipedia.org/wiki/Virtual_Boy)
- **Atari Jaguar**: released November 1993 in NA. Motorola 68000 at 13.295 MHz plus two custom 32-bit RISC processors: "Tom" (GPU, object processor, blitter) and "Jerry" (DSP, sound), both at 26.59 MHz, each with its own local SRAM; 2 MB RAM. Cartridge; Jaguar CD add-on 1995. Many games run their main loop on the 68000 and upload RISC code to Tom/Jerry local RAM. — [Game Tech Wiki: Jaguar](https://emulation.gametechwiki.com/index.php/Jaguar); [Wikipedia: Atari Jaguar](https://en.wikipedia.org/wiki/Atari_Jaguar)
- **3DO Interactive Multiplayer**: first unit (Panasonic FZ-1) released October 1993 in NA. ARM60 (ARMv3) at 12.5 MHz; two custom chips (Madam and Clio) with "cel engine" graphics and a DSP for audio; 2 MB DRAM + 1 MB VRAM; CD-ROM. OS: Portfolio, a multitasking OS with its own executable format. — [Wikipedia: 3DO Interactive Multiplayer](https://en.wikipedia.org/wiki/3DO_Interactive_Multiplayer)
- **Sega 32X**: released November 1994 in NA. Two Hitachi SH-2 (SH7095) at 23 MHz; 256 KB SDRAM; 256 KB frame buffer; PWM sound; sits on top of the Mega Drive's 68000 and Z80. — [Wikipedia: 32X](https://en.wikipedia.org/wiki/32X)
- **Mega-CD / Sega CD**: released December 1991 in Japan, October 1992 in NA. Second Motorola 68000 at 12.5 MHz; 512 KB program RAM, 256 KB word RAM (shared with the Mega Drive), 64 KB PCM RAM, 8 KB backup RAM; Ricoh RF5C164 8-channel PCM; CD-ROM with CD audio; a BIOS on the add-on. — [Wikipedia: Sega CD](https://en.wikipedia.org/wiki/Sega_CD); exact backup RAM size (not confirmed)
- **PC Engine CD-ROM² / TurboGrafx-CD**: released December 1988 in Japan, 1989 in NA. Same HuC6280 (65C02 derivative) at 7.16 MHz as the HuCard system; games depend on a "System Card" BIOS in the HuCard slot (Super System Card 1991 adds RAM; Arcade Card 1994 adds 2 MB). — [Wikipedia: TurboGrafx-CD](https://en.wikipedia.org/wiki/TurboGrafx-CD)
- **Philips CD-i**: first players December 1991 (NA). Philips SCC68070 (68000-compatible) at 15.5 MHz; MCD 212 video; OS: CD-RTOS, based on Microware OS-9; programs are OS-9 modules. — [Wikipedia: CD-i](https://en.wikipedia.org/wiki/CD-i); clock (not confirmed)

#### Seventh generation and later
- **Xbox 360**: released 22 November 2005 (NA). CPU "Xenon": IBM PowerPC, three cores at 3.2 GHz, two hardware threads each, VMX128 vector units, big-endian. GPU "Xenos" (ATI) at 500 MHz with 10 MB eDRAM. 512 MB GDDR3 unified. DVD discs, Xbox Live Arcade digital. Executable: XEX2 (signed, usually encrypted and compressed, wraps a PE image). SDK: Xbox 360 XDK. — [Wikipedia: Xbox 360 technical specifications](https://en.wikipedia.org/wiki/Xbox_360_technical_specifications); [XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV)
- **PlayStation 3**: released 11 November 2006 (JP), 17 November 2006 (NA), March 2007 (EU). Cell Broadband Engine at 3.2 GHz: one PPE (PowerPC 64-bit, two threads) and eight SPEs, one disabled for yield and one reserved by the OS, leaving six for games. RSX GPU (Nvidia). 256 MB XDR + 256 MB GDDR3. Blu-ray. Executables: SELF/SPRX (signed, encrypted ELF; EBOOT.BIN). OS: CellOS ("GameOS"). — [Wikipedia: PlayStation 3 technical specifications](https://en.wikipedia.org/wiki/PlayStation_3_technical_specifications); [psdevwiki](https://www.psdevwiki.com/ps3/)
- **Wii**: released 19 November 2006 (NA). IBM "Broadway" PowerPC (Gekko successor, paired-single floats) at 729 MHz; ATI "Hollywood" GPU at 243 MHz, which also contains an ARM9 "Starlet" running IOS; 24 MB 1T-SRAM + 64 MB GDDR3. Discs AES-encrypted. Executables: DOL with REL modules, as on GameCube. — [Wikipedia: Wii](https://en.wikipedia.org/wiki/Wii); [WiiBrew](https://wiibrew.org/wiki/Ghidra)
- **Nintendo DS**: released 21 November 2004 (NA). ARM946E-S at 67 MHz and ARM7TDMI at 33 MHz; 4 MB RAM; 656 KB VRAM; two screens. Cartridge ROM holds separate ARM9 and ARM7 binaries, overlays and a NitroFS file system. SDK: NitroSDK. DSi (November 2008, JP): ARM9 at 133 MHz, 16 MB RAM, AES-encrypted DSi-enhanced content. — [Wikipedia: Nintendo DS](https://en.wikipedia.org/wiki/Nintendo_DS); [GBATEK](https://problemkaputt.de/gbatek.htm)
- **Nintendo 3DS**: released 26 February 2011 (JP), 27 March 2011 (NA). ARM11 MPCore, two cores at 268 MHz (New 3DS: four cores at 804 MHz); ARM9 at 134 MHz for security and I/O; DMP PICA200 GPU; 128 MB FCRAM (New 3DS 256 MB). Game cards and eShop. Containers: NCSD/CCI (cards), CIA (installable), NCCH with ExeFS ".code"; homebrew uses 3DSX. Content is AES-encrypted with keys held in the ARM9 boot ROM. — [Wikipedia: Nintendo 3DS](https://en.wikipedia.org/wiki/Nintendo_3DS); [3dbrew](https://www.3dbrew.org/)
- **PSP**: released 12 December 2004 (JP), 24 March 2005 (NA). "Allegrex" CPU, MIPS32 R4000-based with FPU and VFPU vector unit, 1 to 333 MHz (games default to 222 MHz); Media Engine second MIPS core; 32 MB RAM (PSP-1000), 64 MB on later models; 2 MB eDRAM in the GPU. UMD discs and PSN. Executables: EBOOT.BIN / PRX, encrypted "~PSP" wrapped ELF. — [Wikipedia: PlayStation Portable](https://en.wikipedia.org/wiki/PlayStation_Portable); [ghidra-allegrex](https://github.com/kotcrab/ghidra-allegrex)
- **PS Vita**: released 17 December 2011 (JP), 22 February 2012 (NA/EU). Quad-core ARM Cortex-A9 MPCore; PowerVR SGX543MP4+ GPU; 512 MB RAM + 128 MB VRAM. Vita cards and PSN. Executables: SELF (eboot.bin) wrapping an ARM ELF with Sony relocation and import/export tables; modules are SUPRX. Game clock speeds are not confirmed. — [Wikipedia: PlayStation Vita](https://en.wikipedia.org/wiki/PlayStation_Vita); [VitaLoaderRedux](https://github.com/CreepNT/VitaLoaderRedux)
- **Wii U**: released 18 November 2012 (NA). IBM "Espresso" PowerPC, three cores at 1.24 GHz (Broadway lineage); AMD "Latte" GPU at 550 MHz; 2 GB DDR3 (1 GB for games). Encrypted optical discs and eShop. Executables: RPX (main) and RPL (libraries), ELF variants with compressed sections. OS: Cafe OS. — [Wikipedia: Wii U](https://en.wikipedia.org/wiki/Wii_U); [GhidraRPXLoader](https://github.com/cemu-project/GhidraRPXLoader)
- **PlayStation 4**: released 15 November 2013 (NA). AMD APU with eight Jaguar x86-64 cores at 1.6 GHz; GCN GPU, 18 CUs at 800 MHz; 8 GB GDDR5. Blu-ray and PSN. OS: Orbis OS, FreeBSD-based. Executables: SELF (eboot.bin) and SPRX/PRX, x86-64 ELF inside. — [Wikipedia: PlayStation 4 technical specifications](https://en.wikipedia.org/wiki/PlayStation_4_technical_specifications); [GhidraPS4Loader / GhidraOrbis mirrors](https://p.codekk.com/detail/64e02e470c2d5a3fbe2aa984)
- **Xbox One**: released 22 November 2013. AMD APU, eight Jaguar cores at 1.75 GHz; GCN GPU, 12 CUs at 853 MHz; 8 GB DDR3 + 32 MB ESRAM. OS: hypervisor running a Windows-kernel-based game OS. Games ship in encrypted XVC containers. — [Wikipedia: Xbox One](https://en.wikipedia.org/wiki/Xbox_One)
- **Nintendo Switch**: released 3 March 2017 worldwide. Nvidia Tegra X1 (custom): four ARM Cortex-A57 cores at 1.02 GHz for games (the four A53 cores are unused); Maxwell GPU with 256 CUDA cores at 307.2/384 MHz handheld and 768 MHz docked; 4 GB LPDDR4. Game cards (XCI) and eShop (NSP). Executables: NSO (retail, LZ4-compressed segments), NRO (homebrew), inside encrypted NCA containers. OS: Horizon; SDK: NintendoSDK. — [Wikipedia: Nintendo Switch](https://en.wikipedia.org/wiki/Nintendo_Switch); [switchbrew NSO](https://switchbrew.org/wiki/NSO)
- **PlayStation 5**: released 12 November 2020. AMD Zen 2, eight cores up to 3.5 GHz; RDNA 2 GPU, 36 CUs up to 2.23 GHz; 16 GB GDDR6; custom NVMe SSD. — [Wikipedia: PlayStation 5](https://en.wikipedia.org/wiki/PlayStation_5)
- **Xbox Series X|S**: released 10 November 2020. Series X: Zen 2, eight cores at 3.8 GHz (3.66 GHz with SMT), RDNA 2 GPU with 52 CUs at 1.825 GHz, 16 GB GDDR6. Series S: 3.6/3.4 GHz, 20 CUs at 1.565 GHz, 10 GB GDDR6. — [Wikipedia: Xbox Series X and Series S](https://en.wikipedia.org/wiki/Xbox_Series_X_and_Series_S)

### Inferences
- The Odyssey cannot be reverse engineered as software: a "port" is a circuit simulation, closer to the discrete-logic arcade games than to any CPU platform.
- For the 2600, timing is the game: a faithful rebuild must model cycles per scanline, not just logic. This argues for cycle-exact emulator traces as ground truth.
- Jaguar, 32X, Sega CD, PS3 (SPU) and DS (ARM7) all split game code across two or more CPUs with different ISAs. Locate and export each CPU's code separately, often by dumping RAM after the main CPU uploads it.
- PS4/Xbox One/PS5/Series are x86-64; many of their games also have PC releases, which may be an easier RE target than the console build.

### Gaps
- Exact 3DO ARM60 and CD-i SCC68070 clocks, Vita game clock limits and Sega CD backup RAM size are not confirmed against primary docs.
- Nintendo Switch 2 (released 5 June 2025) is outside these notes.

## Emulators with debuggers, and their status in 2026

### Takeaway
Every platform in scope has at least one emulator, but debugger quality varies widely: excellent on the 8-bit systems (Stella, jzIntv, MAME, Mednafen, Mesen 2) and on PSP/DS/Wii; usable on PS3 (RPCS3) and 360 (Xenia Canary); fragile and legally contested on Switch, where the leading projects were shut down in 2024 and successors face takedowns in 2026.

### Cited Findings
- **Atari 2600**: Stella is the reference emulator; it has a built-in debugger with a disassembler (based on DiStella), breakpoints, TIA/RIOT views and scanline-level stepping. — [Stella](https://stella-emu.github.io/); debugger feature list (not confirmed)
- **Intellivision**: jzIntv includes a command-line debugger. — [jzIntv](http://spatula-city.org/~im14u2c/intv/) (not confirmed)
- **ColecoVision, Vectrex, 5200, 7800, Lynx, NGP, CD-i, Game Gear**: MAME emulates all of these and its built-in debugger works across CPUs; CD-i is emulated by MAME (cdimono1) and CeDImu. — [MAME docs: debugger](https://docs.mamedev.org/debugger/index.html) (per-system accuracy not confirmed)
- **Atari 5200**: Altirra emulates the 5200 and has a strong debugger (Atari 8-bit lineage). (not confirmed)
- **Lynx**: Handy is the classic emulator; Mednafen's Lynx module derives from Handy; a boot ROM image is needed. — [Wikipedia: Atari Lynx](https://en.wikipedia.org/wiki/Atari_Lynx) (emulator lineage not confirmed)
- **Jaguar**: BigPEmu is closed source, by Rich Whitehouse, and was written for Digital Eclipse as the Jaguar emulator in Atari 50. Virtual Jaguar is the open-source alternative and is being maintained by Provenance developer Joseph Mattiello, adding GPU, DSP and blitter timing models. — [Game Tech Wiki: Jaguar](https://emulation.gametechwiki.com/index.php/Jaguar). Whether BigPEmu exposes a debugger is not confirmed.
- **3DO**: Opera (libretro core, descended from FreeDO/4DO) is the main open emulator. (not confirmed)
- **Virtual Boy, WonderSwan, NGP, PC Engine CD**: Mednafen covers all four and has a debugger in its SDL front end; Mesen 2 covers PC Engine (incl. CD) and WonderSwan with a full debugger. — [Mednafen docs](https://mednafen.github.io/documentation/); [Mesen](https://www.mesen.ca/) (Mesen 2 system list not confirmed)
- **32X / Sega CD**: Genesis Plus GX (Sega CD), PicoDrive (32X and CD), ares and MAME; debugger support is best in MAME and in Gens forks. (not confirmed)
- **Xbox 360**: Xenia Canary is the active branch; the original Xenia repository has slowed (one tracker shows a last push 18 February 2026, while Canary had near-daily commits into July 2026). Release numbers conflict between sources. — [tech-insider (secondary source)](https://tech-insider.org/?p=20173); [Snapcraft: xenia-canary](https://snapcraft.io/xenia-canary). Debugger status in Canary is not confirmed.
- **PS3**: RPCS3 is the main emulator and has a built-in PPU/SPU debugger. — [RPCS3](https://rpcs3.net/) (debugger detail not confirmed)
- **Wii**: Dolphin, with a full debugger (shared with GameCube). — [Dolphin](https://dolphin-emu.org/)
- **DS/DSi**: melonDS, DeSmuME and no$gba (debug version). — [GBATEK / no$gba](https://problemkaputt.de/gbatek.htm) (current debugger features not confirmed)
- **3DS**: Citra was shut down in March 2024. In late 2024 Lime3DS and PabloMK7's Citra fork merged into Azahar, with first public releases in early 2025; Lime3DS is discontinued (last standalone commit reported April 2025). — [Wikipedia: Citra](https://en.wikipedia.org/wiki/Citra_(emulator)); [Azahar](https://azahar-emu.org/); [tech-insider (secondary source)](https://tech-insider.org/?p=19264)
- **PSP**: PPSSPP, with a built-in debugger and a remote (web socket) debugger. — [PPSSPP](https://www.ppsspp.org/) (not confirmed)
- **Vita**: Vita3K. — [Vita3K](https://vita3k.org/) (status not confirmed)
- **Wii U**: Cemu, open source since 2022. — [Cemu GhidraRPXLoader (same org)](https://github.com/cemu-project/GhidraRPXLoader)
- **PS4**: shadPS4 runs Bloodborne; reported versions are v0.16.0 (1 June 2026) and v0.19.0 (1 October 2026); online play is unfinished (shadNet announced 25 June 2026). — [Wikipedia: ShadPS4](https://en.wikipedia.org/wiki/ShadPS4); [itechguides (secondary source)](https://www.itechguides.com/what-the-bloodborne-pc-port-is-and-how-it-works/)
- **Switch**: yuzu ended in March 2024 (see legal section). Successors: Ryubing (Ryujinx fork; official site ryujinx.app) moved to maintenance after lead GreemDev stepped away in February 2026; Eden (GPLv3, has an Android build) and Citron are described as the actively developed forks in 2026; suyu was an early yuzu fork. A February 2026 wave of takedown notices reportedly hit at least 13 Switch emulation projects including Citron, Eden and Ryubing. — [tech-insider: Eden vs Ryubing (secondary source)](https://tech-insider.org/eden-vs-ryubing-best-switch-emulator-2026/); [tech-insider (secondary source)](https://tech-insider.org/?p=23388)
- **Xbox One, PS5, Xbox Series**: no emulator that runs commercial games is known.

### Inferences
- For ground-truth traces, MAME and Mednafen give one debugger interface across many of the small platforms; that may be simpler than learning one emulator per console.
- On Switch, pinning an exact emulator build and keeping a local copy matters, because repositories can disappear.

### Gaps
- Debugger features of Xenia Canary, BigPEmu, Vita3K, Azahar and the Switch forks are not confirmed from their own docs.
- Ryujinx's end in October 2024 after Nintendo contact is widely reported (not confirmed).

## Ghidra support: built-in processors versus extensions

### Takeaway
Ghidra's built-in processors cover most CPUs here (6502, CP1600, Z80, 68000, SuperH, MIPS, PowerPC, ARM, AARCH64, x86), but not TLCS-900, V810, the Jaguar RISCs or the Cell SPU; console formats need community loaders (XEX, NDS, 3DS, RPX, Vita, PS4, NSO/NRO).

### Cited Findings
- Ghidra's `Ghidra/Processors` directory lists: 6502, 68000, 8048, 8051, 8085, AARCH64, ARM, Atmel, BPF, CP1600, CR16, DATA, Dalvik, HCS08, HCS12, Hexagon, JVM, Loongarch, M16C, M8C, MC6800, MCS96, MIPS, NDS32, PA-RISC, PIC, PowerPC, RISCV, Sparc, SuperH, SuperH4, TI_MSP430, Toy, V850, Xtensa, Z80, eBPF, tricore, x86. — [GitHub: ghidra/Ghidra/Processors](https://github.com/NationalSecurityAgency/ghidra/tree/master/Ghidra/Processors)
- Mapping (built-in): 2600/5200/7800/Lynx -> 6502 module (65C02 variant for Lynx); Intellivision -> CP1600; ColecoVision, Game Gear, NGP sound CPU -> Z80; Sega CD, Jaguar 68000, CD-i -> 68000; 32X -> SuperH (SH-2); PSP -> MIPS; Xbox 360, PS3 PPE, Wii, Wii U -> PowerPC; DS, 3DS, Vita, 3DO -> ARM; Switch -> AARCH64; PS4/Xbox One/PS5/Series -> x86 (64-bit); WonderSwan V30MZ -> x86 16-bit real mode. — same source. Vectrex 6809 is expected under the MC6800 module (6809 variant not confirmed). ARMv3 for the 3DO ARM60 is not a listed variant (not confirmed).
- V850 is built in, but the Virtual Boy's V810 is a different ISA; the extension Ghidra_v810_v830 (20Enderdude20, 7 stars, updated May 2026) adds it. — [GitHub](https://github.com/20Enderdude20/Ghidra_v810_v830)
- Cell SPU: GhidraSPU (aerosoul94, 30 stars, updated May 2026), "SPU processor implementation for Ghidra". — [GitHub](https://github.com/aerosoul94/GhidraSPU)
- PSP Allegrex: ghidra-allegrex (kotcrab, 134 stars), adds the Allegrex CPU incl. VFPU. — [GitHub](https://github.com/kotcrab/ghidra-allegrex)
- Xbox 360 XEX: XEXLoaderWV (zeroKilo, 179 stars, updated October 2026). — [GitHub](https://github.com/zeroKilo/XEXLoaderWV)
- Switch NSO/NRO/KIP: Ghidra-Switch-Loader (Adubbz, 375 stars). — [GitHub](https://github.com/Adubbz/Ghidra-Switch-Loader)
- DS: NTRGhidra (pedro-javierf), README states Ghidra 12.0.4 support, last push 24 March 2026. — [gittrend mirror](https://gittrend.io/repo/pedro-javierf/NTRGhidra)
- 3DS: ghidra-3ds (Stary2001; author says loading works, analysis "pretty broken") and ghidra-3dsx-loader (danylokos). — [GitHub: ghidra-3ds](https://github.com/Stary2001/ghidra-3ds); [GitHub: ghidra-3dsx-loader](https://github.com/danylokos/ghidra-3dsx-loader)
- Vita: GhidraVitaLoader (xerpi, 86 stars) and VitaLoaderRedux (CreepNT). — [GitHub: GhidraVitaLoader](https://github.com/xerpi/GhidraVitaLoader); [GitHub: VitaLoaderRedux](https://github.com/CreepNT/VitaLoaderRedux)
- Wii U: GhidraRPXLoader (cemu-project) uses Gekko/Broadway processor definitions if present, else the default PowerPC. — [GitHub](https://github.com/cemu-project/GhidraRPXLoader)
- Wii/GameCube: Ghidra-GameCube-Loader (Cuyler36) loads DOL/REL. — [codekk mirror](https://p.codekk.com/detail/Android/Cuyler36/Ghidra-GameCube-Loader); [WiiBrew: Ghidra](https://wiibrew.org/wiki/Ghidra)
- PS4: GhidraPS4Loader needs a decrypted ELF without the Sony header; GhidraOrbis adds Orbis OS formats (Ghidra 10.0+). — [codekk mirror](https://p.codekk.com/detail/64e02e470c2d5a3fbe2aa984)
- Vectrex: ghidra-vectrex-loader (nevesnunes). — [GitHub](https://github.com/nevesnunes/ghidra-vectrex-loader)
- WonderSwan: ghidra-wonderswan (JidouGhost-TL, 2 stars, updated October 2026) claims a V30MZ language, cartridge loader and p-code emulator for Ghidra 12. — [GitHub](https://github.com/JidouGhost-TL/ghidra-wonderswan)
- No Ghidra module is known for TLCS-900/H (only standalone disassemblers such as NGDis) or the Jaguar Tom/Jerry RISC. For the HuC6280, the community extension [TiCoKH/Ghidra_HuC6280](https://github.com/TiCoKH/Ghidra_HuC6280) exists ([Notes: CPU families](cpu-families.md)). — [Aminet: NGDis](https://se.aminet.net/dev/asm/NGDis.readme)
- Template for new modules: the PS2 Emotion Engine extension builds on Ghidra's MIPS module. — [GitHub: ghidra-emotionengine](https://github.com/astrelsky/ghidra-emotionengine/blob/master/README.md)

### Inferences
- HuC6280 code disassembles mostly correctly with the 6502 module (65C02 variant), but the extra block-transfer and MMU instructions will be wrong; the Ghidra_HuC6280 extension covers them.
- Low star counts (WonderSwan, V810) suggest testing these extensions on a known binary before trusting them.

### Gaps
- No PS3 SELF/PRX loader for Ghidra is confirmed. No Ghidra loader for PS5 or Xbox One is known.

## Notable reverse-engineering projects

### Takeaway
2025–2026 saw static recompilation spread from N64 to Xbox 360 (XenonRecomp, Unleashed Recompiled, ReXGlue) and begin on PS3, while matching decompilations reached full completion on the Wii (Super Mario Galaxy) and progressed on DS and Switch.

### Cited Findings
- **Xbox 360**: hedge-dev's XenonRecomp (PowerPC to C++) and XenosRecomp (shaders to HLSL) produced Unleashed Recompiled, a native PC port of Sonic Unleashed revealed in early March 2025; it ships without game files, needs AVX and D3D12 or Vulkan 1.2, and was inspired by N64: Recompiled. — [Unleashed Recompiled README](https://cdn.jsdelivr.net/gh/hedge-dev/unleashedrecomp@main/README.md); [Time Extension, March 2025](https://www.timeextension.com/news/2025/03/this-new-unofficial-pc-port-of-sonic-unleashed-could-be-the-best-version-of-the-game-yet)
- **ReXGlue** (2026): an open-source SDK that statically recompiles Xbox 360 PowerPC code to C++ for Windows, Linux and macOS, building on XenonRecomp and Xenia; early ports include Blue Dragon, Ninja Gaiden 2 and others; its GitHub describes it as "in early development". — [Time Extension, March 2026](https://www.timeextension.com/news/2026/03/i-hold-my-work-to-a-strict-standard-the-driving-force-behind-xbox-360-recomp-tool-rexglue-speaks-out); [Notebookcheck](https://www.notebookcheck.net/ReXGlue-SDK-debut-sparks-Xbox-360-to-PC-ports-for-Blue-Dragon-more.1231442.0.html)
- **PS3**: PS3Recomp, a static recompiler; one guide says the first playable games appeared October 2026 with no public downloads. Coverage repeating "more than three thousand games" lacks a primary source. — [Generation Amiga, June 2026](https://www.generationamiga.com/2026/06/15/ps3recomp-the-new-tool-bringing-native-playstation-3-ports-closer-to-pc/); [Held Games (secondary source)](https://heldgames.com/tags/ps3recomp)
- **Wii**: Super Mario Galaxy reported 100% decompiled in late September 2026 by SMGCommunity, the first Wii game to reach that; a human-written PC port, AstroCore, is in progress at Luma's Workshop; SMG2 and Skyward Sword decomps are under way. Uses decomp-toolkit (dtk). — [Android Authority](https://androidauthority.com/super-mario-galaxy-decompiled-pc-port-3717193); [GitHub: Petari](https://github.com/Gota7/Petari); [decomp.dev](https://decomp.dev/)
- **DS**: pret/pokeplatinum (C decompilation, actively pushed in 2026), plus pokeheartgold; an SDL3 port fork (pokeplatinum-portable) is in progress. — [gittrend: pokeplatinum](https://gittrend.io/repo/pret/pokeplatinum); [GitHub: pokeplatinum-portable](https://github.com/arcanite24/pokeplatinum-portable)
- **Switch**: Breath of the Wild v1.5.0 decompilation (work in progress; no assets; goals are understanding internals and glitch hunting). — [GitHub: botw](https://github.com/german77/botw)
- **PSP**: the Castlevania: Symphony of the Night matching decompilation includes a PSP build (Dracula X Chronicles); splat supports PSP binaries. — [GitHub: splat](https://github.com/ethteck/splat)
- **Atari 2600**: commented disassemblies of many 2600 games are published on AtariAge and similar sites; Stella's debugger produces annotated disassembly. (not confirmed) — [AtariAge](https://atariage.com/)

### Inferences
- PowerPC consoles (360, Wii, Wii U, PS3 PPE) now share tooling: dtk-style split-and-match for Nintendo, XenonRecomp/ReXGlue-style recompilation for Microsoft, which suggests the guide can treat "big-endian PowerPC" as one family.

### Gaps
- No completed PSP, Vita, 3DS or PS4 matching decomp is known, and the Super Mario Odyssey decomp status is unknown.

## Legal and practical notes

### Takeaway
From the Xbox 360 onward an emulator, loader or recompiler needs decrypted executables or console keys that the user must extract from their own hardware; emulator projects for current Nintendo hardware have faced legal action, and the guide should state those facts and leave choices to the user.

### Cited Findings
- On 4 March 2024 Tropic Haze (yuzu's developer) settled with Nintendo for $2.4 million; a Rhode Island federal court entered a permanent injunction barring it from offering yuzu or its code and from making software that circumvents Nintendo's protections; the yuzu-emu.org domain went to Nintendo. — [Android Authority](https://androidauthority.com/nintendo-switch-yuzu-settlement-3422388); [Nintendo World Report](https://www.nintendoworldreport.com/news/66510/creators-of-yuzu-emulator-settle-lawsuit-with-nintendo)
- Citra, run by overlapping developers, was taken down at the same time. — [Wikipedia: Citra](https://en.wikipedia.org/wiki/Citra_(emulator))
- In February 2026 a wave of takedown notices reportedly hit at least 13 Switch emulation projects. (secondary source) — [tech-insider](https://tech-insider.org/?p=23388)
- Switch: NCA contents are encrypted; emulators need console keys (prod.keys) and title keys dumped from the user's own console, plus firmware. — [switchbrew: NCA](https://switchbrew.org/wiki/NCA) (not confirmed)
- PS3: SELF executables are signed and encrypted; RPCS3 requires the official firmware file, and disc dumps need a disc key. — [psdevwiki: SELF](https://www.psdevwiki.com/ps3/SELF_-_SPRX); [RPCS3 quickstart](https://rpcs3.net/quickstart) (not confirmed)
- Xbox 360: XEX2 executables are encrypted and signed; loaders and Xenia decrypt them with the console-family key. — [XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV) (detail not confirmed)
- PS4: Ghidra loaders need decrypted ELF without the Sony header. — [codekk mirror](https://p.codekk.com/detail/64e02e470c2d5a3fbe2aa984)
- 3DS: content is AES-encrypted with keys in the ARM9 boot ROM. — [3dbrew](https://www.3dbrew.org/)
- Lynx and 7800: the boot ROM checks a signature on the cartridge; emulators need the boot ROM (Lynx) or skip the check. — [Wikipedia: Atari Lynx](https://en.wikipedia.org/wiki/Atari_Lynx); [Wikipedia: Atari 7800](https://en.wikipedia.org/wiki/Atari_7800)
- Recompilation projects (Unleashed Recompiled, BotW decomp, GoldenEye 360 recomp) ship no game files and require the user's own copy. — [Unleashed Recompiled README](https://cdn.jsdelivr.net/gh/hedge-dev/unleashedrecomp@main/README.md); [tbreak](https://tbreak.com/goldeneye-007-xbox-360-pc-recompilation/)

### Inferences
- The harness's existing rule (no game material in the repository) maps directly onto these projects' practice: tools and source only, with the user supplying files and keys from hardware they own.

### Gaps
- No court ruling (as opposed to settlement) on console emulator legality in 2024–2026 is known. Jurisdiction-specific circumvention rules are outside this note.
