---
id: vm-ssh-pkill-matches-itself
title: '`pkill -f` or `pgrep -f` run over SSH matches its own command line'
kind: failure
status: solved
era: []
platform: [linux]
runtime: [virtual-machine]
hardware: []
api: []
genre: []
symptom: [tool-error]
tags: [ssh, pkill, pgrep, shared-vm]
signatures: []
tools: []
skills: [re-observe]
docs: [docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# `pkill -f` or `pgrep -f` run over SSH matches its own command line

## Symptoms

`ssh host 'pkill -f follow.py; restart-the-game'` ends at once and the rest never runs. Or
`ssh host 'pgrep -f "python3 follow.py" && echo running'` says "running" after the script
ended.

## Cause

SSH runs the command through `bash -c 'the whole line'`. That shell's own command line holds
the pattern, so `-f` matches it: `pkill` kills the shell running the line, `pgrep` finds it
(*observed*, 2026-10-23).

## What to do

Make the pattern not match itself, for example `pkill -f "[f]ollow.py"`, or put the command in
a script file in the VM and call the script. In a VM several sessions share, kill only your own
processes by name and display (`pkill -f "Xvfb :1"`, not `pkill Xvfb`): a bare `pkill Xvfb`
in POD's `inside.sh stop` also stopped another session's display on :2.
