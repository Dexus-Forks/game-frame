---
id: wait-for-the-next-race
title: Start a recorder when the next race begins instead of driving the menus
kind: context
status: context
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
genre: [racing-sports]
tags: [recording, trigger, phase, countdown, shared-emulator, automation]
tools: [memrec]
skills: [re-observe]
docs: [docs/knowhow/record-a-run-from-memory.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Start a recorder when the next race begins instead of driving the menus

## What to know

Driving a game's menus from a script is slow and fragile, and in a shared emulator it can
collide with another session. When the game has a state variable that marks the start of a
run, a small watcher can wait for it and start the recorder at that moment, whoever starts
the run.

POD's race phase `DAT_005920a4` is 2 during the countdown. `run/vm/racewait.sh` polls it
twice a second through `/proc/PID/mem` (finding the PID again each time, since the game may
restart) and then runs the recorder. It caught two races from their countdown, one started
by another session. GameFrame's recorder does the same:
`memrec.py record SPEC --name GAME.EXE --wait 0x5920a4=2`.

Limits:
- Polling misses the first moments. At 0.5 s on a slow VM, POD's recordings start at
  countdown "5" or "4". Poll faster, or trigger on an earlier state, if the start matters.
- Under `sudo`, write the output to a new file: `/tmp` files owned by another user cannot be
  written even by root when `fs.protected_regular` is set (Debian's default).
