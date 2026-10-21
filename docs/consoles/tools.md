# Tools for console work

The harness's method works for consoles, but some tools change. Ghidra needs the right
processor module and a loader for the console's file format. Ground truth comes mostly from
emulators with debuggers. The harness's Unicorn emulator covers some console CPUs.

## Disassembly and decompilation

| CPU | Consoles | In Ghidra |
|---|---|---|
| 6502 and 65C02 | NES, PC Engine (HuC6280 is close) | Built in (6502, 65C02). Extensions add NES ROM loaders and mappers |
| 65C816 | SNES | Through community extensions; IDA and dedicated disassemblers are alternatives |
| Z80 and SM83 | Master System, Game Boy, sound CPUs | Z80 built in; the Game Boy's SM83 through extensions |
| 68000 | Mega Drive, Neo Geo, arcade boards | Built in (68000). Extensions add Mega Drive ROM loaders |
| MIPS (R3000, R4300, R5900) | PlayStation, Nintendo 64, PlayStation 2 | MIPS built in; extensions add PS-X EXE and N64 ROM loaders, and the R5900's extra instructions |
| SuperH (SH-2, SH-4) | Saturn, Dreamcast | Built in (SuperH with SH-1, SH-2 and SH-2A; SuperH4) |
| PowerPC (Gekko, Broadway) | GameCube, Wii | PowerPC built in; extensions add paired-single instructions and DOL and REL loaders |
| ARM7 (ARM and Thumb) | Game Boy Advance, Nintendo DS | Built in |
| x86 | Xbox | Built in; an extension adds the XBE loader |

Set the memory map before analysis: ROM, RAM and hardware registers at their addresses, and
name the hardware registers. Most console communities publish register lists; load them as
labels so the decompiled code reads `PPUCTRL` instead of `0x2000`.

For 8-bit and 16-bit games written in assembly, the decompiler's C output is of limited use.
Work from the disassembly and write your findings as tables and prose.

## Emulators with debuggers

| Console | Emulators with good debuggers |
|---|---|
| NES, SNES, Game Boy, PC Engine | Mesen (NES; Mesen 2 adds SNES, Game Boy, PC Engine), bsnes-plus (SNES), BGB and SameBoy (Game Boy) |
| Mega Drive | BlastEm, Exodus, Gens KMod |
| Game Boy Advance | mGBA, no$gba |
| PlayStation | PCSX-Redux, DuckStation, no$psx |
| Nintendo 64 | ares, Project64 with its debugger |
| Saturn | Mednafen, Kronos |
| Dreamcast | Flycast, Redream (limited debugging) |
| PlayStation 2 | PCSX2 with its debugger |
| GameCube and Wii | Dolphin |
| Xbox | xemu |
| Arcade | MAME |
| Several systems, scripted | BizHawk (Lua scripting, input movies, RAM watch, many cores) |

What to use them for:

- **RAM search and watch** to find variables: search for a value that changes when the
  player's lives change, then name the address.
- **Breakpoints** on reads and writes to find the code that changes a variable.
- **Trace logs** of every instruction for a frame, to compare with your port's behaviour.
- **Input movies** (BizHawk, FCEUX, Mesen) as recorded per-frame input. They are the console
  equivalent of demos, and tool-assisted speedrun movies cover whole games.
- **Scripting** (Lua in Mesen, BizHawk, FCEUX) to dump RAM every frame and to drive the game.
  This is the console equivalent of the harness's headless runs.

## The harness's Unicorn emulator

Unicorn 2 emulates ARM, ARM64, M68K, MIPS (32 and 64-bit), PowerPC, RISC-V, s390x, SPARC,
TriCore and x86. It has no 6502, 65816, Z80 or SuperH. For the CPUs it supports, you can run one original function
headless, as `pe_emu.py` does for Windows code:

1. Load the ROM or the executable's code segments at their addresses.
2. Map RAM, and fill it from a RAM dump taken in an emulator at the right moment.
3. Set the registers, call the function, read the results.
4. Stub hardware registers the function touches.

This gives bit-exact ground truth in tests, as for PC games. Unicorn does not emulate
coprocessors such as the PlayStation's GTE or the PS2's vector units; take ground truth for
those from a full emulator.

For CPUs Unicorn lacks, use a small emulator core in Python or Rust for that CPU (6502 and Z80
cores are short), or script a full emulator to run the function and dump the result.

## Splitting and building tools

Projects that decompile console games built tools that also help a reimplementation:

| Tool | What it does |
|---|---|
| splat | Splits a ROM or executable (N64, PlayStation, PS2) into code, data and assets by a configuration file, and disassembles the code |
| asm-differ | Compares the assembly of two builds function by function |
| decomp-toolkit (dtk) and objdiff | Split and compare GameCube and Wii programs |
| decomp.me | A website for comparing the output of the original compilers with candidate C code |
| m2c | Turns MIPS and PowerPC assembly into first-draft C |

See [Matching decompilation](matching-decomp.md) for how they fit together.
