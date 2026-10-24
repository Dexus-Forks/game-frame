---
id: context-powervr-sgl-wrapper
title: 'Writing a PowerVR SGL replacement for a PCX1/PCX2 game'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [3d-card, display]
api: [sgl, directdraw]
tags: [powervr, sgl, scene-graph, upscaling, address-callback, record-replay]
commands:
  - '(?i)\bsgl\.dll\b'
  - '(?i)\bsglhw\.ini\b'
tools: [peinfo, insns]
skills: [re-trace, re-observe]
docs: [docs/hardware/card-apis.md, docs/hardware/replacement-libraries.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Writing a PowerVR SGL replacement for a PCX1/PCX2 game

## What to know

PowerVR's first chips (PCX1, PCX2) were driven through VideoLogic's SGL. Unlike Glide, SGL
is a retained scene graph: the game builds display lists of transforms, materials, meshes,
lights and cameras, edits a few transforms each frame and calls `sgl_render`. A stand-in keeps
the graph and draws it with any API at any size. POD's PowerVR build runs through
GameFrame's `wrappers/sgl` (Rust, 32-bit Windows, OpenGL). What mattered, each *observed* or *measured*
there:

- **Look for the SDK on the game's disc.** Games shipped the PowerVR driver package, with
  `sgl.h` and `sglwin32.h` in it (POD: `POWERVR/310C3.ZIP`). The headers give every
  signature (all `__cdecl`, undecorated names) and the error codes; nothing needs fetching.
- **`sglhw.ini`.** The driver's installer writes it into the Windows folder. A game may read
  values from it and divide by them (POD reads the region size, 0 without the file, and
  divides). The stand-in writes the installer's values when the file is missing.
- **The address callback.** With `sgl_use_address_mode` the game names a function SGL calls
  for the memory to render each frame into. The game's callback also finishes the frame
  before it: draws its 2D overlay (the dashboard), shows it, and locks the next back buffer.
  Call it at the first `sgl_render` of a frame, fill the viewport in that memory with a marker,
  draw the scene with OpenGL, and put every non-marker pixel of the picture the game then
  shows on top. That needs the picture: a small hook in the DirectDraw stand-in passes it on.
  The callback's parameter block is packed (pointer, 16-bit stride, 8-bit depth).
- **Materials inside meshes.** Material calls made right after `sgl_create_mesh` belong to the
  mesh's faces; made elsewhere they are entries of the current list. Lists made with
  `preserve_state` give the transform, material and lights back when they end.
- **Names.** Calls with `generate_name` false return 0 (`sgl_no_err`), not a name.
- **The immediate mode.** SGL's other half (`sgltri_*`) takes faces already projected to the
  screen, in Direct3D's `D3DTLVERTEX` layout (`SGLVERTEX`), under an `SGLCONTEXT` of 0x90
  bytes. Hardwar (1998) uses it. Draw the faces through the same pipeline. A vertex at screen
  `(x, y)` with `1/w` becomes the camera-space point `(x w, -y w, w)` under a camera that maps
  it straight back, with the nearest `w` as the near plane. PowerVR's own SGLMID4.dll answers
  what the header leaves open. A face clockwise on the screen faces the camera. Faces of less
  than 1/32 pixel are dropped. Flat shading takes the first vertex's colour. The fog density
  (0 to 31) is a power-of-two register on a depth of 0.01 / w. Translucency is one 4-bit
  level per face. The driver sorts translucent passes, one per call and facing, farthest
  first; it never sorts single faces. It never reads `SGLTT_VERTEXFOG`.
- **A test without the game.** A small Windows program that makes the same calls on a known
  scene (a textured cube under a 2D bar) checks the projection, lighting, textures and the
  overlay where the game itself cannot run (`wrappers/sgl/test/sgltest.c`, and `sgltritest.c` for
  the immediate mode).

## Still guesses

The projection from the camera rectangle, the fog formula, the default material and how a
light volume adds light are inferred from the headers' names and the game's arguments; each is
marked `Deviation:` in the stand-in until a picture of the original settles it.
