# The card makers' 3D APIs: Glide, ATI3DCIF, S3D, SGL

In 1996 and 1997 each 3D chip had its own programming interface, and a game that wanted to run
well on several cards shipped one renderer, often one whole executable, per card. This page
covers the three most common of them in detail: 3dfx Glide 2.x, ATI's ATI3DCIF for the
3D Rage, and S3's S3D Toolkit for the ViRGE, with VideoLogic's SGL for the PowerVR in less
detail. It says how to find each in a game, how to read
the calls, how textures reach the card, and what the per-card builds tend to differ in.
[3D accelerators](graphics-3d.md) has the overview of the cards and of Direct3D and OpenGL.
One crate per chip holds the formats and constants named here ([crates](../crates/README.md)):
`gameframe-voodoo` for 3dfx, `gameframe-rage` for ATI, `gameframe-virge` for S3,
`gameframe-powervr` for PowerVR, with the texel
decoder they share in `gameframe-texel`. Each chip crate lists the texel layouts its API takes
(`TEXTURE_FORMATS`), and each also draws: `gameframe-voodoo` runs the whole Voodoo pipeline,
while `gameframe_rage::draw` and `gameframe_virge::draw` draw the untextured, blended faces
the ATI and S3 builds of the period gave their chips. Those two crates share
`gameframe-raster`.

## Find the API

List the imports (`tools/pe/peinfo.py GAME.EXE`):

| Library | API | What the imports look like |
|---|---|---|
| `glide.dll`, `glide2x.dll` | Glide 2.x | Many decorated names: `_grSstOpen@24`, `_grTexDownloadMipMap@16`, `_grDrawTriangle@12` |
| `glide3x.dll` | Glide 3 (1998 on) | `grSstWinOpen`, `grDrawVertexArray`: a different API from Glide 2 |
| `ati3dcif.dll` | ATI3DCIF | A few functions: `ATI3DCIF_Init`, `ContextCreate`, `ContextSetState`, `TextureReg`, `RenderBegin`, `RenderPrimList`, `RenderEnd` |
| `sgl.dll` | PowerVR SGL | Undecorated `__cdecl` names: `sgl_create_list`, `sgl_add_vertices`, `sgl_set_texture_effect`, `sgl_render` |
| `S3DTKW.dll` | S3D Toolkit (Windows) | Four functions: `S3DTK_InitLib`, `S3DTK_CreateRenderer`, `S3DTK_DestroyRenderer`, `S3DTK_LinearToPhysical`; the rest come through a function table |

Games linked these at load time, or loaded them with `LoadLibrary` and `GetProcAddress` so one
executable could run without the card. Search the strings for the DLL names to find the second
kind.

A few imports are a clue in themselves. A Glide build that does not import `ddraw.dll` draws
everything through the Voodoo. An ATI3DCIF build that does not import `ATI3DCIF_TextureReg`
never gives the chip a texture, so its textured surfaces are still drawn in software.

## Read the state calls

All three APIs set the render state through one function and a number:
`ATI3DCIF_ContextSetState(context, id, &value)`, `S3DTK_SetState(renderer, key, value)`, and
Glide's many `gr*` calls with enumerated arguments. In the disassembly the number is a pushed
constant just before the call. Name every call site:

1. List the call sites. For a direct import, take the thunk's address from the export's
   `functions.tsv` and list the calls to it: `tools/triage/insns.py GAME.EXE 'call 0xTHUNK'`. For a function table
   (S3D), find the table's address and the calls through `[table + 4 * slot]`.
2. Read the constant pushed as the id or key at each site.
3. Name it with the crate: `ati::render_state_name(id)`, `s3::state_name(key)`,
   `glide::Format::from_u32(v)`. The S3D function table's slots are in `s3::FUNCTIONS`:
   slot 0 is `S3DTK_SetState`, slot 2 `S3DTK_TriangleSet`.
4. Write the list into the topic document: which states the build sets, to which values.

The list says what the card was asked to do. A build that never sets the texture states
(`C3D_ERS_TMAP_EN`, `C3D_ERS_TMAP_SELECT`, `S3DTK_TEXTUREACTIVE`) draws no textures on the chip.
A build that only sets `S3DTK_LITTEXTURE` and `S3DTK_UNLITTEXTURE`, never the `PERSPECT`
types, draws affine textures that swim on large polygons.

## How textures reach the card

### Glide 2.x

A Glide texture is a `GrTexInfo`: smallest and largest mip level (`GR_LOD_256` = 0 down to
`GR_LOD_1` = 8), aspect ratio (`GR_ASPECT_8x1` = 0 to `GR_ASPECT_1x8` = 6, square = 3), format
(`GR_TEXFMT_*`) and a data pointer. The game:

1. asks for the space a texture needs, `grTexCalcMemRequired(smallLod, largeLod, aspect, format)`,
   and places it at an address from `grTexMinAddress(tmu)` upward;
2. uploads it with `grTexDownloadMipMap(tmu, address, evenOdd, &info)`, and a palette or NCC
   table with `grTexDownloadTable(tmu, type, data)`;
3. selects it with `grTexSource(tmu, address, evenOdd, &info)` before drawing.

Games keep these four values per texture in a table of slots. Find it from the arguments of
`grTexSource` and read it: each slot gives a texture's size and format directly
(`glide::size`, `glide::Format`). A Voodoo Graphics card usually had 2 MB of texture memory on its one texture unit (TMU), so
games fitted their textures to it. Look for:

- **Smaller levels for some textures.** A flag per texture page in the data that sets
  `largeLod` to `GR_LOD_128` instead of `GR_LOD_256`, with a loop that halves the page in
  place (every other texel of every other row) before the upload. `resample::halve_point` does
  the same.
- **8-bit textures with a palette** (`GR_TEXFMT_P_8`) for large pages such as fonts, and 16-bit
  RGB565 or ARGB1555/4444 for the rest. A palette entry is `0x00RRGGBB`.
- **NCC textures** (`GR_TEXFMT_YIQ_422`): 8 bits per texel through a table of 16 luminance
  values and 4 + 4 colour offsets (`ncc::NccTable`). Tools of the time compressed 24-bit art to
  it; the table travels with the texture.
- **The number of TMUs.** `grSstQueryHardware` reports them. Some games hard-wire one and leave
  two-TMU paths unused. Check what the code writes before believing a branch is taken.

### ATI3DCIF

A texture is a `C3D_TMAP` with its format (`C3D_ETF_*`: `CI4`, `CI8`, `RGB1555`, `RGB565`,
`RGB8888`, `RGB332`, `Y8`, `YUV422`, `RGB4444`), size and pointers to its levels in
DirectDraw surface memory. `ATI3DCIF_TextureReg` registers it and returns a handle;
`ContextSetState(C3D_ERS_TMAP_SELECT, &handle)` selects it. The drawing surface is set with
`C3D_ERS_SURF_DRAW_PTR` and `C3D_ERS_SURF_DRAW_PITCH`: often the same DirectDraw back buffer
that the game's software renderer draws into. That lets a build mix the two, the chip for some
faces and the CPU for others.

### S3D Toolkit

`S3DTK_CreateRenderer` fills a `S3DTK_FUNCTIONLIST` with function pointers; all drawing goes
through it. A texture is a `S3DTK_SURFACE` (`sfOffset`, `sfWidth`, `sfHeight`, `sfFormat`) in
video memory. `sfFormat` holds `S3DTK_TEXARGB8888`, `TEXARGB4444`, `TEXARGB1555` or
`TEXPALETTIZED8`, with the `S3DTK_TEXTURE` flag. There is no RGB565 texture format. Games turn
linear DirectDraw addresses into the chip's offsets with `S3DTK_LinearToPhysical`. Selecting a
texture is `SetState(S3DTK_TEXTUREACTIVE, &surface)`; the drawing kind is
`SetState(S3DTK_RENDERINGTYPE, …)`.

### PowerVR SGL

VideoLogic's SGL (`sgl.dll`, version 3.1 in 1997) is a scene graph, not a triangle API. The
game builds display lists of transforms, materials, meshes, lights and cameras once
(`sgl_create_list`, `sgl_create_mesh`, `sgl_add_vertices`, `sgl_set_texture_map`), and each
frame only edits a few of them (`sgl_modify_transform`, `sgl_set_ignore_list`,
`sgl_set_point_light`) and calls `sgl_render(viewport, camera, swap)`. A texture is made from
an "intermediate map" of RGBA bytes, alpha 0 for opaque, by `sgl_preprocess_texture` and
`sgl_create_texture` (`sgl_map_16bit`, `sgl_map_trans16`, sizes 32 to 256). The finished
picture goes back to the game through an address callback (`sgl_use_address_mode`), which
hands over the DirectDraw surface's memory. Find the game's scene builder from the first
`sgl_modify_list(SGL_DEFAULT_LIST, clear)`: what follows is the scene, in order.

`gameframe-powervr` keeps the same graph and draws it in software, with the PCX2's per-pixel
rules (the nearest opaque surface per pixel, fog, translucent passes, light volumes) and the
lighting of PowerVR's own driver (`SGLMID4.dll`: the texture effects, the spot light's
fall-off): `Sgl` has the calls by their names. `wrappers/sgl` keeps the same graph and draws it
with OpenGL for the game's own build.

## One game, one build per card

When a game ships a build per card, the builds share most of their code: the game logic, the
file loaders, the sound. Compare them before you trace each:

1. **Match the builds** with `tools/triage/match_functions.py` on their exports, and carry
   your labels from the reference build to the others.
2. **Find which data files each build reads.** A file-name routine often picks an extension
   from a small table by a selector (`.BL3`, `.BL4`, …). Read the jump table and the value each
   build passes. Data in two variants (8-bit and 16-bit textures, say) often holds extra
   fields that only one build reads. A field that the reference build skips is not unused
   until every build has been checked.
3. **Trace the extra fields in the build that reads them.** Find the reader's twin through
   `match_functions.py`, list its reads (`call` sites of the read routine with their sizes),
   and compare them with the reference build's list. The new read, its condition (a global
   set only for some files) and where its value is stored lead to its meaning. Strings next to
   the code that uses the value often name it ("Mirror_Faces").
4. **Write down what each build hands to the chip**: which faces, which textures in which
   format, which states. Builds called "accelerated" sometimes accelerate very little.

## Show what each card drew

The player saw a different picture on each card: 8-bit software textures, 16-bit filtered
textures on a Voodoo, affine textures on a ViRGE, halved texture pages where memory was short.
A viewer or a port can offer the same choice as a setting: a "renderer profile" per card that
uses the file variant, the texture formats and sizes, and the effects (reflections, fog)
that card's build used. each chip crate's `TEXTURE_FORMATS` lists what that chip accepts;
the game's own differences come from tracing its builds. `gameframe-voodoo` draws a 3dfx
profile in software: hand it the triangles and Glide states the game's 3dfx build sends and it
gives the picture at any size, with no Glide library. Its arithmetic follows 3dfx's register
specifications (SST-1 / Voodoo Graphics revision 1.61, Voodoo2 revision 1.16; 3dfx allowed them
to be copied whole and not for profit, so keep them beside the project, not in its
repository): the combine units, the W buffer's floating format, fog and dither are drawn
there as diagrams. Record which profile is the
reference for exact comparisons ([3D accelerators](graphics-3d.md), "How to approach a game
with several renderers").

## Replacing the libraries

To run a card build without its card, or to draw it at a higher resolution, write a library
with the same name and exports: [Replacement libraries for a 1990s game](replacement-libraries.md).

## Sources

- 3dfx, Glide 2.x SDK and `glide.h`.
- ATI Technologies, *3D RAGE Windows 95 Programmer's Guide*, SDK-C02700 rev. 1.30, 1997.
- VideoLogic, PowerVR driver package 3.10 (`sgl.h`, `sglwin32.h`, `SGLMID4.dll`), as shipped
  on POD Gold's disc (`POWERVR/310C3.ZIP`).
- S3, S3D ToolKit SDK 2.6, `S3DTK.H` (published copy: https://github.com/kytulendu/S3DTK).
