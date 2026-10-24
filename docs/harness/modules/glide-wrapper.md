---
id: glide-wrapper
title: A Glide wrapper, for games built for 3dfx Voodoo cards
kind: layer
runtime: [wine-host, emupc-wine]
provides: [api:glide, api:glide2, api:glide3]
requires: []
vars: []
seen: []
---

# A Glide wrapper, for games built for 3dfx Voodoo cards

## What it does

A game's 3dfx build imports `glide.dll`, `glide2x.dll` or `glide3x.dll`. No card has those
today. A wrapper is a replacement DLL with the same exports that draws through Direct3D or
OpenGL. GameFrame's guides cover the choices and how to write one of your own.

## Set up

```bash
# Follow docs/knowhow/context-glide2-wrapper.md and docs/hardware/replacement-libraries.md:
# put the wrapper's glide2x.dll (or glide3x.dll) beside the game's executable.
```

## Check

The game's 3dfx build starts and draws. The wrapper's own log, where it has one, lists the
Glide calls it served.

## Pitfalls

- Read [context-glide2-wrapper](../../knowhow/context-glide2-wrapper.md) and
  [context-3d-card-builds](../../knowhow/context-3d-card-builds.md) first.
- The steps are not yet written as commands. Write them here when a game proves them.
