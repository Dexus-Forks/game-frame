# gameframe-raster

A small software triangle rasteriser. It draws triangles and convex polygons (fans) into a
frame buffer of 8-, 15- or 16-bit pixels, with up to eight attributes interpolated across
each triangle. A shader closure sees the pixel already in the buffer, so it can blend. The
chip models `gameframe-rage` (ATI 3D Rage) and `gameframe-virge` (S3 ViRGE) draw with it,
and so do the replacement DLLs in `wrappers/` that stand in for those chips.

A pixel is drawn when its centre (`x + 0.5`, `y + 0.5`) is inside the triangle. Top and left
edges are in; bottom and right edges are out. Two triangles that share an edge never both
draw a pixel. Attributes are interpolated linearly in screen space (*inferred* for the ATI
Rage and the S3 ViRGE: neither POD build asks for perspective correction). Whether the chips
rounded edges exactly this way is *unknown*.

```rust
use gameframe_raster::{Format, Target, Vertex, polygon};
let mut buf = vec![0u8; 8 * 8 * 2];
let mut t = Target::new(&mut buf, 16, 8, 8, Format::Rgb565);
let v = |x, y| Vertex { x, y, a: [0.0; 8] };
polygon(&mut t, &[v(2.0, 2.0), v(6.0, 2.0), v(6.0, 6.0), v(2.0, 6.0)], &mut |_, _| Some(0xffff));
assert_eq!(t.get(2, 2), 0xffff);
```

Licence: Apache 2.0 (code), CC0 1.0 (this text).
