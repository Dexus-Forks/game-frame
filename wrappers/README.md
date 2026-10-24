# wrappers: replacement libraries for the 3D cards of 1997

A 1997 PC game built for a 3D card imports that card's own library: 3dfx Glide, ATI's
ATI3DCIF, S3's S3D Toolkit or PowerVR's SGL. It also expects a DirectDraw that grants a
full-screen 640 × 480 mode. No modern system has these libraries. Each DLL here has the
original's file name and exports, so the unchanged game loads it. The DLL then draws the
game's picture on today's machine at the size you choose. The method for writing such a
library is in [replacement libraries](../docs/hardware/replacement-libraries.md). The guide
[Replacement 3D-card libraries](../docs/hardware/wrappers.md) says how to build and use each
library, with pictures of POD's runs.

| DLL | Stands in for | Draws | README |
|---|---|---|---|
| `glide2x.dll` | 3dfx Glide 2.x (`glide.dll`, `glide2x.dll`) | The whole scene with OpenGL, at any resolution | [glide](glide/README.md) |
| `ddraw.dll` | DirectDraw 1 to 3 | Every surface in memory, any mode granted, the frame shown scaled in a window | [ddraw](ddraw/README.md) |
| `ati3dcif.dll` | ATI 3D Rage (ATI3DCIF) | The faces the game gives the chip, in software, into its DirectDraw surface | [ati3dcif](ati3dcif/README.md) |
| `s3dtkw.dll` | S3 ViRGE (S3D Toolkit, `S3DTKW.dll`) | The same | [s3dtkw](s3dtkw/README.md) |
| `sgl.dll` | PowerVR SGL, the scene graph and the immediate mode | The game's scene graph or its screen-space faces, drawn with OpenGL at any size; the 2D picture over it through `ddraw.dll` or SGL's own 2D callback | [sgl](sgl/README.md) |
| `94DLL32.dll` (`maxi94`) | Guillemot Maxi Sound 64 (Dream SAM9407) | No picture: the card's wave streams are Wine's wave outputs, so the game's sound plays | [maxi94](maxi94/README.md) |
| `ir41_32.dll` (`ir41`) | Intel's Indeo Video Interactive 4.1 codec (Video for Windows, `IV41`) | The game's videos, decoded by GameFrame's clean-room decoder, picture for picture as Intel's | [ir41](ir41/README.md) |

The shared crates:

| Crate | Does | Tested on the host |
|---|---|---|
| [shim](shim/README.md) | The shared settings (`WRAPPER_*`), Windows 95 port I/O emulated, in-memory patches from a file, the log of what is not implemented | the settings' parsers and the log's counts |
| [raster](raster/README.md) | A small software triangle rasteriser into 8-, 15- and 16-bit frame buffers | yes |
| [scalers](scalers/README.md) | Whole-multiple, Scale2x and CRT-style scalers for a finished frame | yes |
| [pixels](pixels/README.md) | RGB555 and RGB565 to 8-bit colour | yes |

The libraries come from the [POD: Planet of Death](https://github.com/tanepiper/pod-planet-of-death)
project ([case study](../docs/case-studies/pod.md)). They run its unchanged 3dfx, ATI, S3 and
PowerVR builds. Each library implements the calls POD makes, and its README lists what it does
not handle. The sound cards of the time, and which of their libraries can be stood in for,
are in [sound-card APIs](../docs/hardware/sound-card-apis.md).

## Every call of the original

Each DLL exports every call the original library exports, so a game that imports any of them
loads. The calls a library does not implement yet are stubs. A stub logs its call with its
arguments and returns a failure value the game can test. The aim is a full replacement for
each library, one call at a time.

| DLL | The original | Implemented | Exported |
|---|---|---|---|
| `glide2x.dll` | Glide 2.43 (`glide2x.dll`) and Glide 2.1.1 (`glide.dll`) | 131 | 156 |
| `sgl.dll` | PowerVR driver 3.1.0's `sgl.dll`, `sgl_delete_all_textures` from a later SGL, and `sgl_wrapper_present` | 61 | 134 |
| `ati3dcif.dll` | ATI 3D Rage Pro SDK 4.1's `ATI3DCIF.LIB` | 6 | 17 |
| `s3dtkw.dll` | S3's 1996 ViRGE driver's `S3DTKW.DLL` | 6 | 6 |
| `ddraw.dll` | DirectX 5's `DDRAW.DLL`, the public calls, and DirectX 7's `DirectDrawCreateEx` and `DirectDrawEnumerateEx` | 2 | 9 |

Each library's `api.tsv` is its original's API table: every export, its calling convention,
its argument bytes, its parameters and what a stub returns. `tools/pe/apitable.py` reads the
table from the original DLL's own code (the `ret N` of each function) or from its SDK's import
library, with the SDK header for the parameters. `wrappers/stubgen.py` writes each crate's
`.def` and `src/stubs.rs` from the table. It stubs every call the crate does not define. It
fails when a call is defined with the wrong convention. Inside `ddraw.dll`, every COM method
that is not implemented is a stub named by its interface (`IDirectDrawSurface::GetDC`).

The original libraries are not in this repository. Keep your copies outside it, as you do the
game's files. Only the tables, which hold names and sizes, are here.

To implement a call, write it in its module with `#[unsafe(no_mangle)]` and the original's
convention (`extern "system"` for stdcall, `extern "C"` for cdecl). Then run
`python3 wrappers/stubgen.py`. Test it against our own samples (`AGENTS.md`, rule 8).

## Build

Rust with the 32-bit Windows target, and zig 0.17 as the linker. No MinGW is needed.

```bash
rustup target add i686-pc-windows-gnu
cd wrappers && cargo build --release     # release: for playing
cd wrappers && cargo build               # debug: for working on a library
```

| | Release (`target/i686-pc-windows-gnu/release/`) | Debug (`target/i686-pc-windows-gnu/debug/`) |
|---|---|---|
| Optimisation | Full, with link-time optimisation | Light (level 1, the dependencies level 2), so a game still runs |
| Symbols | Stripped | Kept, for a debugger |
| Stubs and gaps | The first call of each logged, the rest counted | The first 50 of each logged, the rest counted |
| OpenGL | Not checked | Errors logged after each frame (`glide2x.dll`, `sgl.dll`) |
| Size | About 0.4 to 1.2 MB a DLL | About 6 to 10 MB a DLL |

The first log line of each DLL names its build.

`.cargo/config.toml` sets the target and links through GameFrame's `scripts/zcc.sh`. zcc takes
each DLL's exports from its crate's `.def` file (`glide/glide2x.def`, `sgl/sgl.def`). The host
tests:

```bash
cd wrappers && cargo test --release --target "$(rustc -vV | sed -n 's/^host: //p')" -p raster -p scalers -p pixels -p shim -p sgl -p maxi94
```

`tests/run.sh` runs them, builds both kinds of DLL, checks each export table against its `.def`
file, and runs `python3 wrappers/stubgen.py --check`.

## Use them with a game

1. **Find the libraries the game imports.** `./gameframe run peinfo GAME.EXE --imports` lists
   them. A game that loads its card library at run time names it only in its strings
   (`./gameframe run strings GAME.EXE | grep -i dll`).
2. **Copy the DLL next to the game's EXE, under the name the game asks for.** Rename
   `glide2x.dll` to `glide.dll` for an early Glide 2 game. Windows finds `s3dtkw.dll` as
   `S3DTKW.dll` (names are not case sensitive). Never put the game's files into this
   repository.
3. **Tell Wine to prefer the copy.** Wine has its own `ddraw.dll`, and some prefixes have a
   Glide wrapper. Set the override for each DLL you copied:
   `WINEDLLOVERRIDES="ddraw=n,b;sgl=n;glide=n;glide2x=n"`. On Windows the DLL next to the EXE
   is loaded first, except for `ddraw.dll`, which some versions take from the system folder.
   A local `GAME.EXE.local` folder or an application manifest then makes Windows load it.
4. **Set the variables.** Each DLL reads its settings from environment variables, and Wine
   passes the Unix environment to the game. The shared ones:

   | Variable | Meaning | Default |
   |---|---|---|
   | `WRAPPER_WINDOW_CLASS` | The class name of the game's main window. `glide` and `sgl` draw into that window. Find it with `./gameframe run strings GAME.EXE` near `RegisterClassA`, or in the game's `CreateWindowExA` call | the first visible top-level window of the process |
   | `WRAPPER_MARKER` | The RGB565 colour that means "not drawn" in a 2D picture laid over the 3D frame (hex `0x0821` or decimal). Choose another when the game draws this exact colour | `0x0821`, almost black |
   | `WRAPPER_PATCHES` | A file of patches to the game's code, applied in memory at load ([shim](shim/README.md)) | none |

   Each library's own variables are in its README: `GLIDE_*`, `DDRAW_*`, `SGL_*`, and the log
   files `GLIDE_LOG`, `DDRAW_LOG`, `ATI3DCIF_LOG`, `S3DTK_LOG`, `SGL_LOG` and `MAXI94_LOG`.
5. **Read the logs.** Each DLL logs `not implemented: NAME(arguments)` the first time the game
   calls a stub, and `not handled: WHAT: detail` the first time an implemented call meets a mode
   it does not draw. When the game exits, each DLL logs every one of them with its count. That
   list is what the game still needs. Use the debug build to see more calls of each.

Licence: Apache-2.0 (code), CC0-1.0 (the READMEs).
