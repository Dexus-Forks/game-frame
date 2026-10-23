---
id: directinput-not-foreground
title: 'DirectInput ignores keys, because a window that is not foreground cannot acquire the device'
kind: failure
status: workaround
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
api: [directinput]
symptom: [no-input]
tags: [acquire, foreground, focus, dinput]
signatures:
  - 'not (?:the )?foreground'
  - 'DIERR_NOTACQUIRED'
  - 'DIERR_INPUTLOST'
tools: []
skills: [re-observe]
docs: [docs/observe/inputs.md, docs/knowhow/wine-input-without-accessibility.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# DirectInput ignores keys, because a window that is not foreground cannot acquire the device

## Symptoms

Keys or the mouse do nothing in the game, though the window shows. `WINEDEBUG=+dinput` reports
that the window is not foreground. Calls may return `DIERR_NOTACQUIRED` or `DIERR_INPUTLOST`.

## Cause

A DirectInput device set to foreground cooperation (`DISCL_FOREGROUND`) can be acquired only
while its window is the foreground window. A window started from a script, or behind another
program, is not.

## What to do

Bring the game's window to the foreground before you send input. Then wait a moment, so the
game can acquire the device again. Inside Wine, a helper that brings the window forward and
then sends input with `SendInput` does both ([input inside Wine](wine-input-without-accessibility.md)).
In POD this helped only in part (*observed*).
