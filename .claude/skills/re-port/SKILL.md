---
name: re-port
description: Reimplement traced game behaviour in new code that behaves exactly like the original - mirroring its structure, numeric types and order of operations, random draw order, iteration order and integer semantics, citing the original function on every item and marking every intentional difference as a Deviation. Use when writing the reimplementation of anything traced (re-trace) and measured (re-emulate).
---

# Reimplement, exactly

Precondition: the behaviour is traced and documented, and anything numeric has ground truth. If
you find yourself inventing a rule while writing code, stop and trace it.

## Write new code, not a translation

Read the pseudocode to understand the behaviour, then write the new code from your notes, with
clear names and the target language's own types and structure. Never paste decompiled code into
the new code base: it is the original author's work, and it is also unreadable. This is the line
between a reimplementation and a copy; keep it visible (see `docs/LEGAL.md`).

## Mirror the original's shape

- One module or type per original class or system, named after it where the name is known
  (from exports, RTTI or strings) and after its role otherwise.
- Keep the data flow: if the original builds a menu from a data file, build it from the same
  data file; do not hand-code what the game ships as data.
- Keep constants with their source address, exactly as read: `0.33333331f`, not `1.0 / 3.0`.
- Every reimplemented item names its original in a comment or doc comment, in one consistent
  form the tools can search, for example ``Original: `FUN_1400be680` ``. Mark guesses
  *inferred*.

## Exactness checklist

These are the details that make a reimplementation drift, in order of how often they bite:

1. **Float precision.** Use the original's types: 32-bit floats stay 32-bit. Languages that
   compute in doubles by default (Python, JavaScript, Lua) need explicit rounding after every
   operation (`Math.fround`, `struct` round trip). 32-bit x87 games computed in 80-bit
   precision until a value was stored: match where it was stored.
2. **Order of operations.** Floating-point addition and multiplication are not associative:
   `(a + b) + c` differs from `a + (b + c)`. Keep the original's order, from the disassembly.
   Watch for fused multiply-add: compilers may fuse `a * b + c` on one side and not the other.
3. **Maths library.** `sin`, `cos`, `atan2`, `pow`, `exp` differ in the last bit between C
   runtimes. If the simulation depends on them, reimplement the original's runtime functions
   (emulate them for ground truth) instead of calling your platform's.
4. **Random draws.** Same generator, same seed, same number of draws, same order. A draw moved
   inside a condition, or skipped when a value is zero, changes everything after it.
5. **Iteration order.** Lists, arrays, hash maps: iterate in the original's order. Never let a
   hash map's order reach anything observable.
6. **Sorting.** Unstable sorts (C `qsort`) order equal elements in a way that depends on the
   algorithm; reproduce the original's algorithm when equal keys are possible.
7. **Integers.** Width, signedness, wrap-around on overflow, division rounding toward zero,
   arithmetic against logical shifts, float-to-int truncation (`cvttss2si`).
8. **Time.** Fixed step against variable step; integer ticks against float seconds; the exact
   step size and when the accumulated time is consumed.
9. **Uninitialised and stale state.** The original may read a field before writing it, or keep
   a value from the last frame. That is behaviour too.

## Deviations

Any intentional difference (a platform difference, a fix the project decided on, an unported
piece) gets a comment containing `Deviation:` and the reason, so a script can list them all.
A deviation is a decision; an accidental difference is a bug.

## Workflow

1. List the functions to reimplement, from the topic document.
2. Do the smallest end-to-end slice first, and test it against ground truth before the rest.
3. Build and test often; commit each verified slice (`re-land`).

`examples/sample-game/port.py` is a complete small example: a reimplementation that matches the
original bit for bit, and a `--naive` mode that makes mistake 1 to show what the checks catch.
