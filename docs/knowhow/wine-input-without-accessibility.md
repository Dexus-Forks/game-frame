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
tools: [winput]
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

Send the input from inside Wine with GameFrame's `winput`. It is a small Windows program that
runs in the same Wine prefix as the game. It finds the game's window, brings it forward and
calls `SendInput`. It sends keys as scancodes, which DirectInput reads. No step needs macOS's
permission. Run `./gameframe run winput run --prefix derived/wine window "GAME" key enter`.

Clicks work on Windows dialogs. A game that draws its own cursor from DirectInput follows a
warped cursor, but may read the press before its cursor arrives: move, wait, then press
(`winput ... point X Y wait 1000 button`; [click timing](directinput-cursor-click-timing.md)).
`./gameframe tool winput` lists every step.
