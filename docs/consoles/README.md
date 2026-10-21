# Console games

A console game runs on fixed hardware with no operating system to speak of. The game owns the
whole machine: it writes to the video chip's registers, starts DMA transfers, reads the
controllers from memory-mapped ports and times everything to the display's vertical blank.
That makes console games different to reverse engineer from PC games, and different to port.

This section covers how to study console games and how to bring them to other systems. It uses
the harness's method unchanged; the tools and the hardware details differ.

## Pages

- [Console platforms](platforms.md): the consoles from the NES to the sixth generation, with
  their CPUs, video and sound chips, and what each means for the work.
- [Tools for console work](tools.md): Ghidra processors and loaders, emulators with debuggers,
  Unicorn's architectures, and splitting tools.
- [Routes to a port](porting.md): reimplementation, matching decompilation, static
  recompilation and emulation, and how to replace each part of the hardware.
- [Matching decompilation](matching-decomp.md): how the decompilation projects work, and when
  that route is worth its cost.

## How console work differs from PC work

| Topic | PC games | Console games |
|---|---|---|
| The program | An EXE with imports, sometimes symbols | A ROM or a disc image; code at fixed addresses, often banked; no imports |
| The CPU | x86 | 6502, 65816, Z80, 68000, MIPS, SH-2, SH-4, PowerPC, ARM; often two or more CPUs |
| Hardware access | Through drivers and APIs | Direct writes to memory-mapped registers |
| Time | A timer interrupt or the frame | The vertical blank, at 60 Hz (NTSC) or 50 Hz (PAL) |
| Graphics | A frame buffer | Often tiles, sprites and layers built by a video chip; frame buffers from the 32-bit generation |
| Sound | A sound card or an API | A sound chip or a separate sound CPU with its own program |
| Saves | Files | Battery-backed RAM in the cartridge, memory cards, passwords |
| Ground truth | Run the EXE in an emulator or on a PC | Run the ROM in an accurate emulator with a debugger |
| Source and symbols | Sometimes symbols, rarely source | Sometimes symbol maps on discs and in prototypes; leaked source exists for some games and must not be used ([Legal](../LEGAL.md)) |

## Where to start

1. Identify the console, the region and the exact revision of the game. Regions differ in
   timing (50 or 60 Hz), text and sometimes code. Record the checksum of the ROM or disc.
2. Get an accurate emulator with a debugger running the game ([Tools](tools.md)).
3. Load the code into Ghidra with the right processor and memory map. For banked cartridges,
   load each bank at the address it runs at.
4. Find the reset vector and the vertical blank interrupt handler. The main loop and the frame
   timing follow from them.
5. Find the controller read. It is usually one function called once a frame, storing the
   buttons in RAM. That is the input record ([Inputs](../observe/inputs.md)).
6. Map the RAM: which addresses hold the player, the enemies, the level, the random state.
   Emulators' RAM watch and search features find these fast. Write them into `labels/`.

Then follow the usual loop: trace, emulate, port, verify.
