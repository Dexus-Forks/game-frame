# Creative SBBASIC Win16 auxiliary volume contract

This contract records one bounded Windows 3.1 driver surface. It covers auxiliary volume queries and updates through `SB16SND.DRV`. Static analysis supports this slice. No runtime call to the auxiliary volume messages was captured.

Evidence labels distinguish measured setup, binary behavior, inference and unknowns. The contract contains behavior descriptions only. It does not record code addresses, source excerpts or disassembly.

## Reference environment

[Observed setup] The package installed in a disposable 86Box clone running MS-DOS 6.22 and Windows 3.11. The driver work clone is paused at `C:\>` after reboot. The clean DOS and Windows base images remain untouched.

[Observed setup] The installed DOS settings use base I/O port 220h, IRQ 5, low DMA channel 1, high DMA channel 5 and MPU-401 port 330h. The `BLASTER` environment describes the same profile and identifies card type 6. Windows settings name the SB16 driver ports, interrupt and DMA channels.

[Observed setup] Windows setup registers `SB16SND.DRV` for Wave, Aux, Mixer and the SB16 MIDI port. It registers `SB16FM.DRV` for the Creative stereo music synthesizer. The Windows drivers use `VSBPD.386` as their virtual-device component.

[Observed setup] DOS startup loads `CSP.SYS` with base port 220h. It sets the SOUND, BLASTER and MIDI environment values, runs `DIAGNOSE /S`, runs `MIXERSET /P /Q`, then starts SMARTDRV. CONFIG.SYS sets FILES to 40.

[Observed runtime] The driver prints that it cannot detect the CSP in this 86Box SB16 profile. `DIAGNOSE /S` still reports that the audio card is configured at A220, I5, D1, H5. This is a profile capability limit. It does not establish a general DOS-driver failure.

## Selected public surface

[Observed artifact] The installed `SB16SND.DRV` exports the Win16 multimedia auxiliary entry point `AUXMESSAGE`. The same driver also exposes Wave, MIDI, mixer and auxiliary entry points. The selected contract covers only the auxiliary volume query and update behavior.

[Inferred from binary behavior] Auxiliary message 5 reads a stereo volume pair. The driver copies two 16-bit channel values from its current per-device state to the caller's output area. Auxiliary message 6 updates a stereo volume pair. The driver passes the two 16-bit values and the selected auxiliary unit to a lower sound-driver service.

[Observed binary behavior] A successful update can trigger an immediate apply step when the selected unit is active. The message handler returns after the lower service call and any applicable apply step. No timer or asynchronous completion is visible on this path.

[Observed binary behavior] The dispatcher reports seven auxiliary units when its lower driver state is available. It reports zero when that state is absent. The setter contains distinct selector handling for unit IDs 0 through 6. The dispatcher allows an ID of 7 through its first range check, but its setter behavior for ID 7 is unresolved.

[Observed binary behavior] The dispatcher returns numeric status 2 for an out-of-range ID or an unresolved unit handle. It returns 3 for ordinary requests when the lower driver is absent. It returns 4 when the unit ownership check fails. It returns 8 for an unhandled message or a disabled-mode conflict. A successful volume request returns 0. The meanings of statuses beyond these observed paths remain unverified.

[Observed binary behavior] The volume query reads the pair held in the driver's unit state. The update path sends the selected unit and channel pair to a lower service before returning. The binary does not expose the hardware register operations through this public path.

## Companion DOS setup behavior

[Observed package behavior] `MIXERSET /P` applies the `[PRESET]` values in `CTMIX.CFG`. `/Q` suppresses the setting display. AUTOEXEC.BAT uses both switches.

[Observed package configuration] The installed preset sets Master, Voice, MIDI, CD and Line levels to 224 with balance 50. It sets microphone level to 224 and PC Speaker level to 128. Bass and Treble use level 160 with balance 50. Input and output gains are both 2 on both sides. AGC is enabled. The preset also selects left and right recording sources and output sources.

[Observed runtime] `DIAGNOSE /S` runs with the installed BLASTER profile and reports the card resources listed above. The package's DOS utility documentation says `/S` configures non-Plug-and-Play devices from BLASTER and reports hardware resources for Plug-and-Play devices. Only the installed 86Box profile was observed here.

## Limits and unknowns

[Unknown] The exact Windows SDK names corresponding to auxiliary message numbers 3 through 6 have not been verified in a runtime call. The volume roles for messages 5 and 6 are inferred from their read and update effects.

[Unknown] The names of auxiliary unit IDs 0 through 6, the valid numeric range and scale for volume values, and any clamping or quantization have not been measured. ID 7 passes the initial range check but has no confirmed setter behavior.

[Unknown] The lower service's hardware port writes, register sequence, IRQ use, DMA use, and timing are not established. This contract does not model them.

[Unknown] Wave output and input have separate Win16 entry points. The complete accepted format matrix, buffer queue order, callback timing, pause, restart and reset behavior remain open. The static export contains a format-validation path and separate configured duplex handling, but no runtime results were captured.

[Unknown] The MIDI output paths in `SB16SND.DRV` and `SB16FM.DRV` have not been traced through the full callback and hardware boundary. MIDI event scheduling and callback order remain open.

[Unknown] CSP-dependent operation is unavailable under the observed 86Box profile. No claim is made about real CSP hardware behavior.

## Software-only verification suggestions

[Suggested] Use a fresh copy of the DOS 6.22 and Windows 3.11 base in 86Box with the same SB16 resources. Call the auxiliary device-count, caps, get-volume and set-volume messages through a small Win16 Multimedia test program. Record return values and left/right readback for unit IDs 0 through 7 and volume inputs 0, midpoint and maximum. Repeat with the lower driver absent if the clone can be configured that way.

[Suggested] In the same disposable profile, use a Win16 Wave test program to open PCM formats, queue multiple buffers, and capture done callbacks during pause, restart and reset. Record the accepted sample rates, channel counts, bit depths and callback order.

[Suggested] Run `DIAGNOSE /S` and `MIXERSET /P /Q` only on a disposable VM copy. Record displayed output, returned status, resulting mixer state and changes to DOS configuration. Do not use the clean base image for these probes.

## Evidence identities

[Observed artifact] The installed disk image and read-only host view were used for this analysis. Their contents remain outside this repository.

[Observed artifact] `SB16SND.DRV` SHA-256: `bd4cd8a266e8950b810b55029af1ef5d7f562594846bda8d86a297d79e7d1979`.

[Observed artifact] `SB16FM.DRV` SHA-256: `a41956bdc8cf1eb53e7b4a8d9e6d898164eff0b217973c1ef2838335927febe0`.

[Observed artifact] `VSBPD.386` SHA-256: `58949a353e12601f5ceafe584d18e2eda3a7de46c08ec3c20248927c0f329c46`.

[Observed artifact] `AUXDRV.DRV` SHA-256: `4ba0f25a7e2484aa5dccbd366e9087212d6835a338536a9d5671598c8af7cc47`.

[Observed artifact] `MIXERSET.EXE` SHA-256: `39f6d64c99c4f4c0f120c1eb802057f6fddad334d71a65e9325a10a090bfa5f1`.

[Observed artifact] `DIAGNOSE.EXE` SHA-256: `aa2edb5f813ad2f6024d0ac87939faa670ec2d5eead0265f5a693a0ffced4816`.
