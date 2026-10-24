---
id: replay-stand-in-for-unported-logic
title: Porting more of the original made a replay agree less with the recording
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [tool-error]
tags: [replay, port, stand-in, ai, lanes, path, verify, recording, race]
tools: []
skills: [re-port, re-verify]
docs: [docs/knowhow/context-port-rules-before-physics.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Porting more of the original made a replay agree less with the recording

## Symptoms

After porting a function exactly (proved under the emulator), the replay of a recorded run
matches the recording *less* often than before. In POD, porting the order between gates
(the path tracker `FUN_004b0ec0` and the order update `FUN_004afd3c`) dropped the samples
with the game's order from 77% to 54%.

## Cause

The ported function depends on state from code not yet ported, and the replay supplied it
with a stand-in. In POD a computer car's place on the race paths is set at each "Troncon"
gate from the lane its driver chooses (`FUN_004593fc`), which is not ported. The replay
guessed the lane from the nearest first points, chose pit and special lanes the game never
takes (entries flagged 0x80 or of type 4 to 6), and the trackers stuck where the car left
that lane (*measured*: each tracker's link and value printed where the orders parted).

## Evidence

The agreement rate per stretch of the race (in runs of 200 samples), then the trackers'
link, point and value at the first long run of differences.

## What to do

Build a stand-in from the original's own rules first, then fill in only what the recording
lacks: here, keep to the lanes the game's chooser can take, and pick among them by where
the car drove in the next two seconds. POD's replay went to 98.3%, then 99.4%. Mark the
stand-in `Deviation:` in the test, report the agreement rate, and let the test require it
(here at least 99%) so a regression shows. Record the missing state next time (POD's
recorder now reads each driver's lane and path state).

## Tried

- Nearest first points of any link out: 54%.
- Nearest points among the lanes the chooser can take: 98.3%.
- The same, scored on the car's next 20 sampled positions: 99.4%; the remaining short runs
  need per-step positions.
