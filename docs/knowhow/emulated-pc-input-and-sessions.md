---
id: emulated-pc-input-and-sessions
title: In a shared, loaded VM the game dies with the SSH session or misses key presses
kind: failure
status: workaround
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
symptom: [wont-start, no-input]
tags: [ssh, setsid, process-group, xdotool, key-press, load, shared-emulator, cd-label]
tools: [shot]
skills: [re-observe]
docs: [docs/observe/running-the-original.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# In a shared, loaded VM the game dies with the SSH session or misses key presses

## Symptoms

- A game started with `nohup wine ... &` over `ssh` quits soon after the SSH command returns,
  with an empty log.
- Menu keys sent with a short press (`podinput`, `xdotool key`) are ignored when the VM is
  busy (load average 19 to 24 on 4 CPUs), then arrive late, several at once.
- With two editions' discs in two drives, both labelled `POD2_0`, the game reads the wrong one.

## Cause

- The game stays in the SSH session's process group and gets its hang-up (*measured*: alive
  in the foreground, gone under `nohup` alone, alive under `setsid`).
- A press shorter than a frame of the slowed game is not seen by DirectInput (*inferred*).
- The game takes the first CD-ROM drive with the right label (*observed* in the drive scan).

## What to do

- Start the game with `setsid nohup ...`.
- Hold keys for 0.4 s (`xdotool keydown K; sleep 0.4; xdotool keyup K`) and look at the
  screen before the next press.
- Give each edition its own Wine prefix whose `dosdevices` maps only its own disc's drive; a
  second disc can be a `cdemu` device (`cdemu add-device`, `cdemu load N FILE.CUE`).
