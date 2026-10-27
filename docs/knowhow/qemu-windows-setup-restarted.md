---
id: qemu-windows-setup-restarted
title: 'Windows Setup in QEMU says "The computer restarted unexpectedly" and will not continue'
kind: failure
status: solved
platform: [windows-nt, macos]
runtime: [virtual-machine, apple-silicon]
symptom: [wont-start]
tags: [qemu, tcg, windows-10, setup, install]
signatures:
  - 'computer restarted unexpectedly or encountered an\s+unexpected error'
tools: []
skills: [re-observe, re-setup]
docs: [docs/knowhow/context-windows-machines-apple-silicon.md]
seen: ['Banished (2014), Windows 10 22H2 x64 in QEMU 10 TCG, 2026-10-27']
---

# Windows Setup in QEMU says "The computer restarted unexpectedly" and will not continue

## Symptoms

The machine boots from its disk, shows the Windows logo, then a box: "The computer restarted
unexpectedly or encountered an unexpected error. Windows installation cannot proceed." OK
restarts into the same box.

## Cause

The machine was stopped while Setup was still installing, between its restarts (*inferred*: the
disk was last written at the time the machine was stopped, and the 32-bit twin, left running,
went on to the first-run screens). Under QEMU's TCG, Setup takes over an hour, so it is easy to
stop it early by mistake: quitting QEMU, `kill`, or a script that shuts the machine down.

## What to do

Start again with an empty disk. The half-installed disk cannot be repaired.

```
rm base.qcow2
qemu-img create -f qcow2 base.qcow2 64G
# boot the ISO again and redo Setup
```

Then leave the machine alone until the desktop or the first-run screens appear. Check its
screen instead of guessing: `screendump FILE.ppm` in the QEMU monitor. Once Windows is at its
first-run screens, a `savevm` snapshot keeps that point.
