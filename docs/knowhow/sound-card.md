---
id: sound-card
title: Sound cards and audio paths in games
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine]
hardware: [sound-card]
api: [dos-ints, winmm, directsound, mci, card-sound-api]
compiler: []
genre: []
symptom: []
tags: [sound, audio, music, sound-effects, midi, sound-blaster]
tools: [peinfo]
skills: [re-trace, re-observe, re-verify]
docs: [docs/hardware/sound.md, docs/hardware/sound-card-apis.md, docs/hardware/replacement-libraries.md, docs/knowhow/sound-card-library-empty-answer-silences.md, docs/knowhow/wine-mixer-wave-volume-zero.md, docs/knowhow/record-sound-without-a-sound-card.md, docs/knowhow/x87-one-multiply-two-channels.md]
seen: ['POD Gold (1997), 2026-10-24']
---

# Sound cards and audio paths in games

## What to know

First find which sound path the game uses. DOS games may write to card ports, handle sound
interrupts, or call a driver. Windows games may use WinMM, DirectSound, MCI, or a card maker's
own library. Record the selected card, then identify which path handles the music, effects, and
timing.

Read the manual and sound setup screen, then check the files the game ships. For Windows builds,
inspect imported libraries and search strings for dynamically loaded library names. Trace the
callers to see what the game sends, what answers it expects, and what it does when an answer is
empty or unsupported. A successful return value alone does not show that the game received a
usable device.

Separate sound generation from sound output. A game can produce buffers or MIDI events while a
mixer, Wine, or a virtual machine sends silence to the speakers. Check the game's volume path
and the host's output route. When recording for a port, keep timing evidence such as register
writes, interrupts, or MIDI events separate from the final audio samples. Host conversion and
emulator timing can change a recording without changing the game's sound logic.

The linked sound-card API guide has measured detail for the Maxi Sound 64 and unverified
descriptions for several other vendor APIs. Check those details against the game before relying
on them.
