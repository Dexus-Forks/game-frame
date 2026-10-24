---
id: recorder-hook-in-section-slack
title: 'A per-step recorder fits in the code section''s mapped slack'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [hook, recorder, section, slack]
tools: [probegen]
skills: [re-emulate, re-observe, re-format]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A per-step recorder fits in the code section's mapped slack

## What to know

*2026-10-24, Hardwar's `patch_recorder.py`.*

To record every flight step (the step's `dt_ms`, the world clock, the seeds, the input
state) a sampler from outside is not enough: it misses most frames and is not atomic
(GameFrame `docs/observe/recording.md`, "What a recording from outside cannot see"). A hook
inside the step is.

Where to put it: Watcom's linker left `BEGTEXT` with a virtual size of 0 and a raw size of
`0x116e00`, so the loader maps it to the next section alignment (`0x10000`), up to `0x530000`.
Only 0x83 bytes at the end of the raw data are free, but `0x526e00`-`0x530000` is mapped and
executable. Growing the section's raw size by `0x1000` and moving the later sections down in
the file (their addresses stay) puts a 4 KB hook there without moving any code. The section
was made writable too, so the hook keeps its file handle and buffer beside its code.

The hook replaces one call (`call controls_read` at `0x43ac72` in `flight_frame`), calls it,
and then writes with the game's own imports (`CreateFileA`, `WriteFile`, `Sleep`). It saves
every register and the flags. The step it records is the one the game used; the time the
write takes is counted by the game's timer in the next step and recorded there.

Pitfall: `pkill -f NAME` run over SSH kills the SSH command itself, since its command line
contains NAME (GameFrame knowhow `vm-ssh-pkill-matches-itself`, seen again here). Stop the
game with its own prefix's `wineserver -k` (Hardwar's `vm_run.sh` stop`).

For GameFrame: the technique (grow a Watcom section into its mapped slack, hook one call,
write with the game's own imports, hold the game between steps for a whole-process
snapshot) would serve any 32-bit Windows game; `probegen` could gain a sibling that writes
such hooks.
