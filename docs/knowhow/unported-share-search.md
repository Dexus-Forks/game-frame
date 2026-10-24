---
id: unported-share-search
title: 'Measuring ported systems when an unported one draws first'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [search, share, unported]
tools: []
skills: [re-observe, re-port, re-verify]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Measuring ported systems when an unported one draws first

## What to know

*2026-10-24, `crates/sim/tests/recorded.rs`, `docs/sim.md` ("Against the recorded runs").*

The recordings give the random streams' state at one point of each step. A port that lacks
some of the step's draws can be compared by counting how far it is short of the original
on each step. But a ported system that draws *after* an unported one draws other numbers
than the original's: the ambient systems (the flash, the fog, the drift) came out right in
count and wrong in value, and their countdowns then fired on other steps.

When the unported system is the first of the step to draw (`pilots_update` here), its share
can be found: for `k` from 0 up to the step's total, make `k` draws, run the ported step on
a copy of the world, and keep the `k` that lands exactly on the original's state. Two
details mattered:

- **The search is ambiguous.** A smaller `k` and a longer branch of a ported system can land
  on the same state (a change of course of 6 draws after 3, or of 5 after 4). The unported
  system's count changes rarely between steps, so trying last step's `k` first picked the
  right one; with it, every paused step of run4 came out `k = 4`, and the ported state
  equalled all five memory dumps.
- **Know when the state is in sync.** On a step the search cannot explain (the vehicles and
  weapons draw between the ported systems in flight), a ported system that draws takes
  other numbers, and from then on its state is not the original's until the next memory
  dump resets it. The test tracks that and requires exactness only while in sync; out of
  sync it re-runs the step with those systems silenced, so the rest of the world is still
  held to never drawing past the original.

For GameFrame: a general method for comparing a partial port against recordings of a
single shared random stream (`re-verify`).
