---
id: held-keys
title: 'Keys must be held for a frame to see them'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [held, keys]
tools: [winput]
skills: [re-emulate, re-observe]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Keys must be held for a frame to see them

## What to know

*2026-10-24, Wine 10 in the emulated PC, xdotool on Xvfb.*

Symptom: `xdotool type` into the pilot-name box gave "TH" for "Truth", and a tapped Return
did nothing.

Cause: the game reads the keyboard once a frame, through DirectInput, into the key table at
`0x59a188` (a byte per key, bit 0 "pressed since read", bit 1 "down"; `FUN_004abb78`,
`FUN_004abbb0`, *observed*). On the shared emulated PC the game ran at 6 to 11 frames a
second (`dt_ms` 90-166, *measured*). xdotool's taps last about 12 ms, so most fall between
two reads (*inferred*).

Fix: hold each key for at least two frames. Hardwar's `vm_drive.sh` holds keys 0.4 s by
default, types one letter every 0.6 s, and holds mouse buttons 0.4 s.

For GameFrame: a general pitfall of driving a slow emulated game with synthetic input. The
`winput` and xdotool recipes should hold keys for a few frames by default.
