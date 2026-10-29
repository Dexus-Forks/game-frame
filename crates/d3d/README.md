# gameframe-d3d

Direct3D 3 immediate mode as games of 1997 drove it, in software. A game's Direct3D build
writes transformed and lit vertices (`D3DTLVERTEX`) and an instruction stream into an execute
buffer and hands it to `IDirect3DDevice::Execute`. A port that has traced what goes into
those buffers can write the same buffers with `Builder` and run them with `Device`, at any
multiple of the game's resolution, with no DirectX and no GPU, natively and in WebAssembly.

What it models:

- execute buffers: `D3DOP_PROCESSVERTICES` (`COPY`), `D3DOP_STATERENDER` and
  `D3DOP_TRIANGLE`, written by `Builder` and read by `exec::parse`, as the documentation's
  structures lay them out. Other opcodes are counted in `Device::skipped`;
- the render states by their `D3DRENDERSTATETYPE` numbers (`consts::rs`);
- the rasterisation rules: pixel centres at integer coordinates and the top-left filling
  convention, so triangles that share an edge draw each pixel once; culling by winding;
- flat shading (the first vertex's colour) and Gouraud shading; textures interpolated
  perspective-correct with `TEXTUREPERSPECTIVE`, everything else in screen space;
- textures in RGB565, ARGB1555 and ARGB4444, nearest or linear, wrapped, mirrored or
  clamped, and a surface's colour key on a card with `D3DPTEXTURECAPS_TRANSPARENCY`;
- the texture blends `DECAL`, `MODULATE`, `DECALALPHA`, `MODULATEALPHA`, `COPY` and `ADD`;
- the specular colour, vertex fog (the specular alpha as the fog factor, with `FOGCOLOR`),
  the alpha test, the stipple pattern, and blending with every `D3DBLEND` factor;
- the depth buffer with every `D3DCMPFUNC`, and an RGB565 target.

The rules come from Microsoft's public documentation of Direct3D immediate mode (the
DirectX 3 and 5 SDK help, and the later pages "Rasterization Rules" and "Directly Mapping
Texels to Pixels"), read as prose. No other implementation's source was read. Where the
documentation leaves a detail open the code says so in a `Deviation:` comment: the stipple
bit order, the stippled-alpha pattern, the defaults DirectX 3 does not list, no dither, no
table fog, no mip levels. Nothing has been compared with a real card yet; real drivers
differed from each other in these details.

```rust
use gameframe_d3d::{Builder, Device, TlVertex, consts::*};
let mut b = Builder::new();
let v = |sx, sy| TlVertex { sx, sy, sz: 0.5, rhw: 1.0, color: 0xffff_0000, ..TlVertex::default() };
let i = [b.vertex(v(0.0, 0.0)), b.vertex(v(8.0, 0.0)), b.vertex(v(0.0, 8.0))];
let n = b.vertex_count() as u32;
b.process_vertices(PROCESS_COPY, 0, 0, n);
b.render_states(&[(rs::CULLMODE, CULL_NONE)]);
b.triangle(i);
let (buf, data) = b.finish();
let mut d = Device::new(16, 16, 2.0);
d.execute(&buf, &data);
assert_eq!(d.colour[2 * 32 + 2], 0xf800);
```

Licence: Apache 2.0 (code), CC0 1.0 (this text).
