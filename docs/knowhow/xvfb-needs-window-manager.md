---
id: xvfb-needs-window-manager
title: Keys reach a game under Wine on Xvfb only with a window manager running
kind: failure
status: solved
platform: [linux]
runtime: [wine, virtual-machine]
api: [directinput]
symptom: [no-input]
tags: [xvfb, window-manager, openbox, foreground, xdotool]
tools: [emupc, winput]
skills: [re-observe]
docs: [docs/knowhow/directinput-not-foreground.md, docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), Wine 10 on Debian, 2026-10-23']
---

# Keys reach a game under Wine on Xvfb only with a window manager running

## Symptoms

On a headless X server (Xvfb), the mouse works in the game but keys do nothing. Keys sent
with `xdotool` reach the window. The game ignores them.

## Cause

With no window manager, no window becomes the foreground window. A DirectInput device with
foreground cooperation then stays unacquired, so the game reads no keys (*observed*: keys moved
the menu highlight only after a window manager ran).

## What to do

Start a small window manager on the display before the game: `openbox &` (or any other). Wine's
virtual desktop (`explorer /desktop=`) does not replace it. On Xvfb under Wine 10 it gave a
black desktop of the wrong size.

## Tried

- Wine's virtual desktop on Xvfb: a black desktop of the wrong size.
