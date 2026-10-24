# sgl: PowerVR's SGL on OpenGL, the scene-graph half

`sgl.dll` stands in for VideoLogic's SGL library for the PowerVR PCX1 and PCX2. The game runs
unchanged. It loads this library in place of SGL's. The library keeps the scene graph the game
builds and draws it with OpenGL at any size. It was made for POD Gold's PowerVR build
(`WPODPVR.EXE`).

SGL has two halves. The scene-graph half takes display lists of transforms, materials,
meshes, lights and cameras, built once; each frame the game moves transforms, switches lists
on and off and calls `sgl_render`. The immediate-mode half (`sgltri_*`) takes lists of
triangles and quads every frame. This library implements the scene-graph half: the 46 calls
POD imports, under their own names (`sgl.def`, `__cdecl`, undecorated), and one more,
`sgl_wrapper_present`, for `ddraw.dll`. How POD uses each call, with evidence, is in POD's
`docs/hardware/3d-cards.md` ("The PowerVR build").

## Not yet here: the immediate-mode half

Hardwar (1998) loads `sgl.dll` at run time for its PowerVR mode and uses the immediate-mode
half. It shares 7 names with POD. These 13 are missing here: `sgl_delete_all_textures`,
`sgl_get_errors`, `sgl_get_free_texture_mem`, `sgl_get_versions`, `sgl_get_win_versions`,
`sgl_use_ddraw_mode`, `sgl_use_eor_callback`, `sgltri_isrendercomplete`, `sgltri_quads`,
`sgltri_render`, `sgltri_shadow`, `sgltri_startofframe` and `sgltri_triangles`
([GAPS.md](../../GAPS.md)). The textures, the render thread and the address-mode frame here
are the parts an immediate-mode stand-in would reuse.

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
32 × 32 when it is missing.

## Test without a game

`test/sgltest.c` draws a known scene through the library: a lit, textured, fogged cube, a
floor, a light volume, a translucent pane and a 2D bar over the marker. Build and run it next
to `sgl.dll`:

```bash
zig cc -target x86-windows-gnu -O2 wrappers/sgl/test/sgltest.c -o sgltest.exe -luser32
WRAPPER_WINDOW_CLASS=SglTest SGL_SHOT_DIR='C:\shots' SGL_SHOT_EVERY=30 SGL_SCALE=2 wine sgltest.exe 120
```

The scene graph's tests run on the host (`../README.md`). `tests/run.sh` builds `sgltest.exe`
when zig is there.

## How it works

| Module | Does |
|---|---|
| `scene/` | The graph (`graph.rs`: lists, transforms, meshes, materials, lights, cameras, textures, one method per call) and the traversal (`walk.rs`: from the root list to lit triangles in the camera's space; `convex.rs`: light volumes). Plain Rust, tested on the host |
| `api/` | The exported calls: they read the game's pointers and edit the graph |
| `frame.rs` | `sgl_render`: the address callback, the marker, the frame to the render thread; `sgl_wrapper_present` |
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
  64 planes keeps the first 64. Other convex objects (shadow volumes, solids drawn from their
  planes) are skipped.
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

## Known problems

- The render thread keeps only the latest frame and picture: in software OpenGL a race runs
  at about 3 pictures a second, and every queued picture used to stay in memory.

- Under Wine on Apple silicon POD faults in `wow64cpu` in the menus, with this library or
  without it: know-how entry `wine-wow64-rosetta-fault`. An emulated x86 PC runs it.

Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
