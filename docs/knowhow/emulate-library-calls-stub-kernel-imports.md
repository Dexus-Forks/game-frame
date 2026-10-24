---
id: emulate-library-calls-stub-kernel-imports
title: Measure a library's calls that lock mutexes, under the emulator
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [unicorn]
compiler: [watcom]
hardware: [x86-cpu]
genre: []
symptom: []
tags: [unicorn, imports, iat, stub, mutex, sound, measure, sequence]
tools: [pe32_emu]
skills: [re-emulate, re-verify]
docs: [docs/hardware/sound.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Measure a library's calls that lock mutexes, under the emulator

## What to know

A sound or input library's entry points often wrap their work in `WaitForSingleObject` and
`ReleaseMutex`, and report errors with `MessageBoxA`. Under a bare emulator those calls jump
to the import table's unresolved entries and crash. Write a few bytes of stub for each into
scratch memory and point the import slots at them: `xor eax, eax; ret 8` for
`WaitForSingleObject` (`WAIT_OBJECT_0`), `mov eax, 1; ret 4` for `ReleaseMutex`, `ret` for
`DebugBreak`, `xor eax, eax; ret 16` for `MessageBoxA` and `PostMessageA`. A game may import a
function twice (POD has two `ReleaseMutex` slots); stub every slot. Get the slots with
`pefile` (`DIRECTORY_ENTRY_IMPORT`, `imp.address`).

Then the library's own calls can be run in random sequences against its own tables, and the
port checked after every call (`measure/sound_channels.py`: play, update, stop, pause and mix
in any order). Two things make such a measurement worth its keep:

- Make sure the random sequences reach the rare paths. In POD a slot index that wraps on the
  eighth update went untested until a third of the cases kept to two looping channels.
  Break the port on purpose and see the test fail.
- Keep inputs the original cannot survive out of the cases, and note them: a loop shorter than
  one step never ends in POD's mixer, and the emulator's instruction limit hides the hang.
