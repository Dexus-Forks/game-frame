---
id: xaudio2-wma
title: A WMA decoder for Wine, for games that play xWMA sounds through XAudio2
kind: layer
runtime: [emupc-wine]
provides: [api:xaudio2, media:wma]
requires: [emupc-wine]
vars: [VM_DIR, VM_NAME]
seen: ['Banished (2014), in emupc-wine, 2026-10-24']
---

# A WMA decoder for Wine, for games that play xWMA sounds through XAudio2

## What it does

Wine decodes WMA through GStreamer. Without GStreamer's libav plugins, a game that plays xWMA
sounds crashes with an access violation in `xaudio2_7.dll`. This module installs them in the
emulated PC's Debian.

## Set up

```bash
ssh -F "$VM_DIR/ssh_config" "$VM_NAME" sudo apt-get install -y gstreamer1.0-libav gstreamer1.0-plugins-good
```

Before the VM's first boot, the same packages can go into `$VM_DIR/cloud-init/user-data`
instead.

## Check

The game plays past the point where it first loads a sound, and its log shows no exception in
`xaudio2_7.dll`.

## Pitfalls

- Access violation in `xaudio2_7.dll`: no WMA decoder. Install the packages above.
- Wine on a macOS host needs GStreamer too. No module covers that yet (not confirmed which
  packages it needs).
