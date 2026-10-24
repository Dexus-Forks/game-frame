---
id: emulating-a-system
title: 'Emulating a whole system routine: stubs, whole records, and stale registers'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [emulating, system]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Emulating a whole system routine: stubs, whole records, and stale registers

## What to know

*2026-10-24, Hardwar's `economy_truth.py`, `crates/economy/tests/emulated.rs`.*

The economy's routines call message posting, searches over the world, sound and the updates
of systems not yet ported. Four practices made them testable bit for bit.

**Stub internal functions at their entry.** `pe32.py` stubs only imports. For a function in
the game's own code, hook its entry address with `UC_HOOK_CODE` and, in the hook, record the
registers (and whatever the pointers in them point at), pop the return address into EIP and
set EAX. Watcom's register convention keeps every register but EAX across a call, so that is
a faithful return. A hook can also skip a routine for one run only (keep a set of addresses
to skip): `bg_set_function` runs the first factory or shop update at its end, and the
configuration test wants the state without it. Hook only the addresses you stub or skip; a
skip set the hook never sees does nothing (the first run of the script walked straight into
the Moth roll-out that way).

**Compare whole records, not chosen fields.** Each case writes the 900-byte building record
and its function memory before and after, and the port reads the record into typed fields
while keeping every other byte (`Building::raw`), then writes it back. A field the port
forgot to write, or wrote at the wrong offset, shows as a byte difference without anyone
having to think of it: the five `0x4000` halfwords at `+0x32c` (not four) were found that way.

**Mind state that carries between cases.** Globals the routine steps keep their value from
case to case unless the script resets them: `bg_create`'s light-group counter `0x582140`
made every configuration after the first differ until the test followed it.

**Following constant arguments through a disassembly needs care.** To read which kind and
subtype `bg_table_build` gives each building, a scan tracked `mov reg, imm` before each call.
EBX survives calls in Watcom code, so a value set for one call (`bg_create`'s EBX = 7) looked
like the next call's subtype; the real subtype came from a global tested a few instructions
earlier. Forget a register on any other write to it, and check odd results by hand.

A small trap on the way: a helper script named `dis.py` shadows the standard library's `dis`,
which Capstone imports through `inspect`, and Capstone then fails with a circular import.
