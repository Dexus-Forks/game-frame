---
id: context-mod-value-replaced-at-run-time
title: A changed value in a data file loads but changes nothing in the game
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, virtual-machine]
hardware: []
api: []
genre: []
tags: [modding, handling, settings, memory]
commands: []
tools: [memrec, memscan]
skills: [re-trace, re-observe]
docs: [docs/observe/memory.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A changed value in a data file loads but changes nothing in the game

## What to know

A mod changes a value that the game's loader reads, such as a car's grip, and the game behaves
as before. The file did load: other changes in the same file show.

The value the loader reads is a default. Later, before the race, the game replaces it with one
computed from saved or menu settings (*measured* in POD: the garage setup).

In POD the car file's `side_grip` and `traction_grip` (+0x94, +0x98 of each wheel record) are
read by `load_car_file` and copied into the race car by `FUN_00476aa0`. Then
`setup_defaults` (`FUN_004436c8`) computes one grip for all four wheels from the car's third
garage setting ("adhérence", 0 to 100), and `car_from_setup` (`FUN_00442f04`) writes it over
the copied values: grip = 1.2 + 0.026 × setting (*measured* at 95, 50 and 5). The five
garage settings at the end of the car file are the car screen's five figures.

## Evidence

Read the running game's memory (`/proc/PID/mem` of the Wine process). The changed values were
in the car's slot (`0x61757c`, +0x94), so the file had loaded. The race car (`0x61aad0`,
+0x94) held 3.67 on every wheel with the mod and with the disc's file. A memory search for that
value found the setup at `0x50769c + car × 0x1a8 + 0x13c`; the decompiled writers of race car
+0x94 led to `car_from_setup`.

## What to do

Before judging a handling or rules mod by play, read the value back from the running game
where the rules use it, not only where the loader puts it:

1. Find the structure the game logic reads (a memory diff while the value matters, or the
   reader named in the format notes).
2. Compare it with and without the mod. Equal values mean something later replaces it.
3. Search memory for the value the game uses, then find the code that writes it: often a
   settings or setup step (garage, difficulty, options).
4. Change the source of that value instead. In POD: the grip setting (`make_mod setting FILE
   OUT 2 N`), which shows on the car screen and changes the race car's grip.

## What was tried

- Grip parameters of all four wheels set to 1.2: race totals 8:05.50 against 8:06.05 with the
  disc's file, within the driver's spread. The race car's grip was unchanged.
