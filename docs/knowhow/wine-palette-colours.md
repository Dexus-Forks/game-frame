---
id: wine-palette-colours
title: 'Wine shows each palette index in a colour of its own'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [colours, palette, wine]
tools: []
skills: [re-emulate, re-observe, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# Wine shows each palette index in a colour of its own

## What to know

*2026-10-24, `crates/hud/tests/screenshot.rs`, `derived/truth/run3/shots/run3-out.png`.*

The recorded frames come from the original under Wine's DirectDraw in an emulated PC. Their
colours are not the palette's: index `0x34`, which `HARDWAR.PAL` sets to (248, 136, 36) (the
6-bit values shifted left by 2), is (255, 138, 33) in the shot (*measured*). The mapping is
the same for every pixel of an index, but it is not a simple rescaling of the 6-bit values.

So compare a frame by position, not by RGB: learn each index's colour in the shot from the
pixels the port draws in that index, then require every such pixel to have it, and no other
pixel of the compared region to have it. The window's client area is at (24, 50) in the
800 x 600 shots (border and title bar). Whether a Windows machine shows the palette's colours
is *unknown*.
