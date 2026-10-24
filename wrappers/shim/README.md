# shim: what every replacement DLL shares

`shim` is a plain Rust library that each DLL in this workspace links in.

- `config`: the settings the DLLs share, read from environment variables.
  `WRAPPER_WINDOW_CLASS` names the game's main window. `WRAPPER_MARKER` sets the marker colour.
  `WRAPPER_PATCHES` names the patch file. The parsers run on any host and are tested there.
- `ports`: port I/O as Windows 95 allowed it. A game of the time may probe the CPU through
  ports (POD's 3dfx build reads the Cyrix registers at ports 0x22 and 0x23). Windows NT and
  Wine stop that with a privileged-instruction fault. A vectored exception handler, installed
  from each DLL's `DllMain`, steps over `cli`, `sti` and `out`, and makes `in` read 0xff. That
  is what a machine without the device answers (*inferred*: no device decodes those ports).
- `patch`: patches to the game's code, applied in memory at load, so the EXE on disk stays
  unchanged.
- `iat`: one of the game's imports replaced by the wrapper's own function, which calls on to
  the original. `maxi94` hooks POD's `waveOutOpen` and `waveOutClose` this way, to know which
  wave streams are open when the game asks the card.
- `log`: one log per DLL, to the file its variable names, or to standard error.

## The patch file

`WRAPPER_PATCHES` names a text file with one patch on each line. Keep the file in the game's
own repository, since it holds bytes of the game's code.

```text
# stamp     build         address   old bytes       new bytes       why
0x345270ab  GAME3DFX.EXE  0x486aa1  c705001122ff    c70500112204    unknown CPU: kind 4 (Intel P6)
```

| Field | Meaning |
|---|---|
| stamp | The EXE's `TimeDateStamp` from its PE header (`./gameframe run peinfo GAME.EXE`). A patch applies only to the build with this stamp |
| build | A name for the log |
| address | The virtual address in the loaded EXE |
| old, new | Hex bytes of the same length. The patch applies only when the bytes at the address are `old` |
| why | The rest of the line, written to the log |

Lines that start with `#` and blank lines are skipped. A line that does not parse stops the
whole file, and the log names the line. When several DLLs in one process load the file, the
first applies each patch and the others find the new bytes already there.

Licence: Apache-2.0 (code), CC0-1.0 (this text).
