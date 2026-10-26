# Using 86Box as a PC test machine

86Box emulates IBM PC-compatible machines. See the [project overview](https://github.com/86Box/86Box).
Use it when behavior depends on a particular PC, BIOS, expansion bus, or guest operating system.
It is useful for observing period sound cards and their drivers in Windows 3.x, Windows 95 and
Windows 98. Use DOSBox-X for quick DOS runs that need its debugger or file access log.

Treat 86Box as a software reference, not as hardware. Label each measurement with the 86Box
version and machine profile. Do not give 86Box source to a source-blind implementation agent.
Give that agent only the behavior description and its setup conditions.

## Start a known machine

Create or select a machine in 86Box's manager and start it there. If the executable is on your
path, launch a machine by its directory. See the [VM manager options](https://86box.readthedocs.io/en/v6.0/usage/manager.html):

```text
86Box --vmpath /path/to/machine
```

`-P` is the short form. The manager can also clone a machine; use a separate clone before a
driver installer or configuration change that should not affect the reference setup. Keep the
machine's ROM files, operating-system media, disks and game files outside the GameFrame checkout.

## Configure a sound card

Open the machine settings and select **Sound**. 86Box supports up to four cards and lists only
those supported by the machine's expansion buses. For ISA cards, use **Configure** to set I/O
ports, IRQ and DMA channels ([sound settings](https://86box.readthedocs.io/en/v6.0/settings/sound.html)).
Parallel port DACs such as Covox and Disney Sound Source are configured under **Ports**.

Match the card and resources to the game's driver setup. For Windows, install the driver for that
card in the guest OS. For DOS, set the game's configuration to the selected ports, IRQ and DMA.
Change one variable at a time when comparing card or driver behavior.

## Debugging

For Windows 3.x drivers, the Windows 3.1 SDK documents CodeView for Windows and WDEB386. The
driver kit notes that CVW needs debug symbols and may use a secondary display. WDEB386 uses a
serial terminal. Confirm that the tool media and the VM's display or serial setup are available
before depending on either debugger.

86Box has an optional GDB stub in source builds. The opt-in `86box-mcp` provider includes tools
for breakpoints, instruction stepping, port I/O and PCI inspection. Build 86Box with both
`-D AUTOMATION=ON` and `-D GDBSTUB=ON`, then start a VM with `vm_start(debugger=true)`. In the
supported source version, `GDBSTUB=ON` forces `DYNAREC=OFF`. The stub reports machine state; it
does not provide the guest's Win16 source symbols. See the
[MCP setup guide](../working/mcp-toolchain.md) and [debugger guide](debuggers.md) for setup and
scope.

## Record the setup

Record enough detail to recreate each run:

- 86Box release and build, and the machine profile used.
- Machine model, CPU and clock, BIOS, memory, chipset and expansion buses.
- Guest OS edition and service pack, plus the installed sound driver and its version.
- Card model and configured I/O ports, IRQ, DMA and MIDI output device.
- Game version, settings, media hashes, and the input sequence or driver action being observed.

Keep captures beside their conditions in the project's ignored `derived/` folder. Label them as
86Box observations. Treat differences as emulator-specific until another source resolves them.
Keep OS media, ROMs, disk images and game files outside Git.
