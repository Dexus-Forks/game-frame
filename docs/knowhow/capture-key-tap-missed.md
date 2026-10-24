---
id: capture-key-tap-missed
title: A scripted key tap the game never saw
kind: failure
status: workaround
era: [late-1990s]
genre: [racing-sports]
symptom: [no-input]
tags: [directinput, keyboard, input, capture, reference-frames, podinput, timing]
tools: [winput]
skills: [re-observe, re-verify]
docs: [docs/observe/inputs.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A scripted key tap the game never saw

## Symptoms

Reference frames captured with a key script drift from the port by one step: one capture
shows the screen as it was before the key, and every later capture is one key behind.

## Cause

The game reads the keyboard's state (DirectInput `GetDeviceState`) once a frame. A script that
sends key down and key up within a few milliseconds can fall between two reads, and the game
never sees the key. In POD one "Enter" on the records screen was lost this way.

## What to do

Hold scripted keys for at least two frames (`podinput` steps `KEY` with a `wait` while held, or
a hold option). When checking existing captures, find the step the game missed and replay the
sequence as the game saw it; `crates/pod-ui/tests/frames.rs` lists such captures in `MISSED`
with the reason.
