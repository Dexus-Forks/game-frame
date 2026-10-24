---
id: sudo-drops-display
title: 'A driver run with sudo sends no keys: "Can''t open display"'
kind: failure
status: solved
era: []
platform: [linux]
runtime: [virtual-machine]
hardware: []
api: []
genre: []
symptom: [no-input]
tags: [sudo, xdotool, display, environment, automation]
signatures:
  - 'failed creating new xdo instance'
  - 'can''t open display'
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A driver run with sudo sends no keys: "Can't open display"

## Symptoms

A script that reads game memory as root and steers with `xdotool` prints `Failed creating new
xdo instance` and `Error: Can't open display: (null)`; the game gets no keys.

## Cause

`sudo` resets the environment and drops `DISPLAY` (*observed*).

## What to do

Pass it through: `sudo env DISPLAY=:1 python3 follow.py ...`.
