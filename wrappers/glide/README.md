# glide: Glide 2 on OpenGL

`glide2x.dll` replaces 3dfx's Glide 2.x library. The game runs unchanged. It loads this
library in place of 3dfx's, and the library draws everything the Voodoo would have drawn with
OpenGL, at any output size (2×, 4×, or a size you name). It was made for POD's 3dfx builds
(`WPOD3DFX.EXE`, first edition and Gold), and then grew to the rest of Glide 2 that games
use. It exports all 156 functions of Glide 2.43 and Glide 2.1.1, under the decorated
`__stdcall` names the games link to (`_grGlideInit@0` and so on, in `glide2x.def`). It
implements all 156 (`api.tsv`, [wrappers](../README.md)).

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

On macOS, Wine gives OpenGL 2.1, so the library asks for a 3.2 core context there and
compiles its shaders as GLSL 1.50. Lines are then one output pixel wide.

Speed in an emulated PC (*measured* in POD, the library logs frames per second): POD's menus
ran at about 5 frames a second at 1×, 2 at 2× and 0.4 at 4×, with Mesa's llvmpipe on an
emulated CPU. `LP_NATIVE_VECTOR_WIDTH=128` made llvmpipe faster under QEMU.

## How it works

| Module | Does |
|---|---|
| `api/` | The exported calls (`sst`, `state`, `tex`, `gutex`, `draw`, `lfb`, `lfbread`, `query`, `mp`, `movie`, `rle`, `pci`). They answer queries and record everything else as commands. `shadow` keeps the state as the game set it, for `grGlideGetState` and for what acts before a triangle is recorded: the origin, culling, the delta-0 colour and the STW hint |
| `vertex.rs`, `layout.rs`, `fog.rs`, `tdf.rs`, `rle.rs` | Plain Rust, tested on the host: culling, polygon fans and clipping; where mip levels lie in texture memory (each level takes at least 2 × 2 texels, as on the Voodoo); the `gu` fog tables; 3DF texture files; the run-length code of `guEncodeRLE16` |
| `cmd.rs` | The commands and the queue. A frame's commands go to the render thread at `grBufferSwap`. A game may call Glide from more than one thread, and an OpenGL context belongs to one |
| `render/` | The render thread: the frame buffer (`frame.rs`, two colour buffers and one depth buffer, like Glide's), the state (`state.rs`), the texture memory (`textures.rs`, decoded with `gameframe-gpu-1997`), the shaders (`shaders.rs`) |
| `window.rs` | The early Glide 2 `grSstOpen` takes no window. So the library finds the game's own window (`WRAPPER_WINDOW_CLASS`, which may still be hidden; without it the first visible window of the process), makes it borderless at the output size, and draws into it. Without a game window it makes one. The later `grSstWinOpen` names the window |
| `shim` (crate) | Port I/O emulated, and patches from `WRAPPER_PATCHES` ([shim](../shim/README.md)) |

The Voodoo's pixel pipeline is one fragment shader. It does the colour and alpha combiners
(`grColorCombine`, `grAlphaCombine` and their `gu` forms), the chroma key, the alpha test,
and table fog (the 64 entries indexed by *w*, as the Voodoo does). It also does the texture
combine for one TMU. Textures are decoded with all their mip levels, so `grTexMipMapMode`
picks or blends levels as the Voodoo did. NCC textures read the table `grTexNCCTable` names,
and multibase textures read each level range from its own address. Depth is the W buffer (or Z buffer), mapped so that Glide's compare functions are
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
- Only TMU 0 draws, as `grSstQueryHardware` reports one TMU. Detail textures
  (`grTexDetailControl`) need a second one and are ignored.
- The `grAADraw*` calls draw without antialiased edges. Nothing is dithered.
- In the W buffer, `grDepthBiasLevel` is added to a linear depth. The Voodoo added it to its
  floating point *w*.
- `GR_COMBINE_LOCAL_DEPTH` in the alpha combiner is taken as the iterated alpha.
- `grSstVRetraceOn` always answers yes, and `grSstPerfStats` counts no pixels.
- The PCI library (`pci*`) shows one Voodoo Graphics at device number 0. Its configuration
  space is made up from the card's usual values. Every map call returns the same zeroed
  16 MB block, and writes there draw nothing. The MTRR calls succeed and do nothing.
- `guMPDrawTriangle` draws the detail and trilinear functions as one pass of texture 0.
  After the passes it puts back the game's texture combine function.
- In Glide 2.43 the `guMovie*` calls only report that the Voodoo cannot record. Here they
  write each frame shown as a 24-bit TGA file (`NAME0000.tga` and on).
- `guEncodeRLE16` stops where the original would read past the image.

## Test

`test/glidetest.c` draws a known scene through the DLL and checks it. It opens 640 × 480 on
its own window, draws, reads pixels back with `grLfbReadRegion` and prints one line per check:
culling, clipping, `gu` and NCC textures, the alpha test, fog, points, an LFB write, saved
state, statistics, a 3DF file, the queries, the PCI library, `guEncodeRLE16`, two textures
blended by `guMPDrawTriangle`, and a frame recorded by `guMovieStart`. Then it animates a mipmapped checker in
perspective. The exit code is the number of failed checks. `tests/run.sh` builds it. Running
it needs Wine or Windows.

```bash
zig cc -target x86-windows-gnu -O2 wrappers/glide/test/glidetest.c -o glidetest.exe -luser32
```

```bash
WINEDLLOVERRIDES=glide2x=n GLIDE_SCALE=2 wine glidetest.exe 120
```

The host tests (`cargo test --release --target <host> -p glide` in `wrappers/`) check the
plain-Rust modules. The fog tables match Glide 2.43's own output byte for byte (*measured*,
the original run under `tools/emu/pe32_emu.py`). So does `guEncodeRLE16`'s code, on 2009
images.

Glide 2.43's own `guEncodeRLE16` has faults, and the port keeps them. Each run takes one
pixel too many from the count, so the code covers fewer pixels than the image has. Pixels of
0x8000 and above never form runs. Its `ConvertAndDownloadRle` calls an empty line decoder,
so it downloads zeros. The port downloads zeros too.

Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
