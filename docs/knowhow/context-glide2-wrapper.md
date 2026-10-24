---
id: context-glide2-wrapper
title: 'Writing a Glide 2 replacement for an early Glide game'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [glide-wrapper, wine, virtual-machine]
hardware: [3d-card, display]
api: [glide]
tags: [glide-wrapper, upscaling, lfb, record-replay]
commands:
  - '(?i)\bglide(?:2x)?\.dll\b'
tools: [peinfo, refs]
skills: [re-trace, re-observe]
docs: [docs/hardware/card-apis.md, docs/knowhow/context-3d-card-builds.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Writing a Glide 2 replacement for an early Glide game

## What to know

A game that draws everything through Glide can be drawn at any resolution by replacing the
library: the game passes screen coordinates, colours and texture memory, and nothing else
depends on the Voodoo's 640 × 480. POD's 3dfx build runs unchanged through `wrapper/glide`
(Rust, built for 32-bit Windows, OpenGL). What mattered, each *observed* or *measured* there:

- **The file and the names.** Early Glide 2 games import `glide.dll`, not `glide2x.dll`, by
  decorated `__stdcall` names (`_grGlideInit@0`, `_grSstOpen@24`). Export exactly those: list
  them with `peinfo --imports` and `strings`, and write them into the `.def` file.
- **No window.** The early `grSstOpen(res, refresh, format, origin, smoothing, buffers)` takes
  no window handle: a Voodoo drew full screen. Find the game's own window (by class, even
  while hidden: POD calls Glide before it shows the window) and draw into it.
- **List the calls with their arguments first.** A capstone scan of every call site, reading
  the constants pushed before it, gives the game's whole use of the card in one table (POD: 47
  calls, 360 sites): which combiner settings, blend factors, depth mode, fog, texture formats
  and LFB modes need to be right, and which never occur.
- **More than one thread.** Games call Glide from a loading thread too. An OpenGL context
  belongs to one thread: record every call as a command and replay on one render thread,
  answering only queries (`grSstScreenWidth`, `grTexCalcMemRequired`, `grLfbGetReadPtr`) at
  once.
- **The LFB.** The write pointer has a fixed stride of 1024 pixels (2048 bytes at 16 bits, 4096
  at 32), and a game may set the write mode after taking the pointer. Hand out a buffer
  filled with a marker and read what changed at `grLfbEnd`. Reads of the depth buffer exist
  (POD's lens flare tests for `0xffff`): copy the depth down to the game's resolution.
- **Depth.** Map the W buffer to `1 - oow` and the Z buffer to `ooz / 65535`, so Glide's
  compare functions are OpenGL's as they are; turn on depth clamping.
- **Texture coordinates.** `sow`/`tow` run 0–256 across the longer side, divided by w;
  pass `s·w` as a perspective-correct varying and the colours `noperspective`, as Glide does.
- **Gamma** belongs in the final copy to the window (the Voodoo's output table), not in the
  blending.
- **Software OpenGL under an emulator is slow.** Mesa's llvmpipe in a QEMU-emulated PC drew
  the menus at 2–5 frames a second; `LP_NATIVE_VECTOR_WIDTH=128` roughly doubled it.
  Log frames per second and where the time goes from the start.
