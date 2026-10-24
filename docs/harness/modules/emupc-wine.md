---
id: emupc-wine
title: An emulated x86-64 PC with Debian and Wine, for Windows games
kind: runtime
runtime: [emupc-wine]
provides: [cpu:x86, cpu:x64, cpu:mmx, cpu:sse, os:windows-9x, os:windows-xp, os:windows-vista-or-later, api:directdraw, api:direct3d-9, api:opengl, api:directsound, api:directinput, api:xinput, lib:vc-runtime, media:cd, media:cd-check]
requires: []
vars: [VM_DIR, DEBIAN_IMAGE, SSH_KEY]
seen: ['POD: Planet of Death (1997), 2026-10-23', 'Banished (2014), 64-bit Direct3D 9, 2026-10-24']
---

# An emulated x86-64 PC with Debian and Wine, for Windows games

## What it does

`tools/live/emupc.py` writes a QEMU VM: a Debian cloud image, Wine with 32-bit libraries,
Xvfb with a window manager, and the game's disc as a real CD-ROM drive. The game runs as Wine
on a PC would. Use it when Wine on the host fails, as it does under Rosetta on Apple silicon
([context-emulated-pc](../../knowhow/context-emulated-pc.md)).

## Set up

```bash
./gameframe run emupc image                           # where to download the Debian cloud image
./gameframe run emupc init "$VM_DIR" --image "$DEBIAN_IMAGE" --key "$SSH_KEY"
"$VM_DIR/start.sh"
```

For a 64-bit game, add the 64-bit Mesa libraries before the first boot: put `libgl1
libglx-mesa0 libgl1-mesa-dri` beside `libgl1-mesa-dri:i386` in `$VM_DIR/cloud-init/user-data`.

## Check

`ssh -F "$VM_DIR/ssh_config" gf-vm-PROJECT test -f /var/lib/gameframe-ready` succeeds once the
first boot has installed everything.

## Pitfalls

- The guest kernel panics with `int3` in `finish_task_switch`: QEMU's software CPUs met the
  kernel patching itself. Restart the VM; use fewer CPUs if it comes back.
- No keys reach the game: Xvfb needs a window manager running
  ([xvfb-needs-window-manager](../../knowhow/xvfb-needs-window-manager.md)).
- `media:cd-check` is met only when the disc is attached with `--disc`, because Wine on Linux
  reads the drive type and label from the device.
