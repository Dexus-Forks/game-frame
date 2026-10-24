---
id: input-driver-keys-held-after-kill
title: The player's car drives itself after a scripted driver was stopped
kind: failure
status: solved
era: []
platform: [linux]
runtime: [virtual-machine, wine]
hardware: []
api: [directinput]
genre: []
symptom: [tool-error]
tags: [xdotool, keydown, timeout, sigterm, input, automation, held-keys]
signatures: []
tools: [winput]
skills: [re-observe]
docs: [docs/observe/inputs.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# The player's car drives itself after a scripted driver was stopped

## Symptoms

In the next race the player's car accelerates or turns with no input. Positions read from memory
show it moving from the countdown.

## Cause

The driver script holds keys with `xdotool keydown` and releases them with `keyup` on its next
step. `timeout` stops it with SIGTERM between the two, so the keys stay down in the X server
(*observed*: `xdotool keyup Up Down Left Right` stopped the car).

## What to do

Release every key the driver can hold in a `finally` block, and turn SIGTERM into a normal exit
(`signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))`). After any killed run, send the keyups
by hand before the next race. GameFrame's `winput` releases every key that its `down` step
holds when it ends, and has a `release` step for a run that was killed.
