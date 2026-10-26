# gameframe-virge

Everything GameFrame knows about the S3 ViRGE and its S3D Toolkit: texture and video formats
with the texel layout each is, surface flags, rendering types, alpha modes, the
`S3DTK_STATEKEY` names and the slots of `S3DTK_FUNCTIONLIST`. Texel decoding is
`gameframe-texel`'s.

Source: S3, S3D ToolKit SDK 2.6, `S3DTK.H` (published copy: https://github.com/kytulendu/S3DTK).

`draw` is the chip's drawing as the toolkit drives it: a `State` holding the
`S3DTK_SetState` values, the 32-byte `S3DTK_VERTEX_TEX` record, and `triangle_set` drawing
lists, strips and fans (Gouraud, with source-alpha blending) into an 8- or 15-bit surface
through `gameframe-raster`. The `wrappers/s3dtkw` DLL and Un-POD's S3 renderer both draw
with it.

Not here yet: texturing, fog and the Z buffer.

```rust
assert_eq!(gameframe_virge::state_name(10), Some("RENDERINGTYPE"));
```

Licence: Apache 2.0.
