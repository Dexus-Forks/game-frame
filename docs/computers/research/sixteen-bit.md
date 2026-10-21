# Notes: 16-bit and 32-bit home computers

Covers the 16-bit and 32-bit home computers as game platforms: machines and chips, system takeover, disk formats and protection, emulators, Ghidra and Unicorn support, and reimplemented games.

## Machines, CPUs, RAM, chipsets and operating systems

### Takeaway
Three families share CPU families. The Amiga, Atari ST and X68000 use the big-endian Motorola 68000 line. The Archimedes and Risc PC use ARM. The PC-98 and FM Towns use x86. Each machine's custom video and sound chips define how its games are written. Exact clocks are confirmed for most models, but the Amiga PAL and NTSC CPU clocks are not confirmed.

### Cited Findings

**Commodore Amiga**
- A1000: 1985–1987, 68000, OCS, 256 KB chip RAM (later units 512 KB), Kickstart 1.0–1.1 — [Wikipedia: Amiga models and variants](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- A500: 1987–1991, 68000, OCS, 512 KB base (later units 1 MB), Kickstart 1.2–1.3 — [same](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- A2000: 1987–1992, 68000, OCS, Kickstart 1.2–2.04; the A-model had 512 KB chip plus 512 KB on a CPU slot card — [same](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- A500+: 1991–1992, 68000, ECS, 1 MB chip RAM, Kickstart 2.04. A600: 1992, 68000, ECS, 1 MB, Kickstart 2.05 — [same](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- A3000: 1990–1992, 68030, ECS, 1 MB chip and 1–4 MB fast RAM, Kickstart 1.3–2.04 — [same](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- A1200: 1992–1996, 68EC020, AGA, 2 MB chip RAM, Kickstart 3.0–3.1. CD32: 1993–1994, 68EC020, AGA, 2 MB, Kickstart 3.1. A4000: 1992–1994, 68EC030 or 68040, AGA, 2 MB chip and 2–4 MB fast RAM, Kickstart 3.0 — [same](https://en.wikipedia.org/wiki/Amiga_models_and_variants)
- Agnus is the DMA controller for chip RAM, with a priority system between the CPU and custom chips; it holds the Blitter and the Copper. The original Agnus addresses 512 KB of chip RAM; "Fat Agnus" revisions raised this to 1 MB (ECS) and then 2 MB. Timing is in 280 ns colour clocks — [Wikipedia: Original Chip Set](https://en.wikipedia.org/wiki/Original_Chip_Set)
- The Blitter copies, fills and draws lines. It combines up to three sources (A, B, C) into a destination (D), on blocks up to 1,024×1,024 pixels — [same](https://en.wikipedia.org/wiki/Original_Chip_Set)
- The Copper runs a program synchronised to the video beam. It has three instructions, MOVE, WAIT and SKIP, and restarts at the start of each frame — [same](https://en.wikipedia.org/wiki/Original_Chip_Set)
- Denise: 320 or 640 pixels wide, 200 (NTSC) or 256 (PAL) lines without overscan. 1–5 bitplanes give 2–32 colours from 4,096. A sixth bitplane enables Extra-Halfbrite and Hold-And-Modify (HAM); HAM can show all 4,096 colours. Eight 16-pixel-wide sprites with three colours plus transparent. Dual playfield and interlace. No text mode — [same](https://en.wikipedia.org/wiki/Original_Chip_Set)
- Paula: interrupt controller, audio, floppy controller, serial port, and mouse/joystick buttons. Four 8-bit DMA audio channels, two left and two right, each with volume control (the article gives 64 levels in one place and 65 in another). Maximum DMA rate about 28,867 samples/s per channel (NTSC), 28,837 (PAL). Floppy controller at 500 or 250 kbit/s; native format 880 KB on 3.5-inch disks. Serial port has no FIFO — [same](https://en.wikipedia.org/wiki/Original_Chip_Set)

**Atari ST family**
- 520ST: June 1985; 1040ST: 1986. Motorola 68000 at 8 MHz. 512 KB or 1 MB. Yamaha YM2149 3-voice PSG. MC68901 MFP. WD-1772 floppy controller — [Wikipedia: Atari ST](https://en.wikipedia.org/wiki/Atari_ST)
- Shifter video modes: 320×200 in 16 colours, 640×200 in 4 colours, both from 512; 640×400 monochrome at 71.2 Hz — [same](https://en.wikipedia.org/wiki/Atari_ST)
- STE (late 1989): Blitter, 4,096-colour palette, hardware fine scrolling and split screen, 8-bit stereo DMA PCM at 6.25/12.5/25/50 kHz, LMC1992 audio controller, SIMM RAM to 4 MB, TOS 1.06 or 1.62 — [same](https://en.wikipedia.org/wiki/Atari_ST)
- Mega STE (1991): 68000 at 16 MHz with a 16 KB cache, SCSI, 1.44 MB floppy, TOS 2.00. TT030 (1990): 68030 at 32 MHz, two MFPs. Falcon030 (1992): 68030 at 16 MHz, Motorola 56001 DSP, 16-bit bus, discontinued 1993 — [same](https://en.wikipedia.org/wiki/Atari_ST)
- Original OS is TOS 1.00 with Digital Research GEM. Floppies: 360 KB single-sided at first, then 720 KB double-sided — [same](https://en.wikipedia.org/wiki/Atari_ST)

**Acorn Archimedes and Risc PC**
- A305/A310/A440: July 1987, ARM2 at 8 MHz, Arthur OS. A3000: May 1989, ARM2 8 MHz, RISC OS 2. A540: June 1990, ARM3. A5000: September 1991, ARM3 at 25 or 33 MHz, RISC OS 3.0. A4 laptop: June 1992, ARM3 at 24 MHz. A3010/A3020/A4000: September 1992, ARM250 at 12 MHz, RISC OS 3.10/3.11 — [Wikipedia: Acorn Archimedes](https://en.wikipedia.org/wiki/Acorn_Archimedes)
- Chipset: MEMC (memory controller), VIDC (video and sound), IOC (I/O). The ARM250 combines ARM2, IOC1, VIDC1a and MEMC1a on one chip — [same](https://en.wikipedia.org/wiki/Acorn_Archimedes)
- VIDC1 sound: 8 channels of 8-bit logarithmic stereo. Video modes include 640×256 in 256 colours, 640×512 in 16 colours and 1152×864 mono — [same](https://en.wikipedia.org/wiki/Acorn_Archimedes)
- Native ADFS floppy: 800 KB (DD) or 1.6 MB (HD). The A5000 was the first with an HD drive and reads MS-DOS and Atari ST 720 KB/1.44 MB disks — [same](https://en.wikipedia.org/wiki/Acorn_Archimedes)
- Risc PC: launched 15 April 1994. ARM610 at 30 or 33 MHz, ARM710 at 40 MHz, StrongARM at 203, 236 or 300 MHz. VIDC20 video. Two 72-pin FPM SIMM slots to 256 MiB, optional VRAM to 2 MiB. RISC OS 3.50 (Risc PC 600), 3.60 (700), 3.70 (StrongARM), 3.71 (J233). Acorn stopped in 1998; Castle Technology made it until 2003 — [Wikipedia: Risc PC](https://en.wikipedia.org/wiki/Risc_PC)

**Sharp X68000**
- Released March 1987, Japan only. Hitachi HD68HC000 (68000 clone) at 10 MHz. XVI (May 1991) and Compact (February 1992) at 16 MHz. X68030 (March 1993): MC68EC030 at 25 MHz, 4 MB — [Wikipedia: X68000](https://en.wikipedia.org/wiki/X68000)
- RAM 1–4 MB, expandable to 12 MB. VRAM 1056 KB: 512 KB graphics, 512 KB text, 32 KB sprites. 65,536-colour palette. Modes 256×240 to 1024×1024 — [same](https://en.wikipedia.org/wiki/X68000)
- Sprites: 128 on screen, 32 per line, 256 patterns, 16×16. Raster interrupts can raise the count to 512 — [same](https://en.wikipedia.org/wiki/X68000)
- Sound: Yamaha YM2151 (OPM), 8 FM channels; YM3012 DAC; Oki MSM6258 one 4-bit ADPCM channel at 15.6 kHz — [same](https://en.wikipedia.org/wiki/X68000)
- OS: Human68k, MS-DOS-like, made by Hudson Soft. Floppy: 77 tracks, 2 heads, 8 sectors of 1024 bytes, 360 rpm (1232 KiB) on 5.25-inch 1.2 MB drives — [same](https://en.wikipedia.org/wiki/X68000)

**NEC PC-9801**
- Launched October 1982: Intel 8086 at 5 MHz, 128 KB (to 640 KB). Later: NEC V30 (PC-9801VM, July 1985, 10 MHz), 80286 (VX, 1986), 80386 (RA, 1988) — [Wikipedia: PC-98](https://en.wikipedia.org/wiki/PC-98)
- Two NEC μPD7220 display controllers, one for text and one for graphics. 640×400 in 8 colours; graphics mode supports 16 from 4,096. GRCG on the VM; EGC blitter from the VX (1986). Kanji ROM optional at first; PC-9801F added JIS level 1 — [same](https://en.wikipedia.org/wiki/PC-98)
- Sound: early models had only a buzzer. Optional PC-9801-26 card based on the YM2203; PC-9801-86 (1993) uses the YM2608 — [same](https://en.wikipedia.org/wiki/PC-98)
- OS: MS-DOS (NEC bundled MS-DOS 2.11 from 1983 to 1987), plus N88-BASIC(86), CP/M-86, Windows and others. Last model PC-9821Ra43 (2000); discontinued 30 September 2003; 18.3 million shipped — [same](https://en.wikipedia.org/wiki/PC-98)

**Fujitsu FM Towns**
- Released 28 February 1989, Japan; discontinued 1997; about 500,000 sold. Intel 80386DX at 16 MHz. Single-speed CD-ROM as standard — [Wikipedia: FM Towns](https://en.wikipedia.org/wiki/FM_Towns)
- Video: 320×200 to 720×512, 256 colours from 32,768, up to 1,024 16×16 sprites. Sound: Ricoh RF5c68 (8 PCM channels) and Yamaha YM2612 (6 FM channels) — [same](https://en.wikipedia.org/wiki/FM_Towns)
- OS: Towns OS, based on MS-DOS with the Phar Lap DOS extender. FM Towns Marty console (1993) runs Towns games — [same](https://en.wikipedia.org/wiki/FM_Towns)

### Inferences
- The 68000 machines are big-endian; the ARM machines run little-endian; the x86 machines are little-endian. Data files moved between platforms (ST to Amiga, Amiga to DOS) may change byte order.
- Amiga games built for OCS chip-RAM limits (512 KB) behave differently from ECS/AGA builds, so the target model and Kickstart version must be recorded with each capture.
- ARM2 and ARM3 use the 26-bit address mode, with status flags in R15. Risc PC CPUs added 32-bit mode. Old Archimedes games depend on 26-bit behaviour.

### Gaps
- Amiga CPU clocks (PAL 7.09 MHz, NTSC 7.16 MHz; A1200 68EC020 at 14 MHz; A4000 at 25 MHz) are not confirmed.
- The Copper, sprite and HAM details come from a Wikipedia article that asks for more citations. Check against the Amiga Hardware Reference Manual.
- Sinclair QL (1984, Motorola 68008) and SAM Coupé (1989, Z80B) lack sources.
- Exact Archimedes VIDC/MEMC function details and the Risc PC's IOMD are not covered by the cited pages.
- FM Towns model-year table and Towns OS version history lack a source.

## System takeover, timing and the OS

### Takeaway
Amiga and ST games often skip the OS and drive the custom chips directly. A reimplementation then has to rebuild chip behaviour, not OS calls. No cited source counts how many games do this.

### Cited Findings
- The Copper restarts every frame and is synchronised to the beam, so raster effects (palette changes per line) come from a per-frame Copper list — [Wikipedia: Original Chip Set](https://en.wikipedia.org/wiki/Original_Chip_Set)
- Display height differs by TV standard: 200 lines NTSC, 256 lines PAL; Paula DMA rates also differ by standard — [same](https://en.wikipedia.org/wiki/Original_Chip_Set)
- Frontier: Elite II was ported by Tom Morton, who started from the Atari ST version, stripped the hardware access in the assembly and changed a 68k assembler to emit C or i386 assembly — [virtuallyfun.com](https://virtuallyfun.com/?p=59)

### Inferences
- A takeover game's timing ties to the vertical blank (50 Hz PAL, 60 Hz NTSC). A port must pick one and record it.
- A game that uses the OS (GEM, AmigaOS libraries, RISC OS SWIs, Human68k or MS-DOS calls) can be traced at the OS call boundary. A takeover game must be traced at hardware register writes.

### Gaps
- No cited source gives a count or list of OS-friendly against takeover games.

## Disk formats and copy protection

### Takeaway
Plain sector images (ADF, ST) cannot hold most copy protection. Protected disks need IPF (Amiga and others) or STX (Atari ST). Copylock used a trace vector decoder on the Amiga, ST and PC.

### Cited Findings
- IPF is the Software Preservation Society's format. Files start with the ASCII letters "CAPS". The official library's source is available. IPF holds low-level data and is often made from KryoFlux captures — [Wikipedia: Amiga Disk File](https://en.wikipedia.org/wiki/Amiga_Disk_File); [IANA: vnd.softpres-ipf-disk-image](https://www.iana.org/assignments/media-types/application/vnd.softpres-ipf-disk-image)
- SPS notes that automatic format scripting is not possible for the Amiga due to its free-form track format — [SPS news 2005-08-05](https://softpres.org/news:2005-08-05)
- SPS support includes Copylock Amiga and Copylock ST, and Speedlock timing tracks — [SPS 2002-10-28](https://softpres.org/wip:2002-10-28); [SPS Speedlock](https://softpres.org/wip:2003-06-01)
- Rob Northen Copylock protected disks on the Amiga, Atari ST and IBM PC. Its trace vector decoder decrypts each instruction, runs it, then re-encrypts it — [Wikipedia: Rob Northen copylock](https://en.wikipedia.org/wiki/Rob_Northen_copylock); [Wikipedia: Trace vector decoder](https://en.wikipedia.org/wiki/Trace_vector_decoder)
- CAPS/SPS reverse engineered the Amiga Copylock in June 2001 and reported over 500 titles using it — [SPS Copylock Unlocked](https://softpres.org/wip:2001-06-07); [Part 2](https://softpres.org/wip:2001-06-08)
- Later Copylock keydisks carried a unique serial number in the disk format — [Codetapper interview with Rob Northen](https://codetapper.com/amiga/interviews/rob-northen)
- Atari ST: .ST is raw data with no metadata and no protection; MSA adds a geometry header and simple RLE packing; STX (Pasti) stores non-standard sectors, fake track addresses and sector timing — [fplanque.com](https://www.fplanque.com/tech/retro/atari/atari-st-fd-image-file-formats/); [8bitchip forum](https://forum.8bitchip.info/atari-faq/disk-imaging-popular-formats/)
- STX was never officially documented by its author; third parties documented it. Hatari 1.8.0 (2014) added the first open-source STX decoder — [IANA: vnd.pasti-stx](https://www.iana.org/assignments/media-types/application/vnd.pasti-stx-disk-image); [st-computer.org](https://en.st-computer.org/news/2014/07/06/hatari-1-8-0-supports-pasti/)
- PC-98 images: floppy FDI, FDD, D88/D98, HDM, NFD; hard disk HDI, NHD. np2kai needs BIOS, font and sound ROMs — [libretro: Neko Project II Kai](https://docs.libretro.com/library/neko_project_ii_kai/); [Batocera wiki](https://wiki.batocera.org/systems:pc98)

### Inferences
- A game whose loader decrypts itself (Copylock TVD) is best read after decryption, from emulator memory, not from the disk image.

### Gaps
- Archimedes ADF and APD image formats lack sources.
- X68000 image formats (XDF, DIM, HDM) lack sources beyond the physical geometry.

## Emulators, debuggers and documentation

### Takeaway
Hatari has the best documented debugger of these machines. WinUAE has a built-in debugger opened with Shift+F12. The debuggers of the other emulators are not confirmed.

### Cited Findings
- Hatari debugger: conditional breakpoints (up to four conditions), memory dump, CPU and DSP disassembly, symbol loading from programs, profiling with callgraph and Callgrind output, tracing of OS calls (GEMDOS, BIOS, XBIOS, AES, VDI), and command scripts at startup and on breakpoints — [Hatari debugger manual](https://hatari.tuxfamily.org/doc/debugger.html)
- WinUAE's debugger opens with Shift+F12; `fp "name"` runs until a named process starts — [markwrobel.dk](https://markwrobel.dk/post/amiga-machine-code-part1-debugger)
- WinUAE supports memory read/write watchpoints; Copper lists can be stepped in the VS Code Amiga extension with FS-UAE — [vscode-amiga-assembly wiki](https://github-wiki-see.page/m/prb28/vscode-amiga-assembly/wiki/Debugging-emulator)
- MAME has a debugger with watchpoints — [MAME docs](https://docs.mamedev.org/debugger/watchpoint.html)

### Gaps
- vAmiga, Steem SSE, Arculator, RPCEmu, XM6, PX68k and Unz debuggers are not confirmed.
- The Amiga Hardware Reference Manual, ST Internals, English Amiga Board, Atari-Forum and Stardot are not cited here.

## Ghidra and Unicorn support

### Takeaway
A Ghidra extension loads Amiga hunk executables and Kickstart ROMs. Unicorn lists M68K and ARM. Atari ST PRG and RISC OS module loaders are not known.

### Cited Findings
- BartmanAbyss/ghidra-amiga, forked from lab313ru's ghidra_amiga_ldr, loads Amiga hunk executables, Kickstart ROMs and WinUAE state files (.uss), with NDK 3.9 symbols. It builds on work by Vladimir Kononovich and Martin Atkins — [BartmanAbyss on GitHub](https://www.github.com/BartmanAbyss); [amiga-news 2022](https://amiga-news.de/en/news/AN-2022-09-00023-EN.html)
- A 2023 update targeted Ghidra 10.3.1 — [amiga-news 2023](https://amiga-news.de/en/news/AN-2023-06-00122-EN.html)
- One 2026 report says the build for Ghidra 12.0.1 would not load in 12.1.3 until its version line was edited (single user report) — [Sebastian Bergmann](https://amiga.sebastian-bergmann.de/2026/08/29/disassembling-my-1994-self/)
- lab313ru also wrote a Sega Mega Drive loader for Ghidra (68000) — [spritesmind forum](https://gendev.spritesmind.net/forum/viewtopic.php?p=38020)
- Unicorn supports "ARM, ARM64 (ARMv8), M68K, MIPS, PowerPC, RISCV, SPARC, S390X, TriCore and X86 (16, 32, 64-bit)" and is based on QEMU — [Unicorn README](https://raw.githubusercontent.com/unicorn-engine/unicorn/master/README.md)
- amitools' hunk library (Python) loads hunk binaries — [lab313ru/amitools_dbg](https://github.com/lab313ru/amitools_dbg)

### Inferences
- Unicorn emulates CPUs only. Amiga or ST custom chips are not emulated, so functions that touch chip registers need stubs or a full emulator.
- QEMU-based ARM emulation is likely to lack ARM2/ARM3 26-bit mode; Archimedes code may need a 26-bit-aware emulator such as Arculator.

### Gaps
- Unicorn's exact M68K CPU models (68000 versus 020/030/040) are not confirmed.
- Ghidra's built-in 68000 and ARM processor modules and their 26-bit ARM support lack sources here.
- No Atari ST PRG/TOS or RISC OS module loader for Ghidra is known.

## Reimplemented games

### Takeaway
Known projects took different routes: assembly translation (glFrontier), bytecode engine rewrites (rawgl), engine rewrites (OpenFodder), and reconstructed source (ReDMCSB).

### Cited Findings
- glFrontier: Tom Morton converted the Atari ST Frontier: Elite II 68k assembly to C or i386 via a modified assembler, and added OpenGL — [virtuallyfun.com](https://virtuallyfun.com/?p=59); [Aminet readme](https://se.aminet.net/game/actio/frontier-elite2-gl.aros.readme)
- fe2-intro reproduces the Frontier intro — [watsonmw/fe2-intro](https://github.com/watsonmw/fe2-intro)
- rawgl (by cyxx) reimplements the Another World engine in C++ with SDL and OpenGL. It reads Amiga, Atari, DOS, Windows 3.1, 3DO, 15th and 20th Anniversary data. Archived 28 June 2025 — [cyxx/rawgl](https://github.com/cyxx/rawgl)
- OpenFodder is an open-source Cannon Fodder engine for Cannon Fodder (Amiga, CD32, DOS CD) and Cannon Fodder 2 (Amiga, DOS CD), GPL-3.0 — [OpenFodder](https://github.com/OpenFodder/openfodder)
- ReDMCSB, maintained by forum user ChristopheF (full name not confirmed), is reconstructed Dungeon Master and Chaos Strikes Back source; its build scripts make Atari ST and Amiga executables with an option to drop copy protection — [dungeon-master.com forum](https://www.dungeon-master.com/forum/viewtopic.php?p=163896)
- DM/CSB version 3 was built with Think C on Apple hardware — [dungeon-master.com forum](https://www.dungeon-master.com/forum/viewtopic.php?p=162234)
- pyDM is a Python port of Amiga Dungeon Master built from the ReDMCSB Amiga V3.x branch — [pyDM](https://codeberg.org/CyCl0ne/pyDM)

### Gaps
- OpenTitus, REminiscence (Flashback), Lemmings ports, Sensible Soccer, Settlers and Populous projects are not confirmed.
- ReDMCSB's main page is dmweb.free.fr. Its method and binary-matching claims are not confirmed.
- OpenFodder's README does not say how it was made.
