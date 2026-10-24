---
id: running-the-original-on-a-snapshot
title: 'Run the original''s routines on a whole-process snapshot'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [original, running, snapshot, the]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Run the original's routines on a whole-process snapshot

## What to know

*2026-10-24, Hardwar's `snap_emu.py`, the HUD (`docs/interface.md`).*

The HUD draw reads a packet the game builds from the whole world (the Moth, its slot, the
fonts, the remap tables, the 3D camera). Building that state by hand for an isolated routine
would take longer than the port. Hardwar's `vm_snapshot.py` already copies every writable
mapping of the running game between two steps; `SnapEmu` maps all of them into Unicorn at
their addresses and calls any routine of the game on that world:

```python
e = SnapEmu('derived/truth/run4/frame-1004.snap')
e.handle('timeGetTime', lambda emu: 1000)   # Wine's DLLs are not in the snapshot
e.skip(0x4b7da8)                            # the renderer's flush, which only reaches DirectDraw
pilot = e.call(0x447d48, eax=0)['eax']      # player_pilot(0)
e.call(0x4962cc, eax=pilot, edx=0)          # the display packet, as the game builds it
fb = e.alloc(640 * 480); e.call(0x49b87b, eax=fb, ebx=640, ecx=480)   # our own screen
e.call(0x427270, eax=e.u32(0x5872f8))       # the scene, the 3D view included
e.call(0x48cda0)                            # the HUD
```

It ran the software renderer's 3D view of the snapshot's world as well (an image for the
renderer's work), in about 4 seconds.

Three things needed care:

- **Imports.** Every import slot is pointed at a stub of ours; an import without a handler
  stops the run with its name and caller. The HUD needed only `timeGetTime`.
- **`push gs` / `pop gs`.** Some of Watcom's functions save GS. With no descriptor table in
  Unicorn, `pop gs` broke the stack (the next `call` wrote to a 16-bit address). The rig
  executes those instructions by hand: four bytes pushed or dropped.
- **The world in the snapshot is the save's.** In save 3 the Moth was already exploding, so
  the game built no HUD packet (the death camera). The rig sets the Moth's structure damage
  and dying timer to 0 first; the port's check does the same to its world.

A memory-read hook over the whole call (`UC_HOOK_MEM_READ`) then lists exactly what a routine
reads, which defined the port's `Inputs`.
