---
id: fusion-windows-arm-setup-hangs
title: 'Windows 11 Arm Setup stops on the Windows logo in a VMware Fusion VM made with vmcli'
kind: failure
status: open
era: [2010s]
platform: [windows-nt, macos]
runtime: [virtual-machine, apple-silicon]
symptom: [hang]
tags: [vmware-fusion, vmcli, windows-11-arm, setup, install]
tools: []
skills: [re-setup]
docs: [docs/knowhow/context-windows-machines-apple-silicon.md, docs/knowhow/fusion-boot-prompt-keys.md]
seen: ['Banished (2014), Fusion 26.0.1, Windows 11 23H2 Arm ISO, M3 Max, 2026-10-27']
---

# Windows 11 Arm Setup stops on the Windows logo in a VMware Fusion VM made with vmcli

## Symptoms

The machine boots the Windows 11 Arm ISO and shows the Windows logo with no spinning dots. It
stays there for over five minutes. `vmware-vmx` uses about one core. Its log shows
`SMC: VPC 0x... iss 0 x0 0xc3000001` around the boot.

## Evidence

The VM was made from the command line, not Fusion's New VM assistant:
`vmcli VM Create -n NAME -d DIR -c arm-windows11-64`, then a hand-written .vmx with `firmware =
"efi"`, an NVMe disk, two SATA CD drives (the ISO and an answer-file ISO), e1000e network,
USB keyboard and mouse, `mks.enable3d = "TRUE"`, and no TPM. The answer file sets
`LabConfig\BypassTPMCheck` and `BypassSecureBootCheck` in the windowsPE pass.

## Tried

- 6 vCPUs, `cpuid.coresPerSocket = "6"`, `hpet0.present`, `svga.graphicsMemoryKB = "8388608"`,
  the GDB stub on: stopped on the logo for over five minutes.
- 4 vCPUs, without `hpet0`, `coresPerSocket`, the graphics memory line and the stub: the boot
  was interrupted at the firmware's boot menu, so the result is not known.
- The same, with a fully unattended answer file: the machine powered off within three minutes
  and started again. Why it powered off was not recorded.

Not tried yet: a VM made with Fusion's New VM assistant, which adds a TPM with partial
encryption; a newer ISO (24H2 or 25H2); the en-us ISO; one CD drive only.
