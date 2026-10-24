---
id: in-flight-save-ends-on-continue
title: 'A save made in flight ends the game when continued'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [continue, ends, flight, save]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A save made in flight ends the game when continued

## What to know

*2026-10-24, the original in Hardwar's own emulated PC (Wine 10), recording copy of the EXE.*

Symptom: a save written by the port loaded in the original, showed "'NAME' Paused" with the
port's day and time, and about three seconds after Continue the game closed and the Hardwar
Manager opened. It looked like the original rejecting the port's save.

Cause: not the port. The original's own `3-in-flight-d2-0014.SAV`, unchanged, does the same:
43 unpaused steps after Continue, then the Manager (*measured*, `derived/save-write/orig3`).
The port's in-flight save ran 36 (idle) and 43 (full throttle) unpaused steps before the same
end. In single player the game ends and starts the Manager when the player's pilot dies
(`docs/ground-truth.md`), so the Moth stopped in flight probably falls or is killed after a
load (*inferred*; what kills it is *unknown*).

Fix: to check that the original accepts a written save, compare it with the original's own
save under the same actions, and prefer a docked save (save 2), which the recorded runs
continue and launch from.

For GameFrame: before blaming a written save, run the unmodified original save through the
same steps. A control run costs one VM turn and rules out the writer.
