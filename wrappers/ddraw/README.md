# ddraw: DirectDraw in memory

`ddraw.dll` stands in for DirectDraw for a game that wants an exclusive full-screen mode with
a flip chain, such as 640 × 480 × 16. Every surface here is plain memory. Every mode is
granted. Each flip shows the front buffer in the game's window, scaled. Under Wine on a
display that cannot change mode, the game needs no EXE patch for its display, and the picture
can be any size. It was made for POD's software, MMX, ATI, S3 and PowerVR builds.

With `sgl.dll` loaded in the same process, each picture shown goes to its
`sgl_wrapper_present` first, which lays the picture over SGL's 3D frame ([sgl](../sgl/README.md)).

| Variable | Meaning | Default |
|---|---|---|
| `DDRAW_SCALE` | The window as a multiple of the mode (`2`, `3`, `1.5`) | 2 |
| `DDRAW_FILTER` | How the frame fills the window (below) | `nearest` |
| `DDRAW_PIXELS` | `555` for 5-5-5 16-bit surfaces (a ViRGE draws only 15-bit) | 565 |
| `DDRAW_SYSTEM` | `1`: hand `DirectDrawCreate` to the system's own `ddraw.dll` when a game needs Direct3D, which this DirectDraw does not have | 0 |
| `DDRAW_LOG` | A file to log to | standard error |

Filters (`DDRAW_FILTER`), from the `scalers` crate except the last:

| Filter | Size | Look |
|---|---|---|
| `nearest` | The largest whole multiple that fits the window, centred | Each pixel a sharp block |
| `scale2x` | The same | Scale2x (EPX): stair steps on edges become diagonals, nothing blurred; once per doubling, `nearest` for the rest |
| `crt` | The same | Scanlines (from 2×) and a red, green and blue column mask (from 3×) |
| `smooth` | Any size | GDI's halftone stretch: soft |

The whole-multiple filters run on the CPU and are drawn pixel for pixel.

Implemented: `DirectDrawCreate`, `DirectDrawEnumerateA`; `IDirectDraw` and `IDirectDraw2`
(cooperative level, modes, caps, video memory, surfaces, palettes, clippers);
`IDirectDrawSurface` up to version 3 (lock, unlock, flip, attached surface, `Blt` with colour
fill or copy and source colour key, `BltFast`, colour keys, palettes, descriptions). Every
other method is a stub that removes its arguments and returns `DDERR_UNSUPPORTED`. The log
names it (`IDirectDrawSurface::GetDC`) with its arguments. The DLL also exports these calls
as stubs (`api.tsv`): `DirectDrawCreateClipper`, `DirectDrawEnumerateW`, `DirectDrawCreateEx`,
`DirectDrawEnumerateExA`, `DirectDrawEnumerateExW`, `DllCanUnloadNow` and
`DllGetClassObject`. A game that needs one of them shows it in the log. Modules: `ddraw.rs`,
`surface.rs`, `palette.rs`, `clipper.rs`, `desc.rs` (`DDSURFACEDESC`), `present.rs` (the
frame to the window), `com.rs` (interface tables).

Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
