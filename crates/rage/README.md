# gameframe-rage

Everything GameFrame knows about the ATI 3D Rage (I, II) and its ATI3DCIF interface: the
texture formats (`C3D_ETEXFMT`) with the texel layout each is, the drawing-surface formats
(`C3D_EPIXFMT`), the render-state ids (`C3D_ERSID`) and their names, and the primitive types.
Texel decoding is `gameframe-texel`'s.

Source: ATI Technologies, *3D RAGE Windows 95 Programmer's Guide*, SDK-C02700 rev. 1.30, 1997:
`C3D_ERSID` (p. 5-34), `C3D_EPIXFMT` (p. 5-31), `C3D_ETEXFMT` (p. 5-39).

Not here yet: the chip's pixel pipeline (GameFrame's `wrappers/raster` draws for the
ATI3DCIF stand-in), YUV 4:2:2 textures and ATI's VQ compression.

```rust
assert_eq!(gameframe_rage::render_state_name(14), Some("SURF_DRAW_PITCH"));
```

Licence: Apache 2.0.
