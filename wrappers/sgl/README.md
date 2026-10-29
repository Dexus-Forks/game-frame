# sgl: PowerVR's SGL on OpenGL

`sgl.dll` stands in for VideoLogic's SGL library for the PowerVR PCX1 and PCX2. The game runs
unchanged. It loads this library in place of SGL's. The library draws with OpenGL at any size.
It was made for POD Gold's PowerVR build (`WPODPVR.EXE`).

SGL has two halves. The scene-graph half takes display lists of transforms, materials,
meshes, lights and cameras, built once. Each frame the game moves transforms, switches lists
on and off and calls `sgl_render`. The immediate-mode half (`sgltri_*`) takes triangles and
quads already projected to the screen, every frame. This library implements both. It has the
46 calls POD imports, under their own names (`sgl.def`, `__cdecl`, undecorated), and 14 for
the immediate mode, which Hardwar (1998) uses. One more, `sgl_wrapper_present`, is for
`ddraw.dll`. It exports all 132 calls of PowerVR driver 3.1.0's `sgl.dll`. It also exports
`sgl_delete_all_textures`, which Hardwar imports from a later SGL. The 73 calls it does not
implement are stubs that log their calls (`api.tsv`, [wrappers](../README.md)). How POD uses
each call, with evidence, is in POD's `docs/hardware/3d-cards.md` ("The PowerVR build").

## The immediate mode

These calls are here: `sgltri_startofframe`, `sgltri_triangles`, `sgltri_quads`,
`sgltri_shadow`, `sgltri_render`, `sgltri_rerender`, `sgltri_isrendercomplete`,
`sgl_use_ddraw_mode`, `sgl_use_eor_callback`, `sgl_get_errors`, `sgl_get_versions`,
`sgl_get_win_versions`, `sgl_get_free_texture_mem` and `sgl_delete_all_textures`.
`sgltri_renderstrip`, which sgl.h says is for VideoLogic's own use, is a stub.

Each face's vertices are `SGLVERTEX`, the layout of Direct3D's `D3DTLVERTEX`: a screen place,
`1/w`, a colour, a specular colour and texture coordinates. The library turns them into the
batches the scene graph's renderer draws, so the textures, the render thread and the output
size are the same for both halves. The details sgl.h leaves open come from PowerVR's own
implementation, SGLMID4.dll of the same driver. `scene/tri.rs` lists each with its function.

The frame reaches the window in one of SGL's two output modes, in the driver's order:

- **Address mode** (`sgl_use_address_mode`). This works as for the scene graph (below).
- **DirectDraw mode** (`sgl_use_ddraw_mode`). SGL owns the screen. At each `sgltri_render` it
  calls the game's 2D callback with the back buffer of the frame just drawn, before the flip.
  The surface is not locked during the call. Then it flips and starts the new frame. The
  library keeps that order. Here the surface is a 16-bit picture of the device's size in this
  library's memory. What the game draws into it goes over the frame, except where it still
  holds the marker.

Shadow and light volumes (`sgltri_shadow`) are drawn. Each call is one convex volume: every
face is a plane, and a pixel is inside when it is inside them all. A face that faces the
camera on the screen is a near side, one that faces away a far side, and a face seen edge-on
is a cut across the screen, as in the driver (`scene/shadow.rs`). So wind each face so that
its normal points out of the volume. After the opaque faces, every pixel is tested against
the planes. Inside a shadow the colour is multiplied by `fShadowBrightness`; inside a light
volume `u32LightVolColour` is added. In those modes a Gouraud face with a translucent map is
drawn flat, as in the driver, which has no smooth translucent format for them.

After each render the end-of-render callback runs, if the game set one. If it returns a value
other than 0, the next `sgltri_render` reports that value as its error. `sgltri_isrendercomplete`
always returns 0 (complete).

## Run

Put `sgl.dll` and this workspace's `ddraw.dll` next to the game's EXE, with Wine told to use
both: `WINEDLLOVERRIDES="sgl=n;ddraw=n,b"`.

| Variable | Meaning | Default |
|---|---|---|
| `SGL_SCALE` | The 3D frame's size as a multiple of the device's (`2`, `4`, `1.5`) | the window's |
| `DDRAW_SCALE` | The window as a multiple of the game's mode ([ddraw](../ddraw/README.md)) | 2 |
| `WRAPPER_WINDOW_CLASS` | The game's window class (`../README.md`). POD's is `Fenetre Pod` | the first visible window |
| `WRAPPER_MARKER` | The marker colour (below) | `0x0821` |
| `SGL_LOG` | A file to log to (`C:\sgl.log`) | standard error |
| `SGL_TRACE` | Log the first N calls of each function, with their arguments | 0 |
| `SGL_SHOT_DIR`, `SGL_SHOT_EVERY` | Save every Nth picture shown as a BMP in that folder | none, 100 |
| `SGL_FOG` | A factor on SGL's fog density (*inferred* scale) | 1 |
| `SGL_PRESENT` | `0`: leave the game's pictures to DirectDraw, so the 3D frame is shown alone | 1 |

SGL's driver installer writes `sglhw.ini` into the Windows folder, with the region size of
the chip's tiles. A game may read it before its first SGL call: POD divides by
`SW_XRegionSize` and `SW_YRegionSize`. The library writes the file with the installer's
settings when it is missing: `[mode] mode=ddraw`, and in `[default]` `SW_XRegionSize=32`,
`SW_YRegionSize=32`, `StrictLocks=0`, `TSPParamSize=256`, `ResetTSPVSync=1` and
`RenderTimeout=5` (`MIDAS.INF`). The driver's own defaults differ (`TSPParamSize` 128,
`RenderTimeout` 2); the library follows the installed driver.

## Test without a game

`test/sgltest.c` draws a known scene through the library: a lit, textured, fogged cube, a
floor, a light volume, a translucent pane and a 2D bar over the marker. Build and run it next
to `sgl.dll`:

```bash
zig cc -target x86-windows-gnu -O2 wrappers/sgl/test/sgltest.c -o sgltest.exe -luser32
WRAPPER_WINDOW_CLASS=SglTest SGL_SHOT_DIR='C:\shots' SGL_SHOT_EVERY=30 SGL_SCALE=2 wine sgltest.exe 120
```

Neither program needs a game. `test/sgltritest.c` draws a known immediate-mode scene: a fogged
floor of quads, a shadow volume and a light volume through the floor, a Gouraud triangle, a
turning textured quad, a flat triangle with a highlight, a translucent triangle and the same
2D bar. Pass `ddraw` after the frame count to use the
DirectDraw mode's 2D callback. At the end it prints what the information calls return.

```bash
zig cc -target x86-windows-gnu -O2 wrappers/sgl/test/sgltritest.c -o sgltritest.exe -luser32
WRAPPER_WINDOW_CLASS=SglTest SGL_SHOT_DIR='C:\shots' SGL_SHOT_EVERY=30 wine sgltritest.exe 120 ddraw
```

The scene graph's tests run on the host (`../README.md`). `tests/run.sh` builds both programs
when zig is there.

## How it works

| Module | Does |
|---|---|
| `scene/` | `tri.rs`: the immediate mode, from the game's contexts and vertices to batches; `shadow.rs`: its shadow and light volumes as planes; `texmem.rs`: free texture memory. The graph (`graph.rs`: lists, transforms, meshes, materials, lights, cameras, textures, one method per call) and the traversal (`walk.rs`: from the root list to lit triangles in the camera's space; `convex.rs`: light volumes) are `gameframe-powervr`'s (`crates/powervr`), which also draws them in software. Plain Rust, tested on the host |
| `api/` | The exported calls: they read the game's pointers and edit the graph |
| `frame.rs` | `sgl_render`: the address callback, the marker, the frame to the render thread; `sgl_wrapper_present` |
| `immediate.rs` | `sgltri_render`: the frame through the address callback or the DirectDraw mode's 2D callback, then the end-of-render callback |
| `render/` | The render thread: OpenGL on the game's window, the off-screen 3D frame, the 2D picture on top |

The PowerVR chip rendered into memory the game named through its address callback
(`sgl_use_address_mode`). POD's callback finishes the frame before: it draws the dashboard
over the last render, shows it, and locks the back buffer for the next. So at the first
`sgl_render` of a frame the library calls the callback, fills the viewport in the memory it
returns with the marker colour, and sends the scene to the render thread. When the game shows
its picture, `ddraw.dll` hands it to `sgl_wrapper_present`. The render thread draws the 3D
frame at the output size and puts every pixel of the picture that is not the marker on top.
With another DirectDraw the 3D frame is shown alone.

## Differences from SGL

Each is marked `Deviation:` in the code.

- Light volumes are drawn, tested per pixel against their planes. A light volume of more than
  64 planes keeps the first 64. Other convex objects of the scene graph (shadow volumes,
  solids drawn from their planes) are skipped.
- A translucent map's clear pixels are cut out. Only a material's opacity blends.
- Faces are lit from both sides. SGL's default material is assumed (white, opaque).
- Point lights are placed from the camera: SGL positions a light with `sgl_position_light`,
  which POD never calls, and POD's car lamps are a spot at (0, 0, 0) along +z (*inferred*).
- A light volume goes into its list where its first plane is added (POD asks for it before the
  transform that places it).
- A texture that no lighting term affects shows unlit (POD's sky); over a texture only the
  specular highlight is added of the terms it does not affect.
- A frame shows under at most three of the game's pictures.
- A 2D pixel of exactly the marker colour counts as not drawn by the game.

In the immediate mode:

- Translucent passes are sorted once for the whole screen. The driver sorts them in each
  32 × 32 region, with that region's own depths. Each call gives one pass for its faces kept
  as sent and one for the faces turned round. The passes are drawn farthest first, and the
  faces in a pass in the order sent, as the driver does.
- Translucency level 15 is drawn fully clear (*inferred*). The driver does not say how clear
  the chip makes it.
- Shadow and light volumes are applied once, after the opaque faces and before the
  translucent ones. The driver applies them after every pass, translucent ones too.
- One brightness and one light colour serve the whole frame: the last a `sgltri_triangles`
  or `sgltri_quads` call gave in that mode. The driver packs them into each face as it is
  sent. Faces sent outside shadow mode are darkened too (*inferred*: the driver leaves
  them).
- `sgltri_shadow`'s bounding box is not used: a volume covers the whole screen. The driver
  files it only into the 32 × 32 regions the box covers.
- A volume of more than 64 planes keeps the first 64 (a call may give 100 faces).
- The light colour is read as a `D3DCOLOR` (0x00RRGGBB). The driver's packer puts its red
  and blue the other way round from the shadow packer's (*measured*); what the chip does with
  that is not known.
- Fog keeps `2^(-c × 2^(R - 22) × w / 0.01)` of a colour, with R the fog register (0 to 31)
  (*inferred* from how SGL sets the register for the scene graph). The chip's constant `c`
  lies between 0.5 and 1 (*inferred*). The library takes 1.0, a person's choice; the chip's
  own value needs a picture from a PCX2. `SGL_FOG` scales it.
- `SGLTT_VERTEXFOG` and the specular alpha are ignored, as the driver ignores them. A face
  fogs by its `w` whenever `bFogOn` is set.
- The DirectDraw mode's 2D callback gets null for both DirectDraw objects. Only the memory is
  real. The surface has the device's size; the driver snaps it to 320 × 240, 640 × 400 or
  480, or 800 × 600.
- Every texture is filtered bilinearly. In the driver, bilinear filtering is one global
  setting (`BilinearFilteringMode` in sglhw.ini); `SGLTT_BILINEAR` is never read per call.
  There is no per-call mipmap flag: mipmapped map types make mipmapped textures. The
  mipmap offset (`SGLTT_MIPMAPOFFSET`) only changes mipmapped textures. The library ignores
  mipmaps and the offset.
- `sgl_get_free_texture_mem` counts as the driver does: 4 MB less two TSP buffers of 256 KB
  (`TSPParamSize` as the installer sets it) and 4 bytes at the start, less a fixed charge for
  each texture by its map type and size (`scene/texmem.rs`). A texture is never refused for
  lack of memory.
- Only the immediate-mode calls and the information calls record errors for
  `sgl_get_errors`.
- `sgl_get_versions` reports driver 3.1.0's strings. `sgl_delete_all_textures` takes no
  arguments (*inferred* from its name).

## Known problems

- The render thread keeps only the latest frame and picture: in software OpenGL a race runs
  at about 3 pictures a second, and every queued picture used to stay in memory.

- Under Wine on Apple silicon POD faults in `wow64cpu` in the menus, with this library or
  without it: know-how entry `wine-wow64-rosetta-fault`. An emulated x86 PC runs it.

Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
