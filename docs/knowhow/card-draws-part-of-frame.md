---
id: card-draws-part-of-frame
title: 'A 3D-card build gives the card only part of the frame: measure before upscaling'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [glide-wrapper, wine]
hardware: [3d-card, display]
api: [ati3dcif, s3d, directdraw]
tags: [per-card-build, upscaling, software-renderer]
tools: [peinfo]
skills: [re-trace, re-observe]
docs: [docs/hardware/card-apis.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A 3D-card build gives the card only part of the frame: measure before upscaling

## What to know

A build named after a 3D card does not always hand its scene to the card. POD's ATI Rage and
S3 ViRGE builds draw into DirectDraw surfaces at 640 × 480 and give the chip only untextured
faces: in a race on Beltane the S3 build made 15,000 `S3DTK_TriangleSet` calls, every one
Gouraud, and never created a texture surface; what both chips drew was the cars' shadows and
light beams (*measured*, by counting calls by rendering type in a stand-in library). The
textured track and cars were the game's own software renderer, as in the software build.

So redrawing such a build's card calls at a higher resolution sharpens only what the card
drew. Before choosing between scaling the finished frame and redrawing at a higher
resolution, count the card calls by kind in one real scene (a counter in the probe library is
enough), and check whether texture surfaces are ever made. Imports hint at it too: an ATI3DCIF
build that does not import `ATI3DCIF_TextureReg` never gives the chip a texture. In POD only the
Glide build gives the card everything, so only its scene draws sharper; the other two get a
good frame scaler (`wrapper/scalers`).
