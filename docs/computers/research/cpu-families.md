# Notes: CPU families and tools

Covers the CPU families of game platforms, their arithmetic traps, and their support in Ghidra, Unicorn, Capstone and other tools. The tool facts describe Ghidra master (release tag `Ghidra_12.1.4_build`), Unicorn master (release `2.1.4`) and Capstone's default branch `next` (release `6.0.0-Alpha11`).

Short link names used below:
- GP = https://github.com/NationalSecurityAgency/ghidra/tree/master/Ghidra/Processors
- UC = https://github.com/unicorn-engine/unicorn/tree/master/include/unicorn
- CS = https://github.com/capstone-engine/capstone/blob/next/include/capstone/capstone.h

## What each CPU family looks like to someone reading game code (years, word size, endianness, registers, notable features)

### Takeaway
The families split into 8-bit little-endian accumulators (8080/Z80/SM83, 6502 line), 8/16-bit big-endian Motorola and TI designs (6809, 68000, TMS9900, CP1600), and 32/64-bit RISC (ARM, MIPS, SuperH, PowerPC, Cell SPU, Jaguar RISC) plus x86. The features that most often mislead a reader are zero page and decimal mode on the 6502, banked or alternate registers (Z80, 65C816 M/X widths), memory-resident registers (TMS9900), branch and load delay slots (MIPS, SuperH), and per-instruction conditions and Thumb mode on ARM.

### Cited Findings

**Intel 8080 / Zilog Z80 / Sharp SM83 (LR35902)**
- The Z80 launched in July 1976 and was made binary compatible with the Intel 8080 — [Wikipedia: Zilog Z80](https://en.wikipedia.org/wiki/Zilog_Z80)
- Z80 adds an alternate register set (AF', BC', DE', HL'), index registers IX and IY, interrupt vector register I and refresh register R; undocumented instructions include use of the 8-bit halves of IX and IY — [Wikipedia: Zilog Z80](https://en.wikipedia.org/wiki/Zilog_Z80)
- Ghidra's Z80 spec defines the undocumented IXL/IXH/IYL/IYH registers — [z80.slaspec line 23](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/Z80/data/languages/z80.slaspec)
- Z80 machines in games include ColecoVision, Sega Master System, Game Gear and ZX Spectrum — [Wikipedia: Zilog Z80](https://en.wikipedia.org/wiki/Zilog_Z80)
- The Game Boy CPU (Sharp SM83) has registers AF, BC, DE, HL, SP, PC; flags Z (bit 7), N (bit 6), H (bit 5), C (bit 4); N and H are used only by DAA — [Pan Docs: CPU Registers and Flags](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html)
- The Game Boy Advance package contains both an ARM7TDMI and a Sharp SM83 for Game Boy and Game Boy Color titles — [Copetti: Game Boy Advance](https://www.copetti.org/writings/consoles/game-boy-advance/)

**MOS 6502 and variants**
- The 6502 was introduced in September 1975; registers are A, X, Y (8-bit), S (8-bit stack pointer), P (status), PC (16-bit) — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- Zero page ($00–$FF) is reached with one address byte (shorter, faster); the stack is fixed to page $01 ($0100–$01FF); the chip is little-endian — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- SED/CLD switch BCD arithmetic; $99 + $01 gives $00 with carry set in decimal mode and $9A in binary mode — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- 105 of 256 opcodes are undefined on the NMOS part; the `JMP ($xxFF)` indirect bug is an NMOS erratum fixed in the 65C02 — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- 6507 (Atari 2600) is a 28-pin part with 8 KB addressing and no interrupts; 6510 (Commodore 64) adds an I/O port at $0000/$0001; 65C02 is the CMOS version designed at WDC, development started 1981 — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- The NES 2A03 is a 6502 core "lacking decimal mode" with an APU, joypad I/O and DMA (OAM DMA at $4014); the PAL RP2A07 uses a different clock divider and has DMA fixes — [NESdev Wiki: 2A03](https://www.nesdev.org/wiki/2A03)
- HuC6280 (PC Engine) is a 65C02-style core with extra instructions, interrupt controller, MMU, timer, I/O port and PSG; it maps a 64 KB logical space onto 2 MB physical through the MMU; a revised HuC6280A was used in CoreGrafx and SuperGrafx — [Wikipedia: Hudson Soft HuC6280](https://en.wikipedia.org/wiki/Hudson_Soft_HuC6280)
- The 65C816 was finished in March 1984 and released in 1985; it resets into emulation mode (65C02-like); in native mode the m flag sets accumulator/memory width and x sets index width (0 = 16-bit, 1 = 8-bit); addresses are 24-bit, with data bank (DB, 8-bit) and relocatable direct page (DP, 16-bit, bank $00) registers; the SNES Ricoh 5A22 is based on it — [Wikipedia: WDC 65C816](https://en.wikipedia.org/wiki/WDC_65C816)

**Motorola 6809**
- Introduced 1978; accumulators A and B combine into 16-bit D; index registers X and Y; stack pointers U and S; 8-bit direct page register DP; condition codes CC — [Wikipedia: Motorola 6809](https://en.wikipedia.org/wiki/Motorola_6809)
- It has an 8x8 hardware multiply (A x B into D) and PC-relative data addressing for position-independent code — [Wikipedia: Motorola 6809](https://en.wikipedia.org/wiki/Motorola_6809)
- Game uses include the Vectrex (68A09), Williams arcade games (Defender, Robotron: 2084, Joust), Dragon 32/64 and TRS-80 Color Computer; the Hitachi 6309 adds registers, block move, more multiply and division — [Wikipedia: Motorola 6809](https://en.wikipedia.org/wiki/Motorola_6809)
- Ghidra describes the 6809 as big-endian (`6809:BE:16:default`) — [6800.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/MC6800/data/languages/6800.ldefs)

**Motorola 68000 family**
- Introduced September 1979; eight 32-bit data registers D0–D7 and eight 32-bit address registers A0–A7 (A7 is the stack pointer with separate USP and SSP); 16-bit ALU and data bus; 24-bit address bus; big-endian; condition codes C, V, Z, N plus X (extend) — [Wikipedia: Motorola 68000](https://en.wikipedia.org/wiki/Motorola_68000)
- Instruction set includes MULU/MULS, DIVU/DIVS and BCD instructions ABCD, SBCD, NBCD; there is an address error exception — [Wikipedia: Motorola 68000](https://en.wikipedia.org/wiki/Motorola_68000)
- 68008 and 68010 came in 1982; 68020 was the first with full 32-bit ALUs and buses; ColdFire is a later descendant — [Wikipedia: Motorola 68000](https://en.wikipedia.org/wiki/Motorola_68000)
- Game uses include Sega Mega Drive/Genesis (7.6 MHz main CPU), Saturn (68EC000 sound CPU), Neo Geo, Atari Jaguar, Amiga, Atari ST, X68000, and arcade boards such as Sega System 16 and Capcom CPS/CPS-2 — [Wikipedia: Motorola 68000](https://en.wikipedia.org/wiki/Motorola_68000)

**TI TMS9900**
- Introduced June 1976; 16-bit; big-endian — [Wikipedia: TMS9900](https://en.wikipedia.org/wiki/Texas_Instruments_TMS9900)
- Only three registers are on chip (PC, WP, ST); the 16 general registers live in RAM at the workspace pointer; BLWP saves old WP, PC, ST into workspace registers 13, 14, 15 and RTWP restores them — [Wikipedia: TMS9900](https://en.wikipedia.org/wiki/Texas_Instruments_TMS9900)
- It has multiply and divide on register pairs; it powered the TI-99/4 and TI-99/4A — [Wikipedia: TMS9900](https://en.wikipedia.org/wiki/Texas_Instruments_TMS9900)

**General Instrument CP1600 / CP1610**
- Introduced February 1975; 16-bit word is the smallest addressable unit (no byte addressing); opcodes use only 10 of 16 bits — [Wikipedia: CP1600](https://en.wikipedia.org/wiki/General_Instrument_CP1600)
- R0 accumulator, R1–R3 data pointers, R4–R5 auto-increment, R6 stack pointer, R7 program counter; SDBD builds a 16-bit operand from two 10-bit ROM reads; BEXT branches on an external signal — [Wikipedia: CP1600](https://en.wikipedia.org/wiki/General_Instrument_CP1600)
- The CP1610 (about 2 MHz) ran the Intellivision, produced 1980–1984 — [Wikipedia: CP1600](https://en.wikipedia.org/wiki/General_Instrument_CP1600)
- Ghidra describes it as `CP1600:BE:16:default` — [CP1600.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/CP1600/data/languages/CP1600.ldefs)

**Intel x86**
- Ghidra offers 16-bit Real Mode, 16-bit Protected Mode, 32-bit, 32-bit System Management Mode, 64-bit and 64-bit compat32 languages, all little-endian, with compiler specs for windows, gcc, borlandcpp, borlanddelphi, clangwindows and golang — [x86.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/x86/data/languages/x86.ldefs)

**ARM**
- GBA ARM7TDMI runs at 16.78 MHz; ARM instructions are 32-bit, Thumb instructions are 16-bit with eight general registers, a two-address format and no conditional execution; nearly every ARM instruction carries a 4-bit condition; ARM and Thumb code mix in one program ("interworking") — [Copetti: Game Boy Advance](https://www.copetti.org/writings/consoles/game-boy-advance/)
- GBA launched 21 March 2001 (Japan) — [Copetti: Game Boy Advance](https://www.copetti.org/writings/consoles/game-boy-advance/)

**MIPS**
- PlayStation (3 December 1994, Japan) uses a CW33300-based core compatible with the R3000A at 33.87 MHz; MIPS I; 32 GPRs (R0 hardwired to zero) plus HI/LO multiply/divide registers; no FPU (no CP1); GTE coprocessor (CP2) works only in fixed point; 1 KB scratchpad and 4 KB I-cache, no D-cache — [Copetti: PlayStation](https://www.copetti.org/writings/consoles/playstation/)
- On the R3000A a load does not stall; an instruction that uses the loaded value in the next slot needs a filler (load delay slot); the instruction after a branch always runs (branch delay slot) — [Copetti: PlayStation](https://www.copetti.org/writings/consoles/playstation/)
- Nintendo 64 (June 1996 Japan) uses the NEC VR4300 (R4300i-compatible) at 93.75 MHz; MIPS III with 32-bit and 64-bit modes; games rarely used 64-bit instructions; the console always runs in kernel mode; the RSP is a cut-down MIPS (no multiply, divide or 64-bit) with a 32 x 128-bit vector unit of 8 x 16-bit lanes — [Copetti: Nintendo 64](https://www.copetti.org/writings/consoles/nintendo-64/)
- PS2 Emotion Engine core R5900: two-way superscalar, MIPS III plus a MIPS IV subset plus Sony's 128-bit MMI SIMD; 32 x 128-bit GPRs; 16 KB I-cache, 8 KB D-cache, 16 KB scratchpad; 294.912 MHz (299 MHz later); vector units VU0 (4 KB data) and VU1 (16 KB data) with 32 x 128-bit registers and 16 x 16-bit integer registers — [Wikipedia: Emotion Engine](https://en.wikipedia.org/wiki/Emotion_Engine)

**SuperH**
- Hitachi introduced SuperH in 1992; SH-2 uses fixed 16-bit instructions in a 32-bit architecture; 16 general registers; SH-1 and SH-2 have delayed branches with one delay slot; SH-2 adds 64-bit multiply (DMULS/DMULU) and a MAC unit; SH-4 (with STMicroelectronics, from 1997) is two-way superscalar with single- and double-precision vector FPU — [Wikipedia: SuperH](https://en.wikipedia.org/wiki/SuperH)
- SH-1/SH-2 appear in the Sega Saturn, 32X and Capcom CPS-3; SH-4 in the Dreamcast and NAOMI — [Wikipedia: SuperH](https://en.wikipedia.org/wiki/SuperH)
- Ghidra's SH-1, SH-2, SH-2A languages are big-endian only; SH-4 has BE and LE languages — [superh.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/SuperH/data/languages/superh.ldefs); [SuperH4.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/SuperH4/data/languages/SuperH4.ldefs)

**PowerPC (Gekko, Broadway, Espresso, Xenon, Cell PPE)**
- Gekko (GameCube) launched 2000, a customised PowerPC 750CXe up to 486 MHz; the 64-bit FPU can act as 2 x 32-bit SIMD ("paired singles"), about 50 new instructions; Broadway (Wii) is its direct successor and Espresso powers the Wii U — [Wikipedia: Gekko](https://en.wikipedia.org/wiki/Gekko_(processor))
- Xenon (Xbox 360) has three in-order big-endian PowerPC cores at 3.2 GHz, two threads each, VMX128 with a 128 x 128-bit register file per thread; cores are modified Cell PPEs — [Wikipedia: Xenon](https://en.wikipedia.org/wiki/Xenon_(processor))
- The Cell PPE is dual-issue, in-order, two-way SMT, with 64-bit GPRs and FPRs and 128-bit AltiVec/VMX registers; Cell was introduced November 2006 with the PlayStation 3 — [Wikipedia: Cell](https://en.wikipedia.org/wiki/Cell_(processor))

**Cell SPU**
- Each SPU has a custom 128-bit SIMD ISA, 128 x 128-bit registers, 256 KiB local store for code and data, no direct access to main memory (DMA via the Memory Flow Controller, up to 16 KB per transfer or lists of 2–2048), no branch prediction — [Wikipedia: Cell](https://en.wikipedia.org/wiki/Cell_(processor))

**Atari Jaguar Tom and Jerry**
- Jaguar launched 23 November 1993 (North America); Tom (GPU) and Jerry (DSP) are 32-bit RISC cores at 26.591 MHz with 4 KB and 8 KB internal RAM; the 68000 at 13.295 MHz is the "manager"; a memory controller flaw forces special conventions to run RISC code from main RAM, and Tom/Jerry synchronisation bugs pushed developers back to the 68000 — [Wikipedia: Atari Jaguar](https://en.wikipedia.org/wiki/Atari_Jaguar)
- The GPU has two banks of thirty-two 32-bit registers — [Jagdox general](https://www.mulle-kybernetik.com/jagdox/general.html) (secondary hardware notes; not confirmed against Atari documentation)

### Inferences
- Every 8-bit family here is little-endian except the 6809, and every 16-bit console CPU except x86 is big-endian; data-file decoders must follow the CPU's byte order.
- On the TMS9900, register reads in disassembly are memory reads; tracing a function's "registers" means tracing the workspace RAM that WP points at.
- On the 65C816, the same opcode bytes decode differently by the M and X flags, so a disassembler must track REP/SEP state; a wrong guess produces garbage after the first immediate operand.

### Gaps
- Exact first-use years for 68030/68040/68060, SH-2 and SH-4 parts, R3000A, R4300i, Broadway (2006), Espresso (2012) and ARM cores (ARM2 1986, ARM7TDMI, ARM946E-S/ARM7 in the DS 2004, ARM11 MPCore in the 3DS 2011, Cortex-A57 in the Switch 2017) are not confirmed.
- Allegrex (PSP, MIPS32-based with VFPU) is covered only by the Ghidra extension below.
- Tom/Jerry register details, delay-slot behaviour and the "MOVEI" 48-bit encoding are not confirmed from a primary source.
- 6502 NMOS decimal-mode flag behaviour (N, V, Z undefined/odd in decimal mode) is not confirmed from a primary source.

## Arithmetic exactness for bit-exact reimplementation

### Takeaway
The risky cases are BCD (6502 decimal mode, Z80/SM83 DAA, 68000 ABCD), the 6502's carry-as-borrow, missing multiply or divide (6502, Z80, SM83, ARM7TDMI), fixed-point-only math (PS1 GTE), and non-IEEE floats (PS2 EE FPU and VUs, Cell SPU single precision). Ghidra's own p-code for 6502 ADC ignores decimal mode and computes V wrongly, so p-code emulation is not ground truth for the 6502.

### Cited Findings
- 6502 decimal mode changes ADC results ($99 + $01 → $00, carry set) — [Wikipedia: MOS Technology 6502](https://en.wikipedia.org/wiki/MOS_Technology_6502)
- The NES 2A03 lacks decimal mode, so the same game code gives different results on a stock 6502 — [NESdev Wiki: 2A03](https://www.nesdev.org/wiki/2A03)
- Ghidra 6502 `ADC` semantics are `C = carry(A, op1); A = A + op1 + tmpC; resultFlags(A); V = C;`: no decimal path, the incoming carry is not part of the carry-out test, and V is copied from C instead of signed overflow — [6502.slaspec lines 127–138](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/6502/data/languages/6502.slaspec#L127-L138)
- Ghidra's Z80 spec implements DAA in full, using N to tell add from subtract — [z80.slaspec line 1234](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/Z80/data/languages/z80.slaspec#L1234)
- The Z80 DAA checks N; the 8080 DAA is defined only after addition; the Z80 has no multiply instruction — [Wikipedia: Zilog Z80](https://en.wikipedia.org/wiki/Zilog_Z80)
- SM83 carry on subtraction matches Z80 and x86 (set on borrow), not 65xx or ARM — [Pan Docs: CPU Registers and Flags](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html)
- 6809 has an 8x8→16 hardware multiply; the 6309 adds division — [Wikipedia: Motorola 6809](https://en.wikipedia.org/wiki/Motorola_6809)
- 68000 has MULU/MULS/DIVU/DIVS and BCD ABCD/SBCD/NBCD with a separate X flag — [Wikipedia: Motorola 68000](https://en.wikipedia.org/wiki/Motorola_68000)
- TMS9900 has multiply and divide on register pairs — [Wikipedia: TMS9900](https://en.wikipedia.org/wiki/Texas_Instruments_TMS9900)
- PS1 has no FPU; the GTE works only in fixed point — [Copetti: PlayStation](https://www.copetti.org/writings/consoles/playstation/)
- N64 VR4300 FPU handles single and double precision and follows IEEE 754 — [Copetti: Nintendo 64](https://www.copetti.org/writings/consoles/nintendo-64/)
- The EE FPU "isn't IEEE 754 compliant" — [Wikipedia: Emotion Engine](https://en.wikipedia.org/wiki/Emotion_Engine)
- PCSX2 developers report that the same arithmetic on the PS2 FPU/VU and on x86 can differ by a few bits, and that full accuracy would need a slow software FPU — [PCSX2: Nightmare on Floating-Point Street](https://pcsx2.net/blog/2006/nightmare-on-floating-point-street); [PCSX2: What's clamping](https://pcsx2.net/blog/2009/whats-clamping-why-do-we-need-it)
- PCSX2 docs say PS2 add/subtract behave much like SSE round-toward-zero without INF/NaN — [PCSX2 wiki VU documentation](https://wiki.pcsx2.net/Special:MobileDiff/38317)
- Cell SPU single precision is documented as round-toward-zero, denormals treated as zero, no INF/NaN — [IBM Cell SDK 3.0: equivalent precision considerations](https://arcb.csc.ncsu.edu/~mueller/cluster/ps3/SDK3.0/docs/accessibility/sdkpt/cbet_3codemapeqv.html) (search snippet)
- Original Cell SPEs give about 1.8 GFLOPS double precision each, far below single precision — [Wikipedia: Cell](https://en.wikipedia.org/wiki/Cell_(processor))
- ARMv4 (ARM7TDMI) has 32-bit and 64-bit-result multiplies but no hardware divide; the GBA BIOS supplies division, square root and arctangent — [Copetti: Game Boy Advance](https://www.copetti.org/writings/consoles/game-boy-advance/)
- SH-1 traps on the SH-2's 64-bit multiply instructions — [Wikipedia: SuperH](https://en.wikipedia.org/wiki/SuperH)

### Inferences
- For 6502 targets, bit-exact ground truth must come from a cycle-tested core (py65, mos6502, MAME, floooh/chips), not from Ghidra's p-code emulator.
- Games on 6502/Z80/SM83/ARM7 call software multiply and divide routines; those routines (and the GBA BIOS SWI routines) are the functions to emulate for exact results.
- For PS2 and Cell SPU float code, a port that uses host IEEE floats will diverge; it needs a software float model that rounds toward zero and clamps.
- Gekko paired-single results depend on the quantised load/store (GQR) setup; this needs confirming against Dolphin's implementation.

### Gaps
- Exact EE FPU rules (maximum value, denormal flush, rounding of each op) are not confirmed from a primary source.
- Allegrex VFPU, Jaguar RISC divide unit, and SH-4 FPSCR precision modes are not covered.
- Gekko GQR quantisation details are not confirmed (the Wikipedia page does not cover them).

## Ghidra support: built-in processors and community extensions

### Takeaway
Ghidra master ships 39 processor folders. Built in for game CPUs: 6502/65C02, 6805, 6809/H6309 (in `MC6800`), 68000 to 68040, ColdFire and CPU32, 8048, 8051, 8085, Z80/Z180, CP1600, x86 (16/32/64), ARM v4–v8 with Thumb, AARCH64, MIPS32/64, PowerPC 32/64 with AltiVec, SuperH (SH-1, SH-2, SH-2A), SuperH4, V850, tricore. Not built in: 65816, SM83, TMS9900, HuC6280, R5900, Allegrex, Gekko paired singles, Cell SPU, Jaguar RISC; community modules exist for all except Jaguar RISC.

### Cited Findings
- `Ghidra/Processors` contains: 6502, 68000, 8048, 8051, 8085, AARCH64, ARM, Atmel, BPF, CP1600, CR16, DATA, Dalvik, HCS08, HCS12, Hexagon, JVM, Loongarch, M16C, M8C, MC6800, MCS96, MIPS, NDS32, PA-RISC, PIC, PowerPC, RISCV, Sparc, SuperH, SuperH4, TI_MSP430, Toy, V850, Xtensa, Z80, eBPF, tricore, x86 — [GP](https://github.com/NationalSecurityAgency/ghidra/tree/master/Ghidra/Processors)
- 6502 folder: `6502:LE:16:default` and `65C02:LE:16:default`; no 65816 — [6502.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/6502/data/languages/6502.ldefs)
- MC6800 folder: `6800.ldefs` holds `6809:BE:16:default` and `H6309:BE:16:default` (described as "missing many instructions"); `6805.ldefs` holds `6805:BE:16:default`; there is no plain 6800 language — [6800.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/MC6800/data/languages/6800.ldefs); [6805.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/MC6800/data/languages/6805.ldefs)
- 68000 folder: `68000:BE:32:default` (described as 68040), `MC68030`, `MC68020`, `Coldfire`, `CPU32` (MC68340) — [68000.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/68000/data/languages/68000.ldefs)
- Z80 folder: `z80`, `z8401x`, `z180`, `z182`, all LE 16 — [z80.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/Z80/data/languages/z80.ldefs)
- 8085 folder: `8085:LE:16:default`; no separate 8080 language — [8085.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/8085/data/languages/8085.ldefs)
- CP1600 folder: `CP1600:BE:16:default` — [CP1600.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/CP1600/data/languages/CP1600.ldefs)
- ARM folder: generic v4, v4t, v5, v5t, v6, v7, v8, v8T, Cortex and v8-m, each LE and BE, plus LE-instruction/BE-data variants for v7 and v8 — [ARM.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/ARM/data/languages/ARM.ldefs)
- AARCH64: v8.5-A LE and BE data, plus ilp32; described as "missing some 8.5 vector" — [AARCH64.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/AARCH64/data/languages/AARCH64.ldefs)
- MIPS: MIPS32 and MIPS64, BE and LE, with mips16e, microMIPS and Release 6 variants and o32/n32/o64 ABIs; no R3000-, R5900- or Allegrex-specific language — [mips.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/MIPS/data/languages/mips.ldefs)
- MIPS branch delay slots are modelled with `delayslot` (47 uses in `mips32Instructions.sinc`); `mips.sinc` has no load-delay modelling — [mips32Instructions.sinc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/MIPS/data/languages/mips32Instructions.sinc)
- PowerPC: 32/64-bit BE/LE "w/Altivec, G2", 4xx, MPC8270, QUICC, e500, e500mc, Power ISA 3.0 A2 and VLE variants; no Gekko/Broadway or VMX128 language; `ppc_instructions.sinc` has no `psq_` or `ps_madd` instructions — [ppc.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/PowerPC/data/languages/ppc.ldefs); [ppc_instructions.sinc](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/PowerPC/data/languages/ppc_instructions.sinc)
- SuperH: `SH-2A`, `SH-2`, `SH-1` (BE 32); SuperH spec uses `delayslot` (10 uses in `superh.sinc`); SuperH4: BE and LE — [superh.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/SuperH/data/languages/superh.ldefs); [SuperH4.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/SuperH4/data/languages/SuperH4.ldefs)
- x86: 16-bit Real and Protected Mode, 32, 64, SMM, compat32 — [x86.ldefs](https://github.com/NationalSecurityAgency/ghidra/blob/master/Ghidra/Processors/x86/data/languages/x86.ldefs)

Community extensions (stars, licence and last push, as of October 2026):
- 65816: [achan1989/ghidra-65816](https://github.com/achan1989/ghidra-65816) — 30 stars, MIT, archived, last push 2024-03-27; a newer [kaayzouee/ghidra-65816](https://github.com/kaayzouee/ghidra-65816) — 1 star, MIT
- SM83 / Game Boy: [Gekkio/GhidraBoy](https://github.com/Gekkio/GhidraBoy) — 286 stars, Apache-2.0, archived, last push 2025-08-30; fork [minorum/ghidra-sm83](https://github.com/minorum/ghidra-sm83) — Apache-2.0, active (2026-09-15)
- TMS9900: [gnulnulf/Ghidra-TMS9900](https://github.com/gnulnulf/Ghidra-TMS9900) — 4 stars, licence not detected by GitHub, last push 2026-04-21
- HuC6280: [TiCoKH/Ghidra_HuC6280](https://github.com/TiCoKH/Ghidra_HuC6280) — 3 stars, no licence detected, last push 2025-03-11
- R5900 / PS2: [chaoticgd/ghidra-emotionengine-reloaded](https://github.com/chaoticgd/ghidra-emotionengine-reloaded) — 259 stars, Apache-2.0, active (2026-10-04); original [beardypig/ghidra-emotionengine](https://github.com/beardypig/ghidra-emotionengine) — Apache-2.0, archived
- Allegrex / PSP: [kotcrab/ghidra-allegrex](https://github.com/kotcrab/ghidra-allegrex) — 134 stars, Apache-2.0, active (2026-08-25)
- Gekko/Broadway paired singles: [aldelaro5/ghidra-gekko-broadway-lang](https://github.com/aldelaro5/ghidra-gekko-broadway-lang) — 135 stars, licence NOASSERTION, last push 2022-02-09
- Cell SPU: [aerosoul94/GhidraSPU](https://github.com/aerosoul94/GhidraSPU) — 30 stars, Apache-2.0, last push 2023-07-07
- Sega Saturn SH-2: [VGKintsugi/Ghidra-SegaSaturn-Processor](https://github.com/VGKintsugi/Ghidra-SegaSaturn-Processor) — MIT, archived, last push 2019-11-02
- Platform loaders (not processors): PSX [lab313ru/ghidra_psx_ldr](https://github.com/lab313ru/ghidra_psx_ldr); N64 [zeroKilo/N64LoaderWV](https://github.com/zeroKilo/N64LoaderWV); Xbox 360 XEX [zeroKilo/XEXLoaderWV](https://github.com/zeroKilo/XEXLoaderWV); GameCube [Cuyler36/Ghidra-GameCube-Loader](https://github.com/Cuyler36/Ghidra-GameCube-Loader) (Apache-2.0); Switch [Adubbz/Ghidra-Switch-Loader](https://github.com/Adubbz/Ghidra-Switch-Loader) (ISC); DS [pedro-javierf/NTRGhidra](https://github.com/pedro-javierf/NTRGhidra) (Apache-2.0); GBA [pudii/gba-ghidra-loader](https://github.com/pudii/gba-ghidra-loader) (Apache-2.0); NES [kylewlacy/GhidraNes](https://github.com/kylewlacy/GhidraNes)
- Atari Jaguar Tom/Jerry RISC: no Ghidra module is known. The only repository matching "ghidra jaguar" is chirpy8/Jaguar_AJ27_B68, a loader for a Jaguar car ECU — [GitHub search](https://github.com/search?q=ghidra+jaguar&type=repositories)

### Inferences
- Ghidra's built-in 68000 default language is a 68040 superset; 68000 code loads fine, but the decompiler accepts instructions a real 68000 would fault on.
- The 8080 is covered by the 8085 language (8085 is a superset); this is untested.
- GhidraBoy is archived, so a project should pin a version that matches its Ghidra release or use the active fork.
- Because the PowerPC module lacks paired singles, GameCube and Wii binaries decompile with gaps unless the Gekko extension is installed.

### Gaps
- Which Ghidra versions each extension builds against is not confirmed.
- Xbox 360 VMX128 support in any Ghidra extension is not confirmed.
- A 6809 or 6502 variant for HuC6280 bank mapping in loaders is not covered.

## Unicorn 2: what it emulates, gaps, and small cores for the gaps

### Takeaway
Unicorn 2 has ten architectures: ARM (with Thumb), ARM64, MIPS, x86, PowerPC, SPARC, M68K, RISC-V, s390x, TriCore. It covers 68000–68060 and ColdFire, ARM926/946/1176/Cortex, PowerPC 601–7450, 750 (incl. 750CL) and 970, and MIPS32/MIPS64 (R4000, VR5432). It has no 8-bit CPU at all, no SuperH, no TMS9900, no CP1600, no Jaguar RISC, no SPU; R5900 is defined in QEMU's headers but not offered as a CPU model, and paired singles and VMX128 are absent from its source.

### Cited Findings
- `uc_arch` enum: UC_ARCH_ARM, ARM64, MIPS, X86, PPC, SPARC, M68K, RISCV, S390X, TRICORE — [unicorn.h](https://github.com/unicorn-engine/unicorn/blob/master/include/unicorn/unicorn.h)
- QEMU targets in the tree: arm, i386, m68k, mips, ppc, riscv, s390x, sparc, tricore — [unicorn qemu/target](https://github.com/unicorn-engine/unicorn/tree/master/qemu/target)
- M68K CPU models: M68000, M68020, M68030, M68040, M68060, M5206, M5208, CFV4E, ANY — [m68k.h](https://github.com/unicorn-engine/unicorn/blob/master/include/unicorn/m68k.h)
- ARM CPU models include 926, 946, 1026, 1136, 1176, 11MPCore, Cortex-A7/A8/A9/A15, Cortex-M0/M3/M4/M7/M33, Cortex-R5, PXA25x/27x, SA1100/SA1110 and TI925T; there is no ARM7TDMI model by name — [arm.h](https://github.com/unicorn-engine/unicorn/blob/master/include/unicorn/arm.h)
- MIPS models: MIPS32 4Kc…P5600, I7200, R6 generic; MIPS64 R4000, VR5432, 5Kc, 20Kc, I6400/I6500, Loongson 2E/2F — [mips.h](https://github.com/unicorn-engine/unicorn/blob/master/include/unicorn/mips.h)
- QEMU's MIPS headers define `INSN_R5900`, `ASE_MMI` and `CPU_R5900`, but the CPU table in Unicorn has no R5900 entry — [mips-defs.h](https://github.com/unicorn-engine/unicorn/blob/master/qemu/target/mips/mips-defs.h); [translate_init.inc.c](https://github.com/unicorn-engine/unicorn/blob/master/qemu/target/mips/translate_init.inc.c)
- PPC models include 601, 602, 603, 604, 740/750 (750CL, 750CX/CXe, 750FX/GX), 7400/7410/7445–7457, e500/e600 and PPC64 970/970FX/970MP, POWER5–POWER10 — [ppc.h](https://github.com/unicorn-engine/unicorn/blob/master/include/unicorn/ppc.h)
- The Unicorn repository contains no `ps_madd` and no `VMX128` — [Unicorn repository](https://github.com/unicorn-engine/unicorn)
- Unicorn is GPL-2.0 per GitHub's licence detection — [unicorn-engine/unicorn](https://github.com/unicorn-engine/unicorn)

Cores for the gaps (licence per GitHub detection or file header):
- 68000 (alternatives to Unicorn): [Musashi](https://github.com/kstenerud/Musashi) — MIT-style permission notice in readme, Copyright 1998–2001 Karl Stenerud; [Moira](https://github.com/dirkwhoffmann/Moira) — core is MIT per its LICENSE
- 6502: [py65](https://github.com/mnaberez/py65) — Python, BSD-3-Clause; [mre/mos6502](https://github.com/mre/mos6502) — Rust no_std, BSD-3-Clause, supports variants; [floooh/chips](https://github.com/floooh/chips) — C headers for 8-bit chips (Z80, 6502 and others), zlib
- Z80 / 8080: [ivanizag/iz80](https://github.com/ivanizag/iz80) — Rust, Z80 and 8080, passes ZEXALL, BSD-3-Clause; [redcode/Z80](https://github.com/redcode/Z80) — ANSI C, LGPL-3.0; [superzazu/z80](https://github.com/superzazu/z80) — C99, MIT
- MAME CPU cores: MAME as a whole is GPL-2.0, individual files may carry less restrictive licences — [MAME COPYING](https://github.com/mamedev/mame/blob/master/COPYING); the m6809, g65816, tms9900, cp1610, jaguar and m68000 core files are headed `license:BSD-3-Clause` — [mame/src/devices/cpu](https://github.com/mamedev/mame/tree/master/src/devices/cpu)
- MAME has cores for, among others, m6502, g65816, h6280, lr35902, z80, m6809, m6800, m6805, m68000, tms9900, cp1610, jaguar, sh, mips, psx, powerpc, arm7, rsp, spc700, superfx — [mame/src/devices/cpu](https://github.com/mamedev/mame/tree/master/src/devices/cpu)
- Multi-system: [TomHarte/CLK](https://github.com/TomHarte/CLK) — MIT
- Full-system emulators covering missing CPUs: PS2 [PCSX2](https://github.com/PCSX2/pcsx2) (GPL-3.0); PS3 incl. SPU [RPCS3](https://github.com/RPCS3/rpcs3) (GPL-2.0); Dreamcast SH-4 [Flycast](https://github.com/flyinghead/flycast) (GPL-2.0); Game Boy [mooneye-gb](https://github.com/Gekkio/mooneye-gb) (GPL-3.0); PSP [PPSSPP](https://github.com/hrydgard/ppsspp), GameCube/Wii [Dolphin](https://github.com/dolphin-emu/dolphin), Xbox 360 [Xenia](https://github.com/xenia-project/xenia), multi-system [ares](https://github.com/ares-emulator/ares) (licence not machine-detected by GitHub for these four)

### Inferences
- For 68000 (Mega Drive, Amiga, Neo Geo) and ARM (GBA, DS, 3DS) and MIPS (PS1, N64) functions, Unicorn can be the ground-truth runner. For the GBA, ARM926/946 or TI925T (ARMv4T/v5) models are the nearest; ARMv5 adds instructions the ARM7TDMI lacks, which only matters if code is malformed.
- Unicorn's MIPS does not model the R3000A load delay slot (QEMU executes MIPS32 semantics); PS1 code that relies on reading the stale register would differ. This is untested.
- For 6502, Z80, SM83, 6809, 65816, TMS9900, CP1600, SH-2/SH-4, Jaguar RISC, R5900, SPU, Gekko paired singles, GameFrame needs a separate core. The BSD-3-Clause MAME cores and the BSD/MIT Python/Rust cores are compatible with GameFrame's Apache-2.0 code; GPL emulators are usable as external tools but not as copied code.

### Gaps
- Whether Unicorn's 750CL model decodes paired-single opcodes at all (likely an illegal-instruction exception) is untested.
- A permissive SH-2/SH-4 standalone core, a 65816 Python/Rust core, a 6809 Python core, and a SM83 standalone core are not known.

## Other disassemblers and tools per family

### Takeaway
Capstone (on `next`) covers 6502/65C02/65816 (MOS65XX), the 6800/6809/6309 family (M680X), 68K, x86, ARM/AArch64, MIPS (incl. MIPS III), PowerPC with a paired-singles mode, and SH-2/SH-2A/SH-3/SH-4. It has no Z80, SM83, TMS9900, CP1600, SPU or Jaguar RISC. 8-bit families rely on cc65's da65, z80dasm and SkoolKit; radare2/rizin cover broad sets.

### Cited Findings
- Capstone `cs_arch`: ARM, AARCH64, SYSTEMZ, MIPS, X86, PPC, SPARC, XCORE, M68K, TMS320C64X, M680X, EVM, MOS65XX, WASM, BPF, RISCV, SH, TRICORE, ALPHA, HPPA, LOONGARCH, XTENSA, ARC — [CS](https://github.com/capstone-engine/capstone/blob/next/include/capstone/capstone.h)
- Capstone modes include `CS_MODE_MOS65XX_6502`, `_65C02`, `_W65C02`, `_65816` (with LONG_M / LONG_X width variants); `CS_MODE_M680X_6800`, `_6801`, `_6805`, `_6809`, `_6309`, `_6301`, `_6811`, `_HCS08`; `CS_MODE_SH2`, `SH2A`, `SH3`, `SH4`; `CS_MODE_PS` (paired-singles, PPC); `CS_MODE_MIPS3` — [CS](https://github.com/capstone-engine/capstone/blob/next/include/capstone/capstone.h)
- No Z80, R5900 or Allegrex mode appears in capstone.h — [CS](https://github.com/capstone-engine/capstone/blob/next/include/capstone/capstone.h)
- cc65 (C compiler for 6502 systems, includes da65 disassembler) — zlib — [cc65/cc65](https://github.com/cc65/cc65)
- SkoolKit, tools for disassemblies of ZX Spectrum games — GPL-3.0 — [skoolkid/skoolkit](https://github.com/skoolkid/skoolkit)
- radare2 — [radareorg/radare2](https://github.com/radareorg/radare2); rizin (LGPL-3.0) — [rizinorg/rizin](https://github.com/rizinorg/rizin)

### Inferences
- GameFrame already depends on Capstone; for 65816 targets the disassembly width must be chosen per function (LONG_M/LONG_X modes), mirroring the M/X flag state.
- For Z80 and SM83, Capstone is not an option; Ghidra (Z80 built in, SM83 via GhidraBoy) or z80dasm/SkoolKit fill the gap.

### Gaps
- IDA ships processor modules for the 6502, 65816, Z80, 6809, 68K, TMS9900 and others (not confirmed).
- z80dasm's repository and licence, and the radare2 and rizin plugin lists per architecture, are not confirmed.
- mGBA, no$gba, Mesen, bsnes-plus and other debugging emulators per platform are not covered in these notes.
