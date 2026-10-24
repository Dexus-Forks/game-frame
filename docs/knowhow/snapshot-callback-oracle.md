---
id: snapshot-callback-oracle
title: 'Verifying a system that calls into another, on a snapshot of the running game'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [callback, oracle, snapshot]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Verifying a system that calls into another, on a snapshot of the running game

## What to know

*2026-10-24, Hardwar's `snap_emu_reset.py`, Hardwar's `collide.py`, `crates/flight/tests/collide.rs`.*

The collision step (`collide_update` 0x4f4234) is a few thousand instructions of its own, but
every test against the city goes through callbacks into the world system, which walk a grid of
objects and their shapes that only `world_install` builds. Porting the world first would have
held the collision port back by days. Three practices made the step testable bit for bit
without it.

**Run on a whole-process snapshot, not on a built world.** Hardwar's `vm_snapshot.py` copies
every writable mapping of the running game between two steps (227 MB). `snap_emu_reset.py` maps all
of it into Unicorn read, write and execute, so the original's code finds its heap, its world
and its 74 colliders where they were. Writes are undone page by page between cases (a write
hook marks the pages, and the harness marks its own writes too: Python's `mem_write` does not
fire the hook). The snapshot does not hold the x87 control word; set it (`0x0c7f`) with a
two-instruction stub before the first call.

**Let the callee run, record what it did at the boundary.** The world's callbacks run for
real. A code hook at each callback's entry records the shared globals; one at each return
address records them again, with the result and what the callee wrote through its pointer
arguments; a hook at the one routine of the collision system that the world calls back
(0x4f7730, the hit report) records its inputs. The port reaches the world through a trait, and
the test answers each call from the recording: it checks that the globals the world reads are
the port's, replays the hit reports through the port's own 0x4f7730, then sets the globals to
the recorded state after the call. So the port's code between callbacks is checked at every
call, and its whole result at the end. The generator also checks that the callbacks change the
colliders only where 0x4f7730 does, so nothing is missed.

**Perturb towards the edges.** The recorded Moths were all flying in open air. Moves that
reached the interesting code: a Moth placed next to another collider, heading for it (pair
hits); a Moth at the centre of its free box, going out through one face with a step that
crosses it (world hits and the back-off); waiting orientations; dropped free boxes (the world
asked again).

A pitfall met on the way: restoring a source file with `mv file.bak file` after a mutation
test keeps the backup's older modification time, so `cargo` does not rebuild and the next test
runs the mutated build. Touch the file, or copy instead of moving.
