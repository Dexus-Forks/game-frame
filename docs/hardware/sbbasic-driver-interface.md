# Creative SBBASIC driver package status

The earlier extraction blocker is resolved. The package installed in a disposable DOS and Windows environment. A bounded Win16 auxiliary volume contract is available in the [driver behavior contract](sbbasic-driver-behavior.md).

The source-specification and source-blind implementation process, its slice boundaries and
remaining software-only probes are recorded in the [SBBASIC workflow note](sbbasic-cleanroom-process.md).

## Package and install provenance

[Observed] The installed package is Creative SBBASIC SDR-31STD-1-US Revision 1. Its SHA-256 is `c28f21521e8d9d8cd02a0c683aadbe369f7a9e182950ccbd46ba21762371871f`.

[Observed] The installer completed in a disposable 86Box clone running MS-DOS 6.22 and Windows 3.11. No binding agreement appeared during installation. The installer reported edits to CONFIG.SYS, AUTOEXEC.BAT, WIN.INI, SYSTEM.INI and CONTROL.INI, and created backup files with B~K extensions.

[Observed] Installation used a disposable clone. The clean base image was left unchanged.

[Observed] Read-only host inspection of the installed image found the drivers, utilities and support files. The primary Windows driver is `SB16SND.DRV`; the Creative stereo synthesizer is `SB16FM.DRV`. The package also installs `VSBPD.386`, `CSPMAN.DLL`, `AUXDRV.DRV`, `DIAGNOSE.EXE` and `MIXERSET.EXE`. The behavior contract records checksums for the traced files.

## Current findings

[Observed] The installed DOS profile uses base I/O 220h, IRQ 5, low DMA 1, high DMA 5 and MPU-401 port 330h. AUTOEXEC sets SOUND, BLASTER and MIDI values, runs `DIAGNOSE /S`, then runs `MIXERSET /P /Q`.

[Observed] Windows setup registers `SB16SND.DRV` for Wave, Aux, Mixer and the SB16 MIDI port. It registers `SB16FM.DRV` for the Creative stereo synthesizer. Both entries use `VSBPD.386`.

[Observed] On boot, the package reports that CSP is not detected in this 86Box SB16 profile. `DIAGNOSE /S` still reports A220, I5, D1, H5 as the configured card resources.

[Inferred] The first bounded implementation target is the `AUXMESSAGE` volume query and update surface in `SB16SND.DRV`. The [behavior contract](sbbasic-driver-behavior.md) separates this surface from the DOS preset utility and lists the unresolved hardware behavior.

[Unknown] Wave format limits and buffer callback timing, MIDI scheduling and low-level port/IRQ/DMA operations still need software-only probes. The CSP capability remains unavailable in the observed emulated profile.

## Earlier extraction attempts

[Observed] Initial attempts under Windows 98 were rejected because the installer expects DOS or Windows 3.1. The supported DOS 6.22 and Windows 3.11 setup resolved that blocker. Those attempts produced no driver payload and are superseded by the installed-file evidence above.

[Observed] No binary, disassembly, pseudocode or raw driver data was added to the repository.
