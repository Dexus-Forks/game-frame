# s3dtkw: S3D Toolkit (S3 ViRGE)

`s3dtkw.dll` exports all six calls of S3's `S3DTKW.DLL`, as `S3DTKW.dll`. They are the four
that POD's S3 build (`WPODS3D.EXE`) imports, `S3DTK_ExitLib` and `S3DTK_PhysicalToLinear`.
The original's code shows that all six are `__cdecl` (`api.tsv`). The library fills the
function table the game draws through (`SetState`, `GetState`, `TriangleSet`; the other slots
log and return). `TriangleSet` draws lists, strips and fans of `S3DTK_VERTEX_TEX` with
Gouraud colour and source-alpha blending, in software, into the `S3DTK_SURFACE` it is given.
The surface is 15-bit (the only 16-bit format the ViRGE's 3D engine draws) or 8-bit. Physical
addresses are linear ones and video memory starts at 0, so `sfOffset` is the surface's own
address. How POD uses each call is in POD's `docs/hardware/3d-cards.md`.

POD sends only Gouraud faces in a race (the cars' shadows). Textured types are logged and
drawn Gouraud. Fog is logged and not drawn. Both stay open until a game that uses them is
found.

Use it with `ddraw.dll` and `DDRAW_PIXELS=555` (`../README.md`). Log: `S3DTK_LOG`. Build:
`../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
