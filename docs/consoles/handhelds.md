# Handhelds and 32-bit oddities

These platforms sit outside the main console lines. Some use a CPU that Ghidra has no module
for. Others put real game code on co-processors. The Nintendo DS, 3DS, PSP and Vita are on the
[Seventh generation onward](modern.md) page, as they ship packaged or encrypted executables.

## The platforms

| Platform | Year | CPU | Media | Ghidra module or loader | Emulator with a debugger |
|---|---|---|---|---|---|
| Sega Game Gear | 6 October 1990 (JP) | Z80 at 3.58 MHz | Cartridge | Z80 | MAME |
| Atari Lynx | September 1989 | 65SC02 at 4 MHz in Mikey | Cartridge with an encrypted loader | 65C02 | Handy, Mednafen (lineage not confirmed) |
| Neo Geo Pocket and Color | October 1998 and March 1999 (JP) | TLCS-900/H at 6.144 MHz; Z80 for sound | Cartridge | No TLCS-900 module; [NGDis](https://se.aminet.net/dev/asm/NGDis.readme) disassembles | MAME, Mednafen |
| WonderSwan, Color, SwanCrystal | March 1999 to 2002 (JP) | NEC V30MZ at 3.072 MHz | Cartridge | x86 real mode; [ghidra-wonderswan](https://github.com/JidouGhost-TL/ghidra-wonderswan) | Mednafen, Mesen 2 |
| Virtual Boy | July 1995 (JP) | NEC V810 at 20 MHz | Cartridge | Built-in V850 differs; [Ghidra_v810_v830](https://github.com/20Enderdude20/Ghidra_v810_v830) | Mednafen |
| Atari Jaguar | November 1993 | 68000 at 13.295 MHz; Tom and Jerry RISC at 26.59 MHz | Cartridge | 68000; no Tom or Jerry module | BigPEmu (closed), Virtual Jaguar |
| 3DO | October 1993 | ARM60 (ARMv3) at 12.5 MHz | CD; Portfolio OS | ARM v4 and up only | Opera (not confirmed) |

Mednafen covers the Virtual Boy, WonderSwan and Neo Geo Pocket with a debugger in its SDL front
end. Mesen 2 covers the WonderSwan (system list not confirmed)
([Mednafen docs](https://mednafen.github.io/documentation/); [Mesen](https://www.mesen.ca/)).

## Sega Game Gear

### Hardware

The Z80 runs at 3.58 MHz. The video chip is the Master System VDP, with 160x144 pixels
visible. Many Game Gear games are Master System ports
([Game Gear](https://en.wikipedia.org/wiki/Game_Gear)).

### Tools

Ghidra builds in the Z80. Unicorn and Capstone have no Z80, so ground truth comes from a small
core such as [iz80](https://github.com/ivanizag/iz80) or from MAME.

## Atari Lynx

### Hardware

The 65SC02 runs at 4 MHz inside the Mikey chip. The Suzy blitter scales sprites and has a maths
unit ([Atari Lynx](https://en.wikipedia.org/wiki/Atari_Lynx)).

### Tools

Load the code with Ghidra's 65C02 language. Ghidra's 6502 `ADC` p-code is not exact, so take
ground truth from a tested core ([Tools](tools.md#disassembly-and-decompilation)).

### For an exact reimplementation

The cartridge loader is encrypted. An emulator needs the Lynx boot ROM to run it.

## Neo Geo Pocket and Neo Geo Pocket Color

### Hardware

The main CPU is a TLCS-900/H at 6.144 MHz, with a Z80 for sound. The screen is 160x152
([Neo Geo Pocket Color](https://en.wikipedia.org/wiki/Neo_Geo_Pocket_Color)).

### Tools

Ghidra has no TLCS-900 module. [NGDis](https://se.aminet.net/dev/asm/NGDis.readme)
disassembles the code. MAME and Mednafen run the games.

## WonderSwan

### Hardware

The NEC V30MZ runs at 3.072 MHz and is compatible with 16-bit x86. The screen is 224x144
([WonderSwan](https://en.wikipedia.org/wiki/WonderSwan)).

### Tools

Load the code with Ghidra's x86 real-mode language.
[ghidra-wonderswan](https://github.com/JidouGhost-TL/ghidra-wonderswan) adds a loader. Unicorn
runs x86 16-bit code, so GameFrame's emulator approach applies to the CPU. The extension has
few users, so test it on a known binary before trusting it.

## Virtual Boy

### Hardware

The NEC V810 runs at 20 MHz. It drives red LED displays with 384x224 pixels for each eye
([Virtual Boy](https://en.wikipedia.org/wiki/Virtual_Boy)).

### Tools

Ghidra's built-in V850 differs from the V810.
[Ghidra_v810_v830](https://github.com/20Enderdude20/Ghidra_v810_v830) adds the V810. The
extension has few users, so test it on a known binary before trusting it. Mednafen has a
debugger.

## Atari Jaguar

### Hardware

The 68000 runs at 13.295 MHz. Tom and Jerry are 32-bit RISC cores at 26.591 MHz, with 4 KB and
8 KB of local RAM. The console has 2 MB of RAM. A memory controller flaw makes it awkward to run
RISC code from main RAM, which pushed developers back to the 68000
([Wikipedia: Atari Jaguar](https://en.wikipedia.org/wiki/Atari_Jaguar)). The main loop runs on
the 68000 and uploads RISC code ([Game Tech Wiki: Jaguar](https://emulation.gametechwiki.com/index.php/Jaguar)).
Tom has two banks of thirty-two 32-bit registers (not confirmed)
([Jagdox general](https://www.mulle-kybernetik.com/jagdox/general.html)).

### Tools

Ghidra builds in the 68000. No Ghidra module is known for Tom and Jerry
([GitHub search](https://github.com/search?q=ghidra+jaguar&type=repositories)). MAME's jaguar
core file carries a BSD-3-Clause header.

BigPEmu is Rich Whitehouse's closed-source Jaguar emulator, written for Digital Eclipse's
Atari 50. Whether it has a debugger is unknown. Virtual Jaguar is the open alternative, now
maintained by Joseph Mattiello, with GPU, DSP and blitter timing models
([Game Tech Wiki: Jaguar](https://emulation.gametechwiki.com/index.php/Jaguar)).

### For an exact reimplementation

Game code is split across the 68000 and the RISC cores. Find and export each CPU's code
separately. The RISC code is often taken from a RAM dump after the 68000 uploads it.

## 3DO

### Hardware

The ARM60 is an ARMv3 core at 12.5 MHz. The Madam and Clio chips hold the cel engine. Games run
on the Portfolio OS
([3DO Interactive Multiplayer](https://en.wikipedia.org/wiki/3DO_Interactive_Multiplayer)).

### Tools

Ghidra's ARM languages start at v4, so the ARM60 has no exact built-in match. Opera runs the
games (not confirmed).

### Projects to learn from

rawgl by cyxx reimplements the Another World engine and reads the 3DO data among others. It was
archived on 28 June 2025 ([cyxx/rawgl](https://github.com/cyxx/rawgl)).

Sources: [Report: game platforms and CPUs](../computers/research/report.md).
