---
id: ssh-too-many-auth-failures
title: 'SSH to a VM fails with "Too many authentication failures"'
kind: failure
status: solved
runtime: [virtual-machine]
symptom: [tool-error]
tags: [ssh, ssh-agent, identitiesonly]
signatures:
  - 'Too many authentication failures'
tools: [emupc]
skills: [re-observe]
docs: [docs/knowhow/context-emulated-pc.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# SSH to a VM fails with "Too many authentication failures"

## Symptoms

`ssh` to a new VM stops with "Received disconnect ... Too many authentication failures".

## Cause

The SSH agent offers each of its keys before the one the VM knows. The server allows a few
tries (six by default) and closes the connection (*observed*).

## What to do

Name the key and offer only it: `ssh -o IdentitiesOnly=yes -i KEY host`, or put
`IdentitiesOnly yes` in the host's entry of an SSH config. `emupc.py init` writes such a config.
