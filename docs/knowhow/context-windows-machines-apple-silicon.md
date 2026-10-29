---
id: context-windows-machines-apple-silicon
title: 'Choosing a Windows machine for a game on an Apple-silicon Mac'
kind: context
status: context
era: [1980s, early-1990s, late-1990s, 2000s, 2010s]
platform: [dos, windows-3x, windows-9x, windows-nt, macos]
runtime: [apple-silicon, virtual-machine, wine]
hardware: [x86-cpu, 3d-card]
api: [direct3d]
tags: [86box, qemu, tcg, vmware-fusion, parallels, prism, windows-10, windows-11-arm, decision]
commands:
  - '(?:^|[;&|(]\s*|/)(?:vmrun|vmcli)(?=\s|$|")'
  - '(?:^|[;&|(]\s*|/)qemu-system-(?:i386|x86_64)(?=\s|$|")'
tools: []
skills: [re-observe, re-setup]
docs: [docs/observe/running-the-original.md, docs/observe/debuggers.md, docs/knowhow/context-wine-macos.md, docs/knowhow/context-emulated-pc.md]
seen: ['Banished (2014), 2026-10-27']
---

# Choosing a Windows machine for a game on an Apple-silicon Mac

## What to know

Pick the machine by the game's era. An Apple-silicon Mac has no x86 hardware, so every x86
Windows runs either in an emulator or translated inside Windows for Arm. The emulators are
exact but slow, and have no modern 3D card. The Arm VMs are fast, but they only run Windows 11
for Arm, which translates the game's x86 code. No option gives a fast x86 Windows 10 with a
DirectX 9 card. For that, use a real x86 PC on the network.

| Era of the game | Machine | Speed | Debugging |
|---|---|---|---|
| DOS, Windows 3.x, Windows 9x (DirectX 1 to 7, Glide) | 86Box, or DOSBox-X for DOS | Right for the era | 86Box's GDB stub, a debugger inside the guest |
| Any x86 Windows, when exact x86 matters more than speed | QEMU with TCG: Windows 10 32-bit or 64-bit | Slow. Setup took over an hour (*observed*). No 3D card, so DirectX 9 falls back to software | A GDB stub on the whole x86 machine, `savevm` snapshots, COM1 for WinDbg's kernel debugger |
| DirectX 9 to 11, x86 or x64 (Banished's era) | VMware Fusion or Parallels with Windows 11 for Arm | Fast. Parallels was reported at 60 fps for Banished on an M1 Max (reported, AppleGamingWiki) | A debugger inside Windows: the x64 WinDbg for x64 programs, the x86 one for 32-bit programs (Microsoft's advice). x64dbg and Frida are untested there |
| The same, for a quick run with API logs | Wine or CrossOver on the Mac | Fast. Banished was reported to run with d3d11 turned off in winecfg (reported, CodeWeavers forum) | Wine's trace channels. Attaching winedbg killed a running game ([Wine on macOS](context-wine-macos.md)) |
| Anything, with every Windows debugger working | A real x86 PC, or a cloud VM with a GPU | Native | Everything: x64dbg, WinDbg, Frida, Ghidra's debugger |

What each one cannot do, as found on an M3 Max in October 2026:

- **86Box** stops at about a Pentium II and a Voodoo 3. It cannot run Windows 10, which needs
  PAE, NX and SSE2, or any DirectX 9 card.
- **QEMU with TCG** runs Windows 10 with a `Nehalem,+nx` CPU (32-bit) or `Skylake-Client`
  (64-bit, which warns that `invpcid`, `rtm` and `xsavec` are missing; Windows does not need
  them). It has no 3D card. Do not stop it during Setup: see
  [Setup restarted](qemu-windows-setup-restarted.md).
- **VMware Fusion** (26.0.1) and **Parallels** run only Arm guests on Apple silicon. Windows 10
  for x86 or x64 does not run in them. Fusion's own Windows 11 Arm Setup had problems here:
  see [Setup hangs on the logo](fusion-windows-arm-setup-hangs.md) and
  [the boot prompt](fusion-boot-prompt-keys.md).
- **Fusion's GDB stub** (`debugStub.listen.guest64 = "TRUE"`, port 8864) works and answers as
  an aarch64 target (*measured*: `qSupported`, `?`, the target description, `g` and a clean
  detach). It sees the Arm CPU. An x64 game appears there only as Prism's translated Arm code,
  so it is not a debugger for the game's own instructions.
- **Fusion's tools** are in `/Applications/VMware Fusion.app/Contents/Public`. `vmrun` starts,
  stops, snapshots and clones. With VMware Tools and a guest login, it also runs programs and
  copies files. `vmcli VMX MKS captureScreenshot FILE` takes a screenshot from the firmware on,
  without Tools. `vmss2core` turns a suspended machine's memory into a core dump.
- **Parallels Standard** limits a VM to 4 vCPUs and 8 GB and has no command line. Pro, a
  subscription, has both (Parallels' buy page).

## Evidence

- *measured*: the Fusion GDB stub answers above; QEMU's Setup times and warnings in its log.
- *observed*: Windows 10 32-bit Setup under QEMU reached the first-run screens; 64-bit Setup
  failed after the machine was stopped mid-install.
- reported, not tested here: the Banished frame rates and Wine settings (AppleGamingWiki's
  Banished page; the CodeWeavers forum thread on Banished); Microsoft's advice on debugging
  emulated programs on Arm64 ("Debugging Arm64" in the Windows debugger documentation).
