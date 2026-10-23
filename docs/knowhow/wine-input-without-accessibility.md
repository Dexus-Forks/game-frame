---
id: wine-input-without-accessibility
title: Clicks and keys from the agent do not reach a Wine window on macOS
kind: failure
status: workaround
platform: [macos]
runtime: [wine]
api: [win32, directinput]
symptom: [no-input]
tags: [sendinput, accessibility, computer-use, automation, scancodes]
tools: []
skills: [re-observe]
docs: [docs/observe/inputs.md, docs/knowhow/context-wine-macos.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Clicks and keys from the agent do not reach a Wine window on macOS

## Symptoms

Synthetic clicks and keys from macOS do nothing in the game. A screen-control tool cannot list
the Wine window at all.

## Cause

macOS refuses synthetic events from a process without the Accessibility permission
(`AXIsProcessTrusted()` is false). Wine windows belong to no application bundle, so tools that
work by application cannot see them (*observed*).

## What to do

Send the input from inside Wine. A small Windows program, built for 32-bit Windows with
`zig cc -target x86-windows-gnu`, finds the game's window, brings it forward and calls
`SendInput`. Send keys as scancodes (`KEYEVENTF_SCANCODE`), which DirectInput reads. Run it with
the same Wine and prefix as the game. Then no step needs macOS's permission.

Clicks work on Windows dialogs. A game that moves its own cursor from DirectInput's relative
mouse movement ignores a warped cursor. Relative moves through `SendInput` may work in such a
game. This is not tried yet. GameFrame has no such tool yet (GAPS.md).
