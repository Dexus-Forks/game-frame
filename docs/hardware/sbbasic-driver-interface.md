# Creative SBBASIC package: driver trace status

This note records provenance and the extraction attempt for Creative's `SBBASIC.EXE` SDR-31STD-1-US
Revision 1 package. Evidence is package metadata, a hash-verified archive and one DOSBox-X run. The
driver payload was not extracted or inspected, so this note contains no behavior specification and
must not be used as implementation input.

## Package provenance

[Observed] Creative's [support listing](https://cn.creative.com/support/downloads/download.asp?DriverLang=2052&searchString=SBBASIC.EXE) identifies the package as the Sound Blaster 16/SB32/AWE32 Basic Disk for DOS/Windows 3.1, dated 30 December 1997. The included package identification file names release SDR-31STD-1-US, revision 1, and notes AWE64 upgrade support. The included README describes DOS audio drivers, a DOS mixer and diagnostic, plus Windows 3.1 utilities.

[Observed] The extracted ZIP came from the [public mirror](https://software.shi.foo/drivers/sound-blaster-basic-dos-drivers/1/sound-blaster-basic-dos-drivers-extracted.zip). Its locally verified SHA-256 is `bc7889685a24aaa4dc01f73700297cdf16bab63b220a48cf0f856c0714641896`. The mirror's [checksum listing](https://software.shi.foo/drivers/sound-blaster-basic-dos-drivers/1/checksums) reports `c28f21521e8d9d8cd02a0c683aadbe369f7a9e182950ccbd46ba21762371871f` for the original `SBBASIC.EXE`; that executable was not separately downloaded or hash-checked in this run.

[Observed] The ZIP contains `INSTALL.EXE` (166,916 bytes; dated 1996-10-15) and multiple `.PVL` payload archives. `file` identifies the payloads as TTComp archives with a 4K dictionary. `7zz l DRIVER.PVL` reported that it could not open the file as an archive. No matching decoder appeared in GameFrame's format tools.

## Extraction attempt

[Observed] `INSTALL.EXE` was launched in DOSBox-X 2026.10.01 with a temporary writable C: drive and the extracted package mounted read-only as D:. The installer staged setup support files under `C:\ISFYZQVO.TWJ` and remained interactive. The same files appeared under `/tmp/sbbasic-trace.lJ24G6/dosroot/ISFYZQVO.TWJ` on the host.

[Unknown] The installer window was not accessible through the available UI connection, so its next prompt and subsequent file extraction could not be observed. No agreement was accepted. The DOS driver and diagnostic executable names remain unknown. No driver I/O or card-detection behavior was observed, and no functional behavior is inferred from the package inventory.

The command used was:

```sh
dosbox-x -defaultconf -fastlaunch -noconfig -noautoexec \
  -c "mount c /tmp/sbbasic-trace/dosroot" \
  -c "mount d /tmp/sbbasic-trace/extracted -ro" \
  -c "d:" -c "install.exe"
```

The paths above stand in for the unique `mktemp` directory used in the run. Create a writable
`dosroot` and an `extracted` folder containing the verified ZIP's contents before running it.

## Open

Download the ZIP from the mirror, verify its SHA-256, and extract it outside the repository. Create an empty writable directory for DOSBox-X's C: drive. Mount the extracted package read-only as D:, run `INSTALL.EXE`, and use the visible installer to choose the temporary C: destination. The earlier command shows the mounts used in this run. Continue only after the DOSBox-X window can be controlled. The package README identifies a DOS diagnostic program; inspect its probes and the extracted driver after installation. No physical card is needed for this extraction step.

Once the behavior is recorded in a neutral specification, a separate implementation agent can receive that specification without the package, disassembly or raw extraction output. The current source pass did not create that specification, so there is no implementation handoff yet.
