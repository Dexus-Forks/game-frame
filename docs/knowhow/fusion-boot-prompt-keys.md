---
id: fusion-boot-prompt-keys
title: 'A Windows ISO times out at "Press any key to boot from CD" in VMware Fusion'
kind: failure
status: workaround
platform: [windows-nt, macos]
runtime: [virtual-machine, apple-silicon]
symptom: [wont-start]
tags: [vmware-fusion, vmcli, efi, boot, keys]
signatures:
  - 'EFI VMware Virtual SATA CDROM Drive \([0-9.]+\)\.\.\. (?:No Media|Time out)'
  - 'Status upon boot failure: (?:No Media|Time out)'
tools: []
skills: [re-setup]
docs: [docs/knowhow/context-windows-machines-apple-silicon.md]
seen: ['Banished (2014), Fusion 26.0.1, Windows 11 23H2 Arm ISO, 2026-10-27']
---

# A Windows ISO times out at "Press any key to boot from CD" in VMware Fusion

## Symptoms

A new machine started with `vmrun start` shows "Time out", then "EFI VMware Virtual SATA
CDROM Drive (1.0)... No Media" and "EFI Network...". The log says
`Status upon boot failure: Time out`.

## Cause

The ISO's boot loader waits a few seconds for a key. Nobody pressed one (*observed*). A second
CD drive with a data-only ISO, such as an answer file, then shows "No Media", as it has no
boot loader.

## What to do

Press a key in the machine's window when the prompt shows. From a script, send the key with
`vmcli`, giving the USB HID code in decimal: space is 44.

```
vmcli VMX MKS sendKeyEvent 44 0
```

- The hex form `0x2c` changed nothing (*observed*).
- Keys sent in the first seconds reach the firmware instead and open its Boot Manager
  (*observed*). From there, the down arrow (81), down again and Enter (40) chose
  "SATA CDROM Drive (0.0)", and space then answered the prompt.
- `vmcli VMX MKS captureScreenshot FILE.png` shows which screen is up before each key.
