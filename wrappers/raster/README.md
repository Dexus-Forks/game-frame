# raster: a small software triangle rasteriser

`raster` draws triangles into a frame buffer of 8-, 15- or 16-bit pixels. Attributes are
interpolated across each triangle. A shader closure sees the pixel already in the buffer, so it
can blend. `ati3dcif` and `s3dtkw` use it to draw what a game gives a 1997 chip into the
game's own DirectDraw surface.

A pixel is drawn when its centre (`x + 0.5`, `y + 0.5`) is inside the triangle. Top and left
edges are in; bottom and right edges are out. Two triangles that share an edge never both
draw a pixel. Attributes are interpolated linearly in screen space (*inferred* for the ATI
Rage and the S3 ViRGE: neither POD build asks for perspective correction). Whether the chips
rounded edges exactly this way is *unknown*.

Tests: `cargo test --release --target HOST -p raster` (`../README.md`). Licence: Apache-2.0
(code), CC0-1.0 (this text).
