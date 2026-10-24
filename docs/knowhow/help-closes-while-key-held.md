---
id: help-closes-while-key-held
title: A screen that closes "on a key" closes at once when the key is still held
kind: failure
status: solved
era: [late-1990s]
genre: [racing-sports]
symptom: [wrong-picture]
tags: [input, menus, help, held-vs-pressed, reference-frames, port]
tools: []
skills: [re-trace, re-port]
docs: [docs/observe/inputs.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A screen that closes "on a key" closes at once when the key is still held

## Symptoms

Captures of the same action disagree: after F1 some show the help screen, others the menu as
if nothing happened.

## Cause

POD's help loop draws the picture until `FUN_004a2bf0` finds a menu key *held*, and F1 is one
of them. If F1 is still down when the loop first checks, the help closes at once; only a quick
tap shows it. Then `FUN_004a2e04` waits until every key is let go.

## What to do

Read whether a wait tests "held" or "newly pressed" before porting it, and port that test. In
frame tests, accept both outcomes where the original's depends on how long a key was held,
and continue the sequence from the one the capture shows.
