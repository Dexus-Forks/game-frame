---
name: re-verify
description: Prove that a reimplementation matches the original - bit-exact parity tests against emulator ground truth (bitcompare.py), random-draw order (seqdiff.py), determinism of two identical runs, round trips of real save files, picture comparison against the original game, and comparison between two versions of the reimplementation. Use after every reimplemented slice and before committing it.
---

# Verify against the original

Report only what you ran and looked at: test names with pass and skip counts, files compared,
pictures opened. "It builds" is not verification.

## 1. Bit-exact parity with ground truth

Write the reimplementation's output in the same raw format as the emulator's (`re-emulate`),
then:

```bash
.venv/bin/python tools/verify/bitcompare.py f32 derived/emu/growth/seed-7.f32 out/growth/seed-7.f32
```

It compares by bits (0.0 and -0.0 differ), prints the match count, the first mismatches and the
largest difference in ULP (units in the last place). Reading the result:

| Result | Usual cause |
|---|---|
| All match | Done for these inputs; widen the inputs |
| Off by 1 to a few ULP from some point | Order of operations, a fused multiply-add, a maths library function, double against float |
| Large differences from the first value | Wrong formula, wrong constant, wrong input layout |
| Matches, then diverges for good | A random draw out of order, or state carried between steps |

Turn this into a test in the reimplementation's test suite: it loads the ground-truth files,
**skips with a clear message when `derived/` is missing** (other machines and CI have no game
files), compares exactly, and on failure reports the first mismatching index and the count.

For a DOS game, also compare against the whole original running in DOSBox-X: its exit code
(`scripts/dosbox-run.sh`), its file reads (`--fileio`), or memory read with the debugger at a
breakpoint (`tools/dos/dbx_control.py`). Fix the CPU speed (`--cycles N`) for any comparison
that depends on timing. Recorded demos (a file of inputs the game replays) are the best
end-to-end check: the reimplementation must replay them to the same frames.

### Reach the code a recording never reaches: fuzz from real states

A demo or a recorded session only covers some branches (one game mode, one difficulty, a
few event types). To cover the rest:

1. Capture real states at the start of the fragment, as the whole program runs (for a DOS
   game, memory images of the data and stack segments at a breakpoint).
2. For each case, take one state and change only the inputs the fragment reads: plant a
   random record (an event, a command), set random mode flags. Draw values the code tests
   for (sentinels such as -99, 0, boundaries, the ends of ranges) as often as random ones.
3. Load the changed state into the emulator and run the original fragment (`dos_emu`
   `run_range`). Heap data it reads goes into a new segment, with the pointers in the
   state set to it. Drop cases where the original never finishes, and say how many.
4. Save before and after, and replay them through the same test as the recorded cases.
   Report which inputs the passing cases covered (for example, the event types planted).

This finds what reading misses: loops that start at 0 instead of 1, writes one past an
array, branches only one mode takes.

### Heap addresses and bytes that are not the game's

- **Addresses from the allocator.** Where the original stores heap pointers in variables
  (a loaded bank's selector, copied into each enemy), the port cannot rebuild the
  allocator's history from a snapshot. Give the port a heap whose allocator a test can
  script: before the run, queue the addresses the original returned (read them from the
  `after` state); with nothing queued it picks its own. Every other byte stays comparable.
- **Compare the named variables, list the rest.** The data segment also holds the runtime
  library's own state (its heap and file bookkeeping), padding, and blocks without symbols
  (a driver's data that an interrupt changes mid-frame). Build a mask of the bytes the
  port's variable model covers (store a model twice, over two different fills: the bytes
  that differ are not covered), compare only those, and write down what is left out and
  why. Never mask a byte you have not explained.

## 2. Random draw order

```bash
.venv/bin/python tools/verify/seqdiff.py derived/emu/ai/seed-1.trace out/ai/seed-1.trace
```

Make the reimplementation log each draw as `label<TAB>file:line`. seqdiff compares the labels
(and arguments) and shows the first divergence with the original's call site on one side and
your line on the other: usually it points straight at the bug.

## 3. Determinism

Run the reimplementation twice with the same seed and inputs and compare everything it outputs
(state dumps, saves, frames). Any difference means hidden nondeterminism: wall-clock time, hash
map order, uninitialised memory, threads, or a random generator seeded from the clock. This
check needs no original at all, so run it on every change.

## 4. Real saves

Load saves made by the original game (several sizes and ages) and check every value you decode;
write them back and compare bytes when the reimplementation writes saves. Continuing a real
save for a while in the reimplementation, then comparing with the original continuing the same
save, is the strongest whole-system test you have.

## 5. Pictures

To compare rendering or interface with the original, capture the same view in both (same save,
camera, time, resolution) and measure the difference with an image tool (mean error, the share
of differing pixels, where they are), not by eye. A heat map of differences shows which part is
wrong. If the original only runs on another machine, stream it (Steam Remote Play, Parsec) and
capture there.

## 6. Version against version

When you optimise or restructure the reimplementation, the results must not change: run fixed
scenarios with the old and the new build, compare saves and frames, and also run the old build
against itself to know what "identical" looks like on this machine. Any difference needs an
exact cause before the change goes in.

## 7. The harness itself

`tests/run.sh` proves the harness's tools against the sample game; run it after changing a tool.
