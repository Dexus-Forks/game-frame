# ir41: an Indeo 4.1 codec for Video for Windows

`ir41_32.dll` stands in for Intel's Indeo Video Interactive 4.1 codec (`IR41_32.DLL`). It has
the same file name and the same export (`DriverProc`), so a game that checks for Intel's file
finds it, and Video for Windows (`AVIFile`, `AVIStreamGetFrame`, `ICDecompress`) plays the
game's `IV41` videos through it. The pictures come from GameFrame's clean-room decoder
([crates/indeo](../../crates/indeo/README.md)), byte-identical to Intel's codec's own.

It was made for POD Gold, whose six videos are `IV41`. `WINPOD.EXE` plays its videos only when
`[drivers32] vidc.IV41=ir41_32.dll` is in `SYSTEM.INI` and the file is in the system folder;
its Direct3D builds crash without a codec.

## What it does

| Message | Answer |
|---|---|
| `DRV_OPEN` | An instance for `vidc` (video), none for anything else |
| `ICM_GETINFO` | `IV41`, "Indeo Video Interactive 4.1 (GameFrame, clean room)" |
| `ICM_DECOMPRESS_QUERY`, `_BEGIN` | `IV41` in; out 24- or 32-bit RGB, 16-bit 555 or 565, `IF09` or `YVU9`, the same size, bottom-up or top-down |
| `ICM_DECOMPRESS_GET_FORMAT` | 24-bit RGB, bottom-up |
| `ICM_DECOMPRESS` | The chunk decoded; the picture Intel's codec would show for it written out (with `ICDECOMPRESS_HURRYUP`, decoded and not written) |
| `ICM_DECOMPRESS_END` | The decoder dropped |

What differs from Intel's codec: it gives no 8-bit palette pictures (Video for Windows then
asks for another format), and its RGB comes from the BT.601 equations, not from Intel's own
colour conversion. Its `IF09` and `YVU9` pictures are Intel's exactly. It has no compressor.

## Intel's tables

The decoder needs Intel's codebooks, run/value maps, scans and quantiser matrices. They are
Intel's data, so they are not in the DLL: it reads them from the player's own copy of Intel's
`IR41_32.DLL`, the first it finds of

1. the file `WRAPPER_IR41_TABLES` names;
2. `ir41_32.intel.dll` beside this DLL (copy the original there under that name);
3. `INSTALL\IR41_32.DLL` on a CD drive, D: to Z: (where POD Gold keeps Intel's setup).

Without them `ICM_DECOMPRESS_BEGIN` fails and the log says so.

## Install

As Intel's setup did: the DLL in the system folder (`syswow64` in a 64-bit Wine prefix,
`system32` in a 32-bit one), `vidc.IV41=ir41_32.dll` under `[drivers32]` in `SYSTEM.INI`, and
the value `vidc.IV41` = `ir41_32.dll` under
`HKLM\Software\Microsoft\Windows NT\CurrentVersion\Drivers32`. POD's `run/vm/indeo.sh` does
all three for a Wine prefix.

`IR41_LOG` names the log file (default: standard error).

## Test

```bash
cd wrappers && cargo test --target "$(rustc -vV | sed -n 's/host: //p')" -p ir41
```

The host tests cover the format negotiation, the rows written in each format, and where the
tables are looked for. Under Wine, Video for Windows playing POD Gold's `FINPOD2.AVI` through
the codec gave all 156 pictures, the `IF09` ones byte-identical to Intel's codec's
(*measured*, 2026-10-24).

Licence: Apache-2.0 (code), CC0-1.0 (this text).
