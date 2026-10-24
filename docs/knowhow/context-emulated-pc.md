---
id: context-emulated-pc
title: 'Running the original in an emulated PC when Wine on the host fails'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x, linux, macos]
runtime: [virtual-machine, wine, apple-silicon]
api: [directinput, directdraw]
symptom: [crash, hang, no-input]
tags: [qemu, cloud-init, xvfb, openbox, ssh, rsync, decision]
commands:
  - '(?:^|[;&|(]\s*|/)qemu-system-x86_64(?=\s|$|")'
tools: [emupc, discinfo, memrec, memlog, emulock, modfolder, winput, shot]
skills: [re-observe, re-setup]
docs: [docs/observe/running-the-original.md, docs/knowhow/wine-wow64-rosetta-fault.md]
seen: ['POD: Planet of Death (1997), 2026-10-23 to 2026-10-24']
---

# Running the original in an emulated PC when Wine on the host fails

## What to know

Try Wine on the host first. When it fails in a way that no patch to the game fixes, move to an
emulated x86-64 PC: QEMU, a Debian cloud image, Wine with real 32-bit libraries, Xvfb and a
window manager. POD's software build ran there at close to full speed under QEMU's TCG on an
M3 Max. Wine under Rosetta on the same Mac froze every few runs (*measured*).
`./gameframe run emupc init DIR --image QCOW2 --key KEY.pub --disc DISC.iso` writes the whole
setup.

What each part fixed in POD:

- **The disc as a CD-ROM device.** Wine on Linux reads a drive's type and label from the
  device. A folder mapped as D: is a fixed drive there, so the game asks for its CD. Cut the
  data track out of the BIN/CUE (`discinfo.py IMAGE --iso DISC.iso`) and attach it as a CD-ROM
  ([CD drive under Wine on Linux](wine-linux-cd-drive.md)).
- **A window manager with Xvfb.** Without one, no window becomes the foreground and
  DirectInput reads no keys ([window manager](xvfb-needs-window-manager.md)).
- **32-bit OpenGL.** Wine runs a 32-bit game with 32-bit libraries. Install
  `libgl1:i386 libglx-mesa0:i386 libgl1-mesa-dri:i386`. Mesa's llvmpipe draws in software.
  `LP_NATIVE_VECTOR_WIDTH=128` roughly doubled its speed under TCG.
- **rsync over SSH to copy files.** Debian's cloud kernel has no 9p modules for shared folders.
- **SSH with `IdentitiesOnly=yes`.** Without it, the agent's keys use up the server's tries
  ([too many authentication failures](ssh-too-many-auth-failures.md)).
- **Root for memory.** Reading the game's memory (`memrec.py`) needs root in the VM. `sudo`
  drops `DISPLAY` ([sudo and DISPLAY](sudo-drops-display.md)).
- **One VM per game.** Give each game project its own VM, in its own `derived/vm`, with its own
  port and name (`emupc.py init` refuses a port in use and names the VM after the project).
  Hardwar first ran in POD's VM: it tied one game to another's machine and load, and put one
  game's files inside the other's.
- **Sessions of one game sharing its VM.** Give each session its own display and Wine prefix. Stop only your own
  processes ([pkill and a shared VM](shared-vm-pkill-display.md),
  [pkill over SSH](vm-ssh-pkill-matches-itself.md)). Keep a note of who is using the VM:
  `emulock.py take gf-vm --who SESSION --why TEXT`, run inside the VM, and `free` after.
- **Colours.** The two routes drew POD's menu in different colours. The VM showed the true
  ones. Compare against a second, independent run before trusting colours.
