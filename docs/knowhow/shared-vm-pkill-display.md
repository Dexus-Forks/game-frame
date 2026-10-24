---
id: shared-vm-pkill-display
title: 'A run in a shared test VM dies: "X connection to :2 broken"'
kind: failure
status: solved
era: []
platform: [linux]
runtime: [virtual-machine, wine]
hardware: [display]
symptom: [crash]
tags: [shared-machine, sessions, xvfb]
signatures:
  - 'X connection to :\d+ broken \(explicit kill or server shutdown\)'
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# A run in a shared test VM dies: "X connection to :2 broken"

## Symptoms

A game running on its own Xvfb display in a VM that other sessions also use dies in the
middle of a run; its log ends with "X connection to :2 broken (explicit kill or server
shutdown)", and the other display's Xvfb has a new process number.

## Cause

Another session's stop script ran a bare `pkill Xvfb`, which ends every Xvfb in the machine
(*observed*: POD's `run/vm/inside.sh stop`, twice in one evening). A related trap, `pkill -f`
matching the ssh command that holds it, is
[its own entry](vm-ssh-pkill-matches-itself.md).

## What to do

Scripts on a shared machine stop only their own processes: `pkill -f "Xvfb :1"`, not
`pkill Xvfb`.
For a long run, start the X server under a name of its own (a symlink: `~/bin/Xglide`), which
nobody else's `pkill Xvfb` matches. Give each session its own display and Wine prefix.
