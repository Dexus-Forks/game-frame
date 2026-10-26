# gameframe-virge

Everything GameFrame knows about the S3 ViRGE and its S3D Toolkit: texture and video formats
with the texel layout each is, surface flags, rendering types, alpha modes, the
`S3DTK_STATEKEY` names and the slots of `S3DTK_FUNCTIONLIST`. Texel decoding is
`gameframe-texel`'s.

Source: S3, S3D ToolKit SDK 2.6, `S3DTK.H` (published copy: https://github.com/kytulendu/S3DTK).

Not here yet: the chip's pixel pipeline (GameFrame's `wrappers/raster` draws for the S3D
Toolkit stand-in).

```rust
assert_eq!(gameframe_virge::state_name(10), Some("RENDERINGTYPE"));
```

Licence: Apache 2.0.
