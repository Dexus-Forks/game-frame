---
id: wine-linux-cd-drive
title: The game asks for its CD under Wine on Linux, though the disc's folder is mapped with the right label
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
hardware: [cd-rom]
api: [win32]
symptom: [file-not-found, wont-start]
tags: [cd-check, getdrivetype, volume-label, iso, device-permission]
signatures:
  - 'cannot open device [A-Z]:'
  - 'open device .*c0000022'
tools: [discinfo, emupc]
skills: [re-observe]
docs: [docs/knowhow/insert-cd-file-open-failed.md, docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), Wine 10 on Debian, 2026-10-23']
---

# The game asks for its CD under Wine on Linux, though the disc's folder is mapped with the right label

## Symptoms

"Insert the CD" with the disc's files mapped as drive D: and the volume label set. With a
real CD device attached, Wine's `+volume` trace shows `cannot open device D: c0000022`.

## Cause

Two causes, one after the other (*observed* in POD with `WINEDEBUG=+volume`):

- Under Wine on Linux a folder mapped as a drive is a fixed drive: `GetDriveTypeW` returns 3,
  even with the drive's type set in `HKLM\Software\Wine\Drives`. On macOS the same setting
  worked. A game that looks for a drive of type 5 never finds the disc.
- Wine opens the CD device to read its label. `/dev/sr1` was readable by root only, so the open
  failed with `c0000022` (access denied).

## What to do

Attach the disc's data track as a CD-ROM device, not a folder: cut it out of the BIN/CUE with
`./gameframe run discinfo IMAGE.cue --iso DISC.iso`, attach the ISO as a CD-ROM drive in the
emulator, and map Wine's D: to the device's mount point. Make the device readable by everyone
(a udev rule: `KERNEL=="sr[0-9]*", MODE="0644"`; `emupc.py` writes it).
