---
id: frames-as-arrays
title: 'Porting routines that build records in their stack frame: keep the frame as an array'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [arrays, frames]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Porting routines that build records in their stack frame: keep the frame as an array

## What to know

*2026-10-24, `crates/pilots/src/actions`, Hardwar's `pilots_actions.py`, `pilots_step.py`.*

The pilots' 37 action routines build each plan they insert in a buffer on their own stack,
set only some of its six words, and copy it to `PilPlanInsert`. The other words are whatever
the frame held: an earlier plan built in the same buffer, or what a callee wrote there through
a pointer (`0x485a50` writes four words of a target's position for status 1 and 2, but only
one word, a 0, otherwise). Five practices made 74,000 emulated runs match word for word.

**Index the frame like the disassembly.** The port keeps a `[u32; 24]` per routine, indexed by
`esp` offset / 4, and every write the original makes to `[esp+N]` is a write to `f[N/4]`.
Callees that write through pointers into the frame get a `&mut` slice of it, so a callee that
writes one word leaves the other three as they were. The port then reads like the listing, and
a plan built at `[esp]` with words copied from `[esp+0x1c]` is `f.copy_within(7..10, 1)`.

**Zero the stack where the port's frame starts.** The frame's contents on entry are a
deviation (0 in the port). The harness zeroes the stack before each case, and, when a routine
runs inside another (the crater route inside Go Crater, `PilWait` inside anything, the
routines inside a whole step), zeroes below ESP on the routine's entry with a code hook that
does not stop the routine. Without it the crater route picked up `PilPlanInsert`'s old frame.

**Record what a callee reads through a pointer.** A position passed as `edx = pilot+0xe0` is
three words the port passes by value; the stub records them and the test compares them, so
which words the port passes is checked, not just that the call happened.

**A fatal error ends the run.** The original's `0x49760c` never returns; the port's trait
does. In a single routine the port returns at once (a `stopped` flag across a nested call); in
a whole step the test's `fatal` unwinds (`resume_unwind`, no panic message) out of
`gen_controls`, and the records are compared where both stopped. Seven of the first 300 steps
differed until it did.

**Run a case of a 16 KB routine from its dispatch.** The command processor's cases read the
pilot's place from its frame (`esp+0x78`, `+0x7c`, `+0x84`) set by the per-pilot preamble.
`run_range(0x45fd15, 0x46007a, ebp=pilot)` with those frame words written at the run's ESP
covers the dispatch and every case, without the network and inbox code before it.

Also: a log filter that hides `push reg` lines to shorten a listing hides the stack argument
of `bg_log_collect` (`ret 4`). Check each callee's `ret N` (`isolate.ret_pops`) before trusting
a filtered listing.
