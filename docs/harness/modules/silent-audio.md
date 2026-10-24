---
id: silent-audio
title: A silent sound device in the emulated PC, so the game has sound and nothing is heard
kind: layer
runtime: [emupc-wine]
provides: [policy:silent-audio]
requires: [emupc-wine]
vars: [VM_DIR, VM_NAME]
seen: ['Banished (2014), in emupc-wine, 2026-10-24']
---

# A silent sound device in the emulated PC, so the game has sound and nothing is heard

## What it does

Some games crash or stop without a sound device. Others run with one, and play sound nobody
wants on an agent's machine. This module makes ALSA's default device the `null` device. The
game opens a working device, its sound code runs as it would on a PC, and nothing is heard. The
analyser never asks for it: add it with `harness.py plan ... --with silent-audio`.

## Set up

```bash
ssh -F "$VM_DIR/ssh_config" "$VM_NAME" 'printf "pcm.!default { type null }\nctl.!default { type hw card 0 }\n" > ~/.asoundrc'
```

## Check

`ssh -F "$VM_DIR/ssh_config" "$VM_NAME" cat ~/.asoundrc` shows `type null`. The game starts and
plays without error.

## Pitfalls

- To measure the game's sound, use a device that records instead of `null`
  ([recording](../../observe/recording.md)).
