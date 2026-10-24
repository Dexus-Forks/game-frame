# glide: Glide 2 on OpenGL

`glide2x.dll` replaces 3dfx's Glide 2.x library. The game runs unchanged. It loads this
library in place of 3dfx's, and the library draws everything the Voodoo would have drawn with
OpenGL, at any output size (2×, 4×, or a size you name). It was made for POD's 3dfx builds
(`WPOD3DFX.EXE`, first edition and Gold). It exports the 55 functions they call, under the
decorated `__stdcall` names the games link to (`_grGlideInit@0` and so on, in `glide2x.def`).

Early Glide 2 games import the file `glide.dll`. Later ones import `glide2x.dll`. Copy the DLL
under the name your game's import table gives (`./gameframe run peinfo GAME.EXE --imports`).
How POD uses each call is in POD's `docs/hardware/3d-cards.md` ("Every Glide call, and how the
game uses it").

## Run

Put the DLL next to the game's EXE, set `WINEDLLOVERRIDES="glide=n;glide2x=n"` under Wine, and
start the game. Software OpenGL works: under Wine without a GPU, Mesa's llvmpipe draws. A
32-bit game needs 32-bit OpenGL (on Debian, `libgl1:i386 libglx-mesa0:i386
libgl1-mesa-dri:i386`).

| Variable | Meaning | Default |
|---|---|---|
| `GLIDE_SCALE` | Output size as a multiple of the game's resolution (`2`, `4`, `1.5`) | 2 |
| `GLIDE_SIZE` | Output size in pixels (`1920x1440`); wins over the scale | |
| `GLIDE_LFB_FILTER` | `linear` smooths the game's 2D pictures when scaled | `nearest` |
| `GLIDE_LOG` | A file to log to (`C:\glide.log`) | standard error |
| `GLIDE_SCALE_FILE` | A file holding a scale, read again at each `grSstOpen`. A game that opens the card again for each level can run its menus small and the level large | |
| `WRAPPER_WINDOW_CLASS` | The game's window class (`../README.md`). POD's is `Fenetre Pod` | the first visible window |
| `WRAPPER_MARKER` | The marker colour of the 2D frame buffer (below) | `0x0821` |

Speed in an emulated PC (*measured* in POD, the library logs frames per second): POD's menus
ran at about 5 frames a second at 1×, 2 at 2× and 0.4 at 4×, with Mesa's llvmpipe on an
emulated CPU. `LP_NATIVE_VECTOR_WIDTH=128` made llvmpipe faster under QEMU.

## How it works

| Module | Does |
|---|---|
| `api/` | The exported calls (`sst`, `state`, `tex`, `draw`, `lfb`). They answer queries and record everything else as commands |
| `cmd.rs` | The commands and the queue. A frame's commands go to the render thread at `grBufferSwap`. A game may call Glide from more than one thread, and an OpenGL context belongs to one |
| `render/` | The render thread: the frame buffer (`frame.rs`, two colour buffers and one depth buffer, like Glide's), the state (`state.rs`), the texture memory (`textures.rs`, decoded with `gameframe-gpu-1997`), the shaders (`shaders.rs`) |
| `window.rs` | The early Glide 2 `grSstOpen` takes no window. So the library finds the game's own window (`WRAPPER_WINDOW_CLASS`, which may still be hidden; without it the first visible window of the process), makes it borderless at the output size, and draws into it. Without a game window it makes one. The later `grSstWinOpen` names the window |
| `shim` (crate) | Port I/O emulated, and patches from `WRAPPER_PATCHES` ([shim](../shim/README.md)) |

The Voodoo's pixel pipeline is one fragment shader. It does the colour and alpha combiners
(`grColorCombine`, `guColorCombineFunction`, `guAlphaSource`), the chroma key, and table fog
(the 64 entries indexed by *w*, as the Voodoo does). It also does the texture combine for one
TMU. Depth is the W buffer (or Z buffer), mapped so that Glide's compare functions are
OpenGL's unchanged. Gamma (`grGammaCorrectionValue`) is applied when the frame is shown, as
the Voodoo's output table did.

A game can write 2D pictures straight into the frame buffer (`grLfbGetWritePtr`). The library
gives it a memory buffer with the Voodoo's stride of 1024 pixels, filled with the marker. At
`grLfbEnd` every changed pixel is drawn into the frame at the output scale. Reads of the depth
buffer (POD's lens flare tests it) and of the front buffer (screenshots) come back at the
game's resolution.

## Differences from the Voodoo

Each is marked `Deviation:` in the code.

- A 2D pixel written with the exact marker value counts as not written.
- 2D writes through the pixel pipeline are blended and chroma-keyed, but neither depth-tested
  nor fogged.
- The destination blend factor `GR_BLEND_PREFOG_COLOR` is taken as zero.
- Only TMU 0 draws. NCC textures and mipmaps are not handled (POD has none).

Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
