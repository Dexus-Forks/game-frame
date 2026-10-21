# Notes: 8-bit home computers

Covers the 8-bit home computers as game platforms: hardware, loaders and protection, image formats, emulators and notable projects. A fact marked "(not confirmed)" is a lead. Check it at the linked page before relying on it.

## Hardware per machine: release, CPU, RAM, memory map, ROM and banking, video, sound, timing

### Takeaway
Two CPU families cover almost everything: the Zilog Z80 (ZX81, Spectrum, CPC, MSX, PC-8801, X1) and the MOS 6502 family (BBC/Electron, VIC-20, C64, Atari 8-bit, Oric, Apple II), with the Motorola 6809E in the Dragon and CoCo. Exact timing matters most on the Spectrum (contended memory), the C64 (VIC-II badlines and raster interrupts), the Atari (ANTIC DMA and display list interrupts) and the CPC (wait states round every access to 4 cycles).

### Cited Findings

**Sinclair ZX81**
- Launched in the UK on 5 March 1981; made by Timex in Dundee; £49.95 kit, £69.95 assembled; discontinued 1984; over 1.5 million sold — [Wikipedia: ZX81](https://en.wikipedia.org/wiki/ZX81)
- Z80A at 3.25 MHz (the infobox and ULA clock division give 3.25 MHz; one section of the article says 3.5 MHz, so the article conflicts with itself) — [Wikipedia: ZX81](https://en.wikipedia.org/wiki/ZX81)
- 1 KB RAM, 16 KB RAM pack; 8 KB ROM with Sinclair BASIC; a Ferranti ULA replaced about 18 ZX80 chips (four chips total) — [Wikipedia: ZX81](https://en.wikipedia.org/wiki/ZX81)
- The CPU generates the display. SLOW mode computes and displays; FAST mode blanks the display to run faster. 32x24 text, 64x48 block graphics, monochrome — [Wikipedia: ZX81](https://en.wikipedia.org/wiki/ZX81)
- Tape at roughly 250-300 bit/s. Timex variants TS1000 and TS1500 — [Wikipedia: ZX81](https://en.wikipedia.org/wiki/ZX81)
- The ZX81 tape encoding differs from the Spectrum's, so TZX does not describe it natively — [TZX format](https://worldofspectrum.net/TZXformat.html)

**ZX Spectrum (16K/48K, 128K, +2, +2A/+3)**
- 48K launched in the UK on 23 April 1982 (£175); West Germany 1983; US November 1983 — [Wikipedia: ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum)
- 128K launched in Spain in September 1985 and in the UK in January 1986 (£179). +2 after Amstrad bought the range in 1986 (£149). +3 in 1987 (£249). +2A sold from late 1988 — [Wikipedia: ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum)
- CPU Z80A at 3.5 MHz; ULA designed by Richard Altwasser — [Wikipedia: ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum). The 128K clock is 3.5469 MHz (not confirmed) — [Sinclair Wiki: 128K](https://sinclair.wiki.zxnet.co.uk/wiki/ZX_Spectrum_128)
- 128K models add memory as eight 16 KB pages by bank switching; AY-3-8912 gives three channels — [Wikipedia: ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum)
- Paging register is port 0x7FFD (bits 0-2 RAM page at 0xC000, bit 3 shadow screen page 7, bit 4 ROM select, bit 5 lock); the +2A/+3 add port 0x1FFD for special all-RAM modes and the second ROM bit (not confirmed) — [Sinclair Wiki: ZX Spectrum 128](https://sinclair.wiki.zxnet.co.uk/wiki/ZX_Spectrum_128)
- 48K memory map: ROM 0x0000-0x3FFF; screen bitmap 0x4000-0x57FF; attributes 0x5800-0x5AFF (not confirmed) — [Sinclair Wiki](https://sinclair.wiki.zxnet.co.uk/wiki/ZX_Spectrum_48K)
- Display 256x192, 15 colours, colour in a 32x24 grid of 8x8 attribute cells (ink, paper, bright, flash), which causes attribute clash — [Wikipedia: ZX Spectrum](https://en.wikipedia.org/wiki/ZX_Spectrum)
- Contended memory, 48K: 0x4000-0x7FFF contended. First contended cycle is T-state 14335 after the interrupt; delay pattern 6,5,4,3,2,1,0,0 per 8 T-states; 128 T-states of contention then a 96 T-state gap per line, so 224 T-states per line. "Late timing" ULAs run one T-state later — [Sinclair Wiki: Contended memory](https://sinclair.wiki.zxnet.co.uk/wiki/Contended_memory)
- Contended memory, 128K/+2: pages 1, 3, 5, 7 contended; pattern starts at T-state 14361; 228 T-states per line — [Sinclair Wiki: Contended memory](https://sinclair.wiki.zxnet.co.uk/wiki/Contended_memory)
- +2A/+3 (Amstrad gate array): pages 4, 5, 6, 7 contended; contention only when MREQ is active; pattern 7,6,5,4,3,2,1,0,0 from 14363; no late-timing variants — [Sinclair Wiki: Contended memory](https://sinclair.wiki.zxnet.co.uk/wiki/Contended_memory)
- Frame length 69888 T-states (48K) and 70908 T-states (128K), giving about 50.08 Hz and 50.02 Hz; one maskable interrupt per frame, IM 2 used by most games (not confirmed) — [Sinclair Wiki: Contended memory](https://sinclair.wiki.zxnet.co.uk/wiki/Contended_memory); [World of Spectrum FAQ](https://worldofspectrum.org/faq/reference/48kreference.htm)

**BBC Micro (A, B, B+, Master 128) and Acorn Electron**
- Model A/B on 1 December 1981, most deliveries in 1982; B+ mid-1985; B+128 late 1985; Master 1986 — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro)
- 2 MHz 6502 (6502A in A/B; Rockwell 6512A in B+). RAM: A 16 KB, B 32 KB, B+64 64 KB, B+128 128 KB, Master 128 KB — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro). The Master uses a 65SC12 (65C02-class) CPU (not confirmed) — [Wikipedia: BBC Master](https://en.wikipedia.org/wiki/BBC_Master)
- Acorn MOS in a 16 KB ROM, last official A/B version OS 1.2. Four 16 KB sideways (paged) ROM sockets, expandable to sixteen — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro). Paged ROM select register ROMSEL at &FE30, sideways area &8000-&BFFF, MOS at &C000-&FFFF, SHEILA I/O page &FE00 (not confirmed) — [Stardot / BeebWiki](https://beebwiki.mdfs.net/Paged_ROM)
- Video: Motorola 6845 CRTC for modes 0-6 (2 to 16 colours); mode 7 Teletext with the Mullard SAA5050 and 1 KB RAM — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro). The Video ULA does serialisation and the palette (not confirmed) — [BeebWiki: Video ULA](https://beebwiki.mdfs.net/Video_ULA)
- Sound: TI SN76489, three tone channels and one noise channel — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro)
- DFS from 1982 on the Intel 8271; the B+ moved to the WD1770, which broke some software — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro)
- Electron from 1983, a simplified 32 KB home version — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro). Its single ULA replaces the Video ULA, CRTC and sound; RAM is 4-bit wide, so the CPU drops to an effective 1 MHz (or lower in modes 0-3) when touching RAM; no mode 7; one-channel sound (not confirmed) — [Wikipedia: Acorn Electron](https://en.wikipedia.org/wiki/Acorn_Electron)

**Commodore VIC-20 and Commodore 64**
- VIC-20: Japan 1980 as VIC-1001, worldwide 1981 at US$299.95; first computer to sell one million; 2.5 million sold; discontinued January 1985 — [Wikipedia: VIC-20](https://en.wikipedia.org/wiki/Commodore_VIC-20)
- VIC-20: MOS 6502, 5 KB RAM (about 3.5 KB free to BASIC), VIC chip with 22x23 text and 16 colours, three square-wave voices plus noise, cartridge slot, serial bus, Datasette — [Wikipedia: VIC-20](https://en.wikipedia.org/wiki/Commodore_VIC-20). VIC chips are 6560 (NTSC) and 6561 (PAL); clocks 1.0227 MHz NTSC and 1.1087 MHz PAL; two 6522 VIAs (not confirmed) — [Wikipedia: MOS VIC](https://en.wikipedia.org/wiki/MOS_Technology_VIC)
- C64: shown at CES 7-10 January 1982, marketed August 1982 — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64)
- C64: MOS 6510 at 1.023 MHz NTSC and 0.985 MHz PAL; 64 KB DRAM plus 1 KB 4-bit colour RAM; 20 KB ROM (BASIC, KERNAL, character ROM); the 6510's on-chip port at $01 switches ROMs and the $D000 I/O area in and out — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64)
- C64 memory map: BASIC ROM $A000-$BFFF, I/O or character ROM $D000-$DFFF, KERNAL $E000-$FFFF; VIC-II at $D000, SID at $D400, CIAs at $DC00 and $DD00; CIA 2 port A selects the VIC-II's 16 KB bank (not confirmed) — [C64 Wiki: Memory Map](https://www.c64-wiki.com/wiki/Memory_Map)
- C64 video: 320x200, multicolour 160x200, eight hardware sprites, raster interrupts — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64). VIC-II models 6567 (NTSC, 263 lines x 65 cycles, early 6567R56A 262 x 64) and 6569 (PAL, 312 lines x 63 cycles, 19656 cycles per frame, 50.12 Hz); "badlines" steal 40 cycles every eighth line (not confirmed) — [Christian Bauer, The MOS 6567/6569 VIC-II](https://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt)
- C64 sound: SID 6581 or 8580, three voices, ADSR, filter, ring modulation — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64). The two revisions differ audibly, mainly in the filter (not confirmed) — [Wikipedia: SID](https://en.wikipedia.org/wiki/MOS_Technology_6581)
- C64 cartridges: auto-start signature "CBM80" at $8000; about 300 cartridges, most up to 16 KB — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64)
- 1541 drive has its own 6502 and ROM — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64). This is why fast loaders and disk protection can run custom code on the drive

**Amstrad CPC (464, 664, 6128, Plus)**
- 464 June 1984 (UK); 664 April 1985; 6128 June 1985 (US), August 1985 (Europe); 464 plus, 6128 plus and GX4000 September 1990 — [Wikipedia: Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC)
- Z80A at 4 MHz; wait states give an effective rate of about 3.3 MHz — [Wikipedia: Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC). In practice every instruction is stretched to a multiple of 4 T-states ("NOP time", 1 microsecond) (not confirmed) — [CPCWiki: CPU timing](https://www.cpcwiki.eu/index.php/Z80)
- 64 KB (464, 664) or 128 KB (6128); 6845 CRTC (or compatible) plus a custom gate array; modes 160x200x16, 320x200x4, 640x200x2; 27-colour palette, 4096 on Plus; AY-3-8912; 50 Hz — [Wikipedia: Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC)
- CRTC types 0-4 differ in edge behaviour, which matters for split-screen and overscan demos and some games (not confirmed) — [CPCWiki: CRTC](https://www.cpcwiki.eu/index.php/CRTC)
- Gate array raises an interrupt every 52 scan lines, six per frame (not confirmed) — [CPCWiki: Gate Array](https://www.cpcwiki.eu/index.php/Gate_Array)
- 3-inch disks, typically 180 KB per side; AMSDOS, Locomotive BASIC, CP/M on disk models — [Wikipedia: Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC)

**MSX, MSX2**
- Announced by ASCII on 16 June 1983; first machine sold 21 October 1983 (Mitsubishi ML-8000). MSX2 1985, MSX2+ 1988 (Japan only), turbo R announced 1990 — [Wikipedia: MSX](https://en.wikipedia.org/wiki/MSX)
- Z80A at 3.58 MHz (MSX to MSX2+); turbo R uses the R800 — [Wikipedia: MSX](https://en.wikipedia.org/wiki/MSX)
- VDP: TMS9918 family in MSX1, Yamaha V9938 in MSX2, V9958 in MSX2+/turbo R. Sound: AY-3-8910 PSG or YM2149; optional MSX-Audio (Y8950), MSX-Music (YM2413); Konami SCC on cartridge adds five channels — [Wikipedia: MSX](https://en.wikipedia.org/wiki/MSX). European PAL machines use the TMS9929A, Japanese and US ones the TMS9918A/9928A (not confirmed) — [MSX Wiki](https://www.msx.org/wiki/)
- Memory: MSX1 8-64 KB; MSX2 64-512 KB, up to 4 MB per slot through a memory mapper; at least one cartridge slot, up to 16 — [Wikipedia: MSX](https://en.wikipedia.org/wiki/MSX). Slots and subslots are selected through the PPI at port A8h and register FFFFh; VRAM is reached only through VDP ports 98h/99h (not confirmed) — [MSX Wiki: Slots](https://www.msx.org/wiki/Slots)
- MegaROM cartridges use bank-switching mappers (Konami, Konami SCC, ASCII 8K, ASCII 16K); emulators need the right mapper type to run a ROM dump (not confirmed) — [MSX Wiki: MegaROM mappers](https://www.msx.org/wiki/MegaROM_Mappers)

**Atari 400/800, XL, XE**
- 400 and 800 November 1979; 1200XL March 1983 (dropped June 1983); 600XL/800XL late 1983; 65XE and 130XE 1985; XEGS 1987; about two million sold to mid-1985 — [Wikipedia: Atari 8-bit](https://en.wikipedia.org/wiki/Atari_8-bit_computers)
- 6502B or the custom "SALLY" 6502C; 1.79 MHz NTSC, 1.77 MHz PAL — [Wikipedia: Atari 8-bit](https://en.wikipedia.org/wiki/Atari_8-bit_computers)
- ANTIC runs a display list (one instruction per mode line) and reads screen data; CTIA, later GTIA, does colour and player/missile graphics; POKEY does sound, keyboard and serial I/O — [Wikipedia: Atari 8-bit](https://en.wikipedia.org/wiki/Atari_8-bit_computers)
- 130XE has 128 KB bank-switched; XL/XE have built-in BASIC; SIO serial bus; 810 disk, 1010 cassette — [Wikipedia: Atari 8-bit](https://en.wikipedia.org/wiki/Atari_8-bit_computers). Banking uses PIA PORTB at $D301 (16 KB window at $4000); display list interrupts (DLIs) are enabled through NMIEN; ANTIC DMA steals cycles per line (not confirmed) — [Altirra Hardware Reference Manual](https://www.virtualdub.org/altirra.html)

**Dragon 32/64 and TRS-80 Color Computer**
- CoCo announced July 1980 at US$399; 6809E plus MC6847 VDG; CoCo 1 4/16/32K, CoCo 2 (1983-1986) 16/64K, CoCo 3 (1986-1991) 128/512K with GIME, 64-colour palette and software-selected double speed; Color BASIC is Microsoft BASIC; OS-9 available — [Wikipedia: TRS-80 Color Computer](https://en.wikipedia.org/wiki/TRS-80_Color_Computer)
- Dragon 32 August 1982, Dragon 64 a year later (adds serial port); 6809E at 0.89 MHz; Microsoft BASIC; Dragon Data collapsed 1984; Eurohard took over and failed 1987; 798 commercial games cited — [Wikipedia: Dragon 32/64](https://en.wikipedia.org/wiki/Dragon_32/64) (the article carries citation-needed tags)
- Both use the MC6883 SAM for memory and timing; sound is a 6-bit DAC through a PIA; the SAM "speed-up poke" doubles the clock on some machines (not confirmed) — [Dragon Wiki](https://archive.worldofdragon.org/)

**Oric-1 and Atmos**
- Oric-1 1 September 1982; Atmos 1 February 1984; 6502A at 1 MHz; 16 or 48 KB; ULA HCS 10017; AY-3-8912 (one section says 8910); 40x28 text and 240x200 8-colour graphics; serial attributes take one character cell each; BASIC v1.0 (Oric-1) and v1.1 (Atmos), and some games depend on v1.0; Microdisc 3-inch drive around the end of 1984 — [Wikipedia: Oric (computer)](https://en.wikipedia.org/wiki/Oric_(computer))
- A 6522 VIA drives the AY, keyboard, printer port and tape (not confirmed) — [Defence Force Oric wiki](https://wiki.defence-force.org/)

**Apple II, IIe, IIgs**
- Apple II on sale 10 June 1977; 6502 at 1.023 MHz; 4 KB base, 48 KB practical maximum; lo-res 40x48 in 16 colours; hi-res 280x192 (140 colour) with 6 visible colours; 1-bit speaker toggled by software; FSK cassette; Disk II and Apple DOS late 1978, 140 KB — [Wikipedia: Apple II (original)](https://en.wikipedia.org/wiki/Apple_II_(original))
- Later models use the 65C02; IIgs 1986 with the 65C816; about six million Apple IIs sold; IIe the best seller — [Wikipedia: Apple II](https://en.wikipedia.org/wiki/Apple_II)
- Speaker toggles at $C030; soft switches at $C000-$C0FF; the language card banks RAM over $D000-$FFFF; IIe 128K adds auxiliary memory and double hi-res; IIgs runs at 2.8 MHz with the Ensoniq 5503 DOC (not confirmed) — [AppleWin documentation](https://github.com/AppleWin/AppleWin); [Wikipedia: Apple IIGS](https://en.wikipedia.org/wiki/Apple_IIGS)

**NEC PC-8801 and Sharp X1 (brief)**
- PC-8801 shipped November 1981; Z80-based; N88-BASIC; later mkII and mkII SR (the SR has the YM2203 FM chip), later models the YM2608; it was one of the four major Japanese home computers with the FM-7, X1 and MSX; Ys, Thexder, Snatcher and Dragon Slayer series first appeared on it — [Wikipedia: PC-8800 series](https://en.wikipedia.org/wiki/PC-8800_series)
- The PC-8801 uses a second Z80 in the disk unit; D88 is the common disk image format (not confirmed) — [Wikipedia: PC-8800 series](https://en.wikipedia.org/wiki/PC-8800_series)
- Sharp X1 launched 1982 with a Z80A at 4 MHz, an HD46505 CRTC, programmable character generator (PCG) and AY-3-8910; X1 turbo 1984 (not confirmed) — [Wikipedia: Sharp X1](https://en.wikipedia.org/wiki/Sharp_X1)

### Inferences
- A reimplementation that must match frame timing on the Spectrum needs the contention tables per model; 48K, 128K and +2A/+3 differ in contended pages, start T-state and pattern.
- On the C64, which ROMs and I/O are visible depends on $01 and on cartridge lines, so a disassembly must record the bank state for every code region.
- PAL and NTSC versions of C64, VIC-20 and Atari games are often different builds, because cycles per line and lines per frame differ.

### Gaps
- The exact 128K Spectrum clock (3.5469 MHz) and frame lengths lack a primary source.
- The Electron's 1 MHz RAM limit, the Master's 65SC12 and the BBC memory map are not confirmed.
- Sharp X1 and PC-8801 CPU clocks, and model years after the original PC-8801, are not confirmed.
- VIC-II cycle counts and badline behaviour are not confirmed. Christian Bauer's VIC-II article is the standard reference.

## How games were written, how they loaded, and loaders and protection

### Takeaway
Commercial games were mostly hand-written assembly, often cross-assembled on bigger machines; BASIC appears in loaders and in budget or type-in games. Tape was the main medium in the UK and Europe (Spectrum, C64, CPC, BBC, Electron), disk in the US (C64 1541, Apple II, Atari) and Japan (PC-8801), and cartridge on MSX, VIC-20 and Atari. Custom tape loaders (turbo loaders) both sped loading and acted as copy protection.

### Cited Findings
- TZX (revision 1.20, December 2006) exists to preserve turbo and custom loaders: it stores pilot, sync and bit timings, pulse sequences, direct recordings and generalized symbol encodings so the emulator need not infer them. Timings are in T-states at 3.5 MHz — [TZX format](https://worldofspectrum.net/TZXformat.html)
- Block 0x10 is standard ROM timing; 0x11 turbo; 0x12 pure tone; 0x13 pulse sequence (used by some protection schemes); 0x14 pure data; 0x15 direct recording; 0x18 CSW; 0x19 generalized data; 0x20 pause or stop; 0x2A stop only in 48K mode — [TZX format](https://worldofspectrum.net/TZXformat.html)
- Spectrum turbo and protected loaders include Speedlock (several versions), Alkatraz, Bleepload and others; many use a BASIC stub followed by machine code that decrypts the next stage (not confirmed) — [Sinclair Wiki: Tape protection schemes](https://sinclair.wiki.zxnet.co.uk/wiki/Tape_protection_schemes)
- The +3 uses 3-inch disks with copy protection based on non-standard sectors such as Speedlock +3 disk protection (not confirmed) — [CPCWiki: Disk protection](https://www.cpcwiki.eu/index.php/Copy_protection)
- The 1541 has its own CPU and ROM — [Wikipedia: Commodore 64](https://en.wikipedia.org/wiki/Commodore_64). C64 tape and disk fast loaders (Novaload, Freeload, Cyberload, Vorpal) and disk protections (bad sectors, half tracks, density changes) depend on this (not confirmed) — [Codebase64](https://codebase64.org/)
- Apple II Disk II stores 140 KB per 5.25-inch side — [Wikipedia: Apple II (original)](https://en.wikipedia.org/wiki/Apple_II_(original)). Its software-controlled GCR encoding (6-and-2 in DOS 3.3) let publishers write custom sector formats, nibble counts and spiral tracks as protection (not confirmed) — [WOZ format reference](https://applesaucefdc.com/woz/reference2/)
- The BBC B+ change from the 8271 to the 1770 disk controller broke some software, which is a known cause of disk-protection failures across BBC models — [Wikipedia: BBC Micro](https://en.wikipedia.org/wiki/BBC_Micro)
- Oric games can depend on BASIC ROM v1.0 behaviour — [Wikipedia: Oric (computer)](https://en.wikipedia.org/wiki/Oric_(computer))
- MSX games were often MegaROM cartridges with bank-switching mappers (not confirmed); Konami's SCC sound chip shipped inside some cartridges — [Wikipedia: MSX](https://en.wikipedia.org/wiki/MSX)
- Many VIC-20 titles were ROM cartridges — [Wikipedia: VIC-20](https://en.wikipedia.org/wiki/Commodore_VIC-20)
- Elite was written for the BBC Micro by Ian Bell and David Braben and ported to Electron, Master, C64, Apple II and NES; the versions share 6502 code — [Elite on the 6502](https://www.bbcelite.com/)

### Inferences
- The first job on a tape game is often to get past the loader: run it under an emulator to the point the game code is in memory, then snapshot. SkoolKit's tap2sna.py does this for Spectrum tapes by simulating the load.
- Protection code is part of the original's behaviour only until the game starts; a reimplementation normally begins from the decrypted in-memory image.

### Gaps
- Named protection schemes (Speedlock, Novaload, Alkatraz) lack primary sources. So do the cross-assemblers studios used (for example PDS on Spectrum, or 6502 cross-assemblers on PDP or Apple hosts).

## Image file formats and what each preserves

### Takeaway
Two kinds of image exist: snapshots (memory and CPU state at one moment) and media images (the tape or disk as written). Lossless media images (TZX, G64, WOZ, CSW, raw flux) preserve protection and loader timing; simple ones (TAP, D64, DSK sector dumps, ATR) preserve only the data the standard OS can read.

### Cited Findings
- TZX preserves custom loaders and timing; signature "ZXTape!"; Amstrad CPC uses the same structure as .CDT, and CPC timings scale by 4/3.5 — [TZX format](https://worldofspectrum.net/TZXformat.html)
- SkoolKit reads TAP, TZX and PZX tapes and Z80 and SZX snapshots, and plays RZX input recordings — [SkoolKit](https://skoolkit.ca/)
- Spectrum TAP holds only standard ROM-loader blocks (length, flag, data, checksum); SNA is a simple snapshot (48K form pushes PC on the stack); Z80 is a compressed snapshot with versions for 48K/128K; SZX (from Spectaculator) stores fuller machine state; RZX records inputs for replay (not confirmed) — [Sinclair Wiki: file formats](https://sinclair.wiki.zxnet.co.uk/wiki/Category:File_formats)
- C64: PRG is a file with a 2-byte load address; T64 is a tape container of files; TAP (C64) is raw pulse lengths; D64 is a 1541 sector image (683 sectors, 174,848 bytes, optional error bytes); G64 is a GCR track image that keeps protection; CRT is a cartridge image with a hardware type field (not confirmed) — [VICE manual: file formats](https://vice-emu.sourceforge.io/vice_17.html)
- BBC/Electron: UEF is a chunked tape (and state) format; SSD single-sided and DSD double-sided are DFS sector images (not confirmed) — [BeebWiki: UEF](https://beebwiki.mdfs.net/Uniform_Emulator_Format)
- CPC: DSK and Extended DSK (EDSK) store sectors with per-sector size and status, so EDSK can keep many protections (not confirmed) — [CPCWiki: DSK format](https://www.cpcwiki.eu/index.php/Format:DSK_disk_image_file_format)
- Atari: ATR is a sector disk image; XEX is a DOS binary executable; CAS is a tape image; ATX keeps timing and weak sectors for protected disks (not confirmed) — [Altirra](https://www.virtualdub.org/altirra.html)
- Apple II: DSK/DO/PO are 140 KB sector images in DOS or ProDOS order; NIB is a nibble image; WOZ (from Applesauce) keeps bit-level tracks and timing so protected disks run (not confirmed) — [WOZ reference](https://applesaucefdc.com/woz/reference2/)
- MSX: ROM dumps plus mapper type; DSK disk images; CAS tapes (not confirmed) — [openMSX manual](https://openmsx.org/manual/)
- PC-8801: D88 disk images (not confirmed) — [Wikipedia: PC-8800 series](https://en.wikipedia.org/wiki/PC-8800_series)

### Inferences
- For reverse engineering, a snapshot taken after the loader finishes is usually the most useful starting file, but keep the lossless tape or disk image as the ground truth of what shipped.

### Gaps
- Exact header layouts are covered only for TZX.

## Accurate emulators with debuggers, and community documentation

### Takeaway
Every machine on the list has at least one cycle-accurate or near-cycle-accurate open emulator with a debugger. MAME covers most of them as a fallback with a common debugger.

### Cited Findings
- SkoolKit's trace.py traces Spectrum code execution and can write AY output as WAV; tap2sna.py simulates tape loading — [SkoolKit](https://skoolkit.ca/)
- The following are the standard emulators per machine, each with a built-in or attached debugger (current feature sets not confirmed):
  - Spectrum and ZX81: Fuse — [fuse-emulator.sourceforge.net](https://fuse-emulator.sourceforge.net/); ZEsarUX — [github.com/chernandezba/zesarux](https://github.com/chernandezba/zesarux); SpecEmu — [specemu.zxe.io](http://www.specemu.zxe.io/); EightyOne for ZX81
  - BBC/Electron: b-em — [github.com/stardot/b-em](https://github.com/stardot/b-em); BeebJit — [github.com/scarybeasts/beebjit](https://github.com/scarybeasts/beebjit); B2 — [github.com/tom-seddon/b2](https://github.com/tom-seddon/b2); Elkulator for Electron
  - VIC-20 and C64: VICE (monitor with breakpoints, remote monitor protocol) — [vice-emu.sourceforge.io](https://vice-emu.sourceforge.io/); CCS64 (Windows)
  - CPC: WinAPE — [winape.net](http://www.winape.net/); Caprice32 — [github.com/ColinPitrat/caprice32](https://github.com/ColinPitrat/caprice32)
  - MSX: openMSX (Tcl console and the openMSX Debugger) — [openmsx.org](https://openmsx.org/)
  - Atari 8-bit: Altirra (debugger plus a hardware reference manual) — [virtualdub.org/altirra.html](https://www.virtualdub.org/altirra.html)
  - Dragon/CoCo: XRoar — [6809.org.uk/xroar](https://www.6809.org.uk/xroar/)
  - Oric: Oricutron — [github.com/pete-gordon/oricutron](https://github.com/pete-gordon/oricutron)
  - Apple II: AppleWin — [github.com/AppleWin/AppleWin](https://github.com/AppleWin/AppleWin)
  - All, including PC-8801 and X1: MAME (shared debugger, Lua scripting) — [docs.mamedev.org](https://docs.mamedev.org/debugger/index.html)
- Community documentation (content scope not confirmed): Spectrum Computing — [spectrumcomputing.co.uk](https://spectrumcomputing.co.uk/); World of Spectrum — [worldofspectrum.org](https://worldofspectrum.org/); Sinclair Wiki — [sinclair.wiki.zxnet.co.uk](https://sinclair.wiki.zxnet.co.uk/); Stardot forums — [stardot.org.uk](https://stardot.org.uk/forums/); BeebWiki — [beebwiki.mdfs.net](https://beebwiki.mdfs.net/); Codebase64 — [codebase64.org](https://codebase64.org/); C64-Wiki — [c64-wiki.com](https://www.c64-wiki.com/); CPCWiki — [cpcwiki.eu](https://www.cpcwiki.eu/); MSX Resource Center — [msx.org](https://www.msx.org/); AtariAge — [atariage.com](https://atariage.com/); World of Dragon — [worldofdragon.org](https://worldofdragon.org/); Defence Force (Oric) — [defence-force.org](https://www.defence-force.org/)

### Inferences
- Emulators with scriptable debuggers (VICE remote monitor, openMSX Tcl, MAME Lua, ZEsarUX remote protocol) suit scripted ground-truth capture better than GUI-only debuggers.

### Gaps
- Debugger features, scripting interfaces and accuracy claims of each emulator are not confirmed against current documentation.

## Notable disassemblies, reimplementations and source releases

### Takeaway
The best documented 8-bit game code is Elite (Mark Moxon's annotated 6502 source across several machines) and Richard Dymond's SkoolKit disassemblies of Spectrum games. Both show the target style: every routine labelled, every variable named, built so the source reassembles to the original bytes.

### Cited Findings
- Mark Moxon's "Elite on the 6502" gives fully documented source for the BBC Micro, Acorn Electron, BBC Master, Commodore 64, Apple II and NES versions, with about 130 deep dives — [bbcelite.com](https://www.bbcelite.com/)
- SkoolKit is a Spectrum game disassembly toolkit by Richard Dymond — [skoolkit.ca](https://skoolkit.ca/)
- SkoolKit-based disassemblies listed: Skool Daze, Back to Skool, Contact Sam Cruise, Manic Miner, Jet Set Willy, Hungry Horace, and the 48K Spectrum ROM — [skoolkit.ca/disassemblies](https://skoolkit.ca/disassemblies/)
- Other leads (not confirmed): Ultimate Play the Game Spectrum disassemblies hosted in community archives; C64 source releases collected on Codebase64 and GitHub (for example the Prince of Persia Apple II source released by Jordan Mechner in 2012 — [github.com/jmechner/Prince-of-Persia-Apple-II](https://github.com/jmechner/Prince-of-Persia-Apple-II)); Elite's original BBC source discs released by Ian Bell on his site — [iancgbell.clara.net/elite](http://www.iancgbell.clara.net/elite/)

### Inferences
- These projects show the useful end product for an 8-bit game: a source that reassembles byte-for-byte to the original, which gives an exact oracle for any port.

### Gaps
- Ultimate disassemblies, C64 source releases, and the Prince of Persia and Ian Bell releases lack a primary page. Their exact scope and licences need checking.
- Licensing of the Elite and SkoolKit disassemblies (whether they include game bytes) is not confirmed; relevant to a "no game material" rule.
