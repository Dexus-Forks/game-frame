---
id: renderer-on-a-snapshot
title: 'Running the original''s renderer on a snapshot of the game'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
tags: [renderer, snapshot]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Running the original's renderer on a snapshot of the game

## What to know

*2026-10-24, Hardwar's `snapemu.py`, Hardwar's `render_truth.py`.*

A software renderer needs the whole loaded game to draw anything: the world's objects, the
shapes, the textures, the remap tables, the back end's tables. Building that in an emulator by
hand is not practical. A whole-process snapshot (Hardwar's `vm_snapshot.py`) already holds it
all. Mapped back into Unicorn, the game's own `flight_display` and `scene_draw` draw a frame
exactly as the game would, from the snapshot's state, and nothing but the window's sizes needs
setting. That gives an exact 8-bit reference frame (and every polygon Draw3D rasterises) for a
state the port can also load (the snapshot was taken after loading save 3).

How:

- Map every region of the snapshot at its address; the image's code is in it too (Wine maps
  the sections writable). Point the import table at stubs; stop on any import without one.
- **Give Unicorn a GDT.** Watcom code saves and restores GS (`push gs` ... `pop gs`, 0x450888).
  With no descriptor table, Unicorn's `pop gs` left the stack segment 16-bit and the next push
  wrote at `ESP & 0xffff` ("invalid access at 0xdf94"). One flat data descriptor loaded into
  DS, ES, SS, FS and GS fixes it.
- Set the x87 control word again (0x0c7f): a snapshot holds memory, not registers.
- Enter below the DirectX calls: `flight_draw` touches DirectInput and DirectDraw objects whose
  code is not in the snapshot. Build the packet with `flight_display` (0x4962cc, EAX the pilot
  from `player_pilot` 0x447d48) and call `scene_draw` (0x48cfac) with it.
- For an 8-bit frame: `0x55468c` = 0, the row table `0x55469c` over a buffer, base `0x554690`,
  pitch `0x554694`, height `0x554698`, screen size `0x5999dc`/`0x5999e0`/`0x599a28`/`0x599a2c`,
  window `0x59a160`..`0x59a16c`. A paused console leaves 640 × 400 in several of these; miss
  one and the frame stops at row 400.
- The clipper loops for ever on a polygon with no inside vertex; its callers drop those first
  (the AND of the outcodes). A random-input harness must do the same.
- A polygon's `+0x20` is a resource handle, not a material: map handles to sprites by matching
  the texture's pixels (`res_lock` 0x499acc) against the pack's `.SPR` files.

Candidate for GameFrame (a snapshot mode for pe32_emu, and the GDT pitfall).
