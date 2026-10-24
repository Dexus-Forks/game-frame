---
id: replay-sample-straddles-frames
title: A replay matches the recording at the end but not at some samples, one sample early or late
kind: failure
status: solved
era: [late-1990s]
runtime: [virtual-machine]
genre: []
symptom: [tool-error]
tags: [replay, recording, sample, frame, straddle, proc-mem, verify, race]
tools: [memrec]
skills: [re-verify, re-observe]
docs: [docs/knowhow/record-a-run-from-memory.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A replay matches the recording at the end but not at some samples, one sample early or late

## Symptoms

A port replaying a recorded run ends with the original's results, but comparing its state
with each sample fails: a lap counts one sample later in the port, two samples show the
same clock, a sample's records already hold a crossing stamped after the sample's own
clock. In POD's first replay 1,038 of 4,489 samples differed this way, every one a sample
early or late.

## Cause

The recorder reads the game from outside while it runs (`/proc/PID/mem`, ten times a
second). It reads the clock first, then the positions, then the records, so one sample can
straddle frames: the records can be a frame or more newer than the clock (*measured*: in
one POD sample 0.57 s of game time, with the VM busy), and the positions a frame older
than the records. And the game stamps a crossing with the clock at the start of the frame,
before the frame's time is added.

## Evidence

Printing the port's state and the sample side by side at the first difference; the
recorder's read order in `run/vm/racerec.py`; two consecutive samples with the same clock.

## What to do

Keep every state the port passes through (after each frame), and accept a sample when one
of them, from the sample's clock to the end of the frame that runs at the next sample's
clock, equals it. Compare only what the sample read as a block (here the race records) as
one value. In POD every sample then matched. A recorder hook inside the game loop that
writes each frame avoids the window altogether.

## Tried

- Comparing at the sample's clock exactly: 1,112 differences, all straddles.
- A window up to the next sample's clock: one difference left, where two samples shared a
  clock and the records came from the frame after.
