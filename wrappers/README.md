# wrappers: replacement libraries for the 3D cards of 1997

A 1997 PC game built for a 3D card imports that card's own library: 3dfx Glide, ATI's
ATI3DCIF, S3's S3D Toolkit or PowerVR's SGL. It also expects a DirectDraw that grants a
full-screen 640 × 480 mode. No modern system has these libraries. Each DLL here has the
original's file name and exports, so the unchanged game loads it. The DLL then draws the
game's picture on today's machine at the size you choose. The method for writing such a
library is in [replacement libraries](../docs/hardware/replacement-libraries.md).

| DLL | Stands in for | Draws | README |
|---|---|---|---|
| `glide2x.dll` | 3dfx Glide 2.x (`glide.dll`, `glide2x.dll`) | The whole scene with OpenGL, at any resolution | [glide](glide/README.md) |
| `ddraw.dll` | DirectDraw 1 to 3 | Every surface in memory, any mode granted, the frame shown scaled in a window | [ddraw](ddraw/README.md) |
| `ati3dcif.dll` | ATI 3D Rage (ATI3DCIF) | The faces the game gives the chip, in software, into its DirectDraw surface | [ati3dcif](ati3dcif/README.md) |
| `s3dtkw.dll` | S3 ViRGE (S3D Toolkit, `S3DTKW.dll`) | The same | [s3dtkw](s3dtkw/README.md) |
| `sgl.dll` | PowerVR SGL, the scene-graph half | The game's scene graph, kept and drawn with OpenGL at any size; the 2D picture over it through `ddraw.dll` | [sgl](sgl/README.md) |
| `94DLL32.dll` (`maxi94`) | Guillemot Maxi Sound 64 (Dream SAM9407) | No picture: the card's wave streams are Wine's wave outputs, so the game's sound plays | [maxi94](maxi94/README.md) |

The shared crates:

| Crate | Does | Tested on the host |
|---|---|---|
| [shim](shim/README.md) | The shared settings (`WRAPPER_*`), Windows 95 port I/O emulated, in-memory patches from a file, the log | the settings' parsers |
| [raster](raster/README.md) | A small software triangle rasteriser into 8-, 15- and 16-bit frame buffers | yes |
| [scalers](scalers/README.md) | Whole-multiple, Scale2x and CRT-style scalers for a finished frame | yes |
| [pixels](pixels/README.md) | RGB555 and RGB565 to 8-bit colour | yes |

The libraries come from the [POD: Planet of Death](https://github.com/tanepiper/pod-planet-of-death)
project ([case study](../docs/case-studies/pod.md)). They run its unchanged 3dfx, ATI, S3 and
PowerVR builds. Each library implements the calls POD makes, and its README lists what it does
not handle. A game that makes other calls gets a log line for each unknown call. Add those
calls here, with a test against our own samples (`AGENTS.md`, rule 8). The sound cards of
the time, and which of their libraries can be stood in for, are in
[sound-card APIs](../docs/hardware/sound-card-apis.md).

## Build

Rust with the 32-bit Windows target, and zig 0.17 as the linker. No MinGW is needed.

```bash
rustup target add i686-pc-windows-gnu
cd wrappers && cargo build --release
```

`.cargo/config.toml` sets the target and links through GameFrame's `scripts/zcc.sh`. zcc takes
each DLL's exports from its crate's `.def` file (`glide/glide2x.def`, `sgl/sgl.def`). The DLLs
land in `target/i686-pc-windows-gnu/release/`. The host tests:

```bash
cd wrappers && cargo test --release --target "$(rustc -vV | sed -n 's/^host: //p')" -p raster -p scalers -p pixels -p shim -p sgl -p maxi94
```

`tests/run.sh` runs both, and checks each DLL's export table against its `.def` file.

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
5. **Read the logs.** Each DLL logs every call it does not handle, once. The first run shows
   which calls a new game needs.

Licence: Apache-2.0 (code), CC0-1.0 (the READMEs).
