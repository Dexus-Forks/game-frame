# gameframe-scale

Scaling of a finished frame by a whole multiple, for ports and replacement libraries that draw
a 320 × 200 or 640 × 480 picture and show it on a modern screen:

- `nearest`: every pixel becomes a k × k block. The picture stays exactly the original's.
- `scale2x`: Andrea Mazzoleni's Scale2x (Eric's Pixel Expansion, EPX), which rounds the
  staircase on diagonal edges and leaves flat areas and straight edges as they are.
  `scale4x` is two passes of it.
- `crt`: nearest, then the last row of every block darkened, for the look of a CRT's
  scanlines.

`nearest` and `scale2x` take any pixel type that can be copied and compared: palette indices
(`u8`), 16-bit pixels (`u16`) or 32-bit RGBA (`u32`). `crt` works on RGBA in a `u32`, with each
8-bit channel scaled. Scale the frame after it is finished, never the game's own buffers, so
the game's drawing stays bit for bit the original's. No dependencies and no `unsafe` code.

It comes from the POD: Planet of Death (1997) project, whose replacement Glide and DirectDraw
libraries show the game's frame scaled this way
([replacement libraries](../../docs/hardware/replacement-libraries.md)).

## Use it

```toml
[dependencies]
gameframe-scale = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_scale::{crt, nearest, scale2x};

// A 2 x 2 frame of palette indices.
let frame = [1u8, 2, 3, 4];
let big = nearest(&frame, 2, 2, 2);
assert_eq!(big, [1, 1, 2, 2, 1, 1, 2, 2, 3, 3, 4, 4, 3, 3, 4, 4]);

// Scale2x keeps a flat frame flat.
assert!(scale2x(&[7u8; 9], 3, 3).iter().all(|&p| p == 7));

// The CRT look: rows 0 and 2 keep their colour, rows 1 and 3 (the last of each block) are darker.
let rgba = crt(&[0xff_80_40_20u32], 1, 1, 2, 50);
assert_eq!(rgba, [0xff_80_40_20, 0xff_80_40_20, 0xff_40_20_10, 0xff_40_20_10]);
```

The output is `w * k` pixels wide, row after row, as the input is `w` wide.
