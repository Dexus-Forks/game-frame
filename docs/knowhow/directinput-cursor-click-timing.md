---
id: directinput-cursor-click-timing
title: Scripted menu clicks land one time in three in a game with its own cursor
kind: failure
status: workaround
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: []
api: [directinput]
genre: []
symptom: [no-input]
tags: [menu, cursor, click, sendinput, automation, timing]
signatures: []
tools: [winput]
skills: [re-observe]
docs: [docs/observe/inputs.md, docs/knowhow/wine-input-without-accessibility.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Scripted menu clicks land one time in three in a game with its own cursor

## Symptoms

The same scripted menu sequence picks the wanted item once and another item (or nothing) the
next times. Screenshots show the game's cursor still on its way, or a button highlighted but not
pressed.

## Cause

The game draws its own cursor from DirectInput, polled once a frame. `podinput click:X,Y` warps
the system cursor (`SetCursorPos`) and presses 50 ms later; the game's cursor follows the warp
(*observed*: it ends on the point), but the press can be read before the game has moved its
cursor there. A fixed wait after a screen change can also be shorter than the screen's load
(*observed* in POD's menus under Wine, 2026-10-24).

## What to do

Move, wait about a second, then press, and take a screenshot before each step that matters
instead of chaining waits. With GameFrame's `winput`: `point X Y wait 1000 button`. A warped
cursor does reach a DirectInput game under Wine, as long as the press waits for it. At a low
frame rate, hold keys longer too (`winput --hold 500`), so a poll sees each press.
