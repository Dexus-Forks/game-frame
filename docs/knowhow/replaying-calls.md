---
id: replaying-calls
title: 'Verifying a decision routine by replaying its calls'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [calls, replaying]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Verifying a decision routine by replaying its calls

## What to know

*2026-10-24, Hardwar's `pilots_gen.py`, `crates/pilots/tests/decide.rs`.*

`PilGenControls` (0x466c64) is 2,600 instructions of decisions about one pilot after another,
and nearly every decision asks another system something: the vehicle, the trade board, the
clock, the garrison. Building a real world for it in the emulator would mean porting half the
game first. Three practices made it testable bit for bit without that.

**Stub every callee and record the call, not just the answer.** Each routine the step calls
is a stub that draws a random answer, writes random results through its pointer arguments,
and appends a record: the address, EAX, EDX, EBX, ECX, the stack arguments, the answer and the
results. The port calls the same routines through a trait, one method per original routine;
the test's implementation of the trait takes the next record, checks that the address and the
arguments that carry values (not frame pointers) are the port's, and hands back the
original's answer. A wrong branch shows up as the first call that differs, with its index. It
also makes the order of calls, and so the order of every random draw made inside them, part
of what is verified.

**Keep pointers as pointers.** The earlier pilot tests wrote pointers as small indices. Here
a plan's word 1 is sometimes a 16-bit good written over a building pointer left in the
buffer, so its high half is half a pointer; an index encoding turns that into a word the port
cannot reproduce. Writing the emulator's addresses as they are, and giving the port the same
handles, fixed it.

**Model the frame when the routine reuses it.** The routine builds plans in a handful of
stack buffers and sets only some words of each; the rest is whatever the previous pilot left.
The port keeps those buffers (`decide::Frame`) and the emulator zeroes the stack before each
case, so only the frame's state before the first pilot is a deviation.

A pitfall on the way: filtering a long listing to read it faster (dropping `inc eax`, `mov
edx, ebp` and the like as loop noise) also dropped a real `inc eax`, and the port first left
out a day carry that the original has. The emulator found it on the 11th case; read the
unfiltered listing for anything the port depends on.
