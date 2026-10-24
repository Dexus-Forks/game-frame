# ati3dcif: ATI 3D Rage (ATI3DCIF)

`ati3dcif.dll` exports the six ATI3DCIF calls POD's ATI build (`WPODATI.EXE`) imports. It
draws what the game gives the chip in software into the game's DirectDraw back buffer, 16-bit
RGB565, with the `raster` crate. That covers triangles and quads with solid, flat or Gouraud
colour, blended by source alpha. Vertices are `C3D_VTCF`. In a POD race these faces are the
cars' shadows. How POD uses each call is in POD's `docs/hardware/3d-cards.md`.

Not handled (POD does not use them): textures, lines, rectangles, points, Z buffering, fog.
A game that sets those states gets a log line for each. *Unknown*: whether the Rage rounded
pixel edges exactly as `raster` does.

Use it with `ddraw.dll` (`../README.md`), which gives the game its surfaces. Log:
`ATI3DCIF_LOG`. Build: `../README.md`. Licence: Apache-2.0 (code), CC0-1.0 (this text).
