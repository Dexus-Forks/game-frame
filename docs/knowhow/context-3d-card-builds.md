---
id: context-3d-card-builds
title: 'Games with a build per 3D card: Glide, ATI3DCIF and the S3D Toolkit'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x, dos]
runtime: [glide-wrapper, wine, virtual-machine]
hardware: [3d-card, display, mmx]
api: [glide, ati3dcif, s3d, direct3d, directdraw]
tags: [per-card-build, texture-formats, renderer]
commands:
  - '(?i)\b(?:glide2x|glide3x|ati3dcif|s3dtkw)\.dll\b'
tools: [peinfo, match_functions, insns, pe32_emu]
skills: [re-inventory, re-trace, re-observe]
docs: [docs/hardware/card-apis.md, docs/hardware/graphics-3d.md, docs/hardware/simd.md]
seen: ['POD: Planet of Death (1997), 2026-10-23 and 2026-10-24']
---

# Games with a build per 3D card: Glide, ATI3DCIF and the S3D Toolkit

## What to know

Before Direct3D settled, many games shipped one executable per 3D card: a software build, a
3dfx Glide build, an ATI Rage build, an S3 ViRGE build, sometimes an MMX build. The builds share
most of their code. Pick one reference build and match the others to it. Then trace only what
differs. Read the imports first. They name the API. They also show how much each build hands
to the card.

- **Find the API.** Run `peinfo GAME.EXE --imports`. `glide.dll` or `glide2x.dll` is Glide 2.x,
  `glide3x.dll` is Glide 3, `ati3dcif.dll` is ATI3DCIF, `S3DTKW.dll` is the S3D Toolkit. Some
  builds load the DLL with `LoadLibrary`. Search the strings for those builds
  ([docs/hardware/card-apis.md](../hardware/card-apis.md)).
- **Clues in the imports.** A Glide build that does not import `ddraw.dll` draws everything
  through the Voodoo. An ATI3DCIF build without `ATI3DCIF_TextureReg` never gives the chip a
  texture.
- **Match the builds.** Run `match_functions.py` on two builds to carry labels from the
  reference build to the others. Then list which data files each build reads. Another build may
  read a field that the reference build skips.
- **Pick the reference.** The software build is usually the one to copy exactly. The card
  builds show what was drawn differently. Record which build each finding comes from.
- **Measure what the card gets.** A build named after a card does not always hand it the
  scene. POD's ATI Rage and S3 ViRGE builds draw the textured track and cars with the game's
  software renderer, into DirectDraw surfaces at 640 × 480, and give the chip only untextured
  faces: shadows and light beams (*measured*: in one race the S3 build made 15,000
  `S3DTK_TriangleSet` calls, all Gouraud, and never made a texture surface). Redrawing such a
  build's card calls at a higher resolution sharpens only what the card drew. Count the card
  calls by kind in one real scene, and check whether texture surfaces are ever made, before
  choosing between scaling the finished frame and redrawing it.
- **Running a card build today.** Glide needs a wrapper that translates it to a modern API
  (dgVoodoo 2 or nGlide on Windows), or a virtual machine with an emulated Voodoo card
  (86Box). Wine has none of the card libraries, so a build that imports one does not start
  ([missing card library](wine-missing-card-library.md)). A replacement written for the game
  is the other way ([a Glide 2 replacement](context-glide2-wrapper.md),
  [S3D Toolkit return values](s3dtk-error-setstate.md), [ViRGE pixels](s3-virge-15-bit-surfaces.md)).
  Start with the software build.
- **MMX builds.** [docs/hardware/simd.md](../hardware/simd.md) covers a separate MMX
  executable and a path chosen by CPUID.
