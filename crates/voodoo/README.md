# gameframe-voodoo

A software model of the 3dfx Voodoo Graphics pixel pipeline as Glide 2.x drives it. A port
that has traced which triangles and states a game's 3dfx build hands to Glide can draw them
here, at any resolution, with no Glide library and no GPU, natively and in WebAssembly.

What it models, in the order of the Glide 2.x reference manual's pipeline:

- one TMU: texture sampling, point or bilinear, wrapped or clamped, perspective-correct
  (`sow`, `tow` over `oow`); `guTexCombineFunction` decal, zero and one;
- the colour and alpha combine units (`grColorCombine`, `grAlphaCombine`, and
  `guColorCombineFunction` as the calls it stands for);
- the chroma key, the alpha test, iterated-alpha and table fog;
- the Z buffer (`ooz`) and the W buffer (`1/oow`), with every compare function and the
  depth mask;
- alpha blending against the pixel already in the buffer;
- the 16-bit RGB565 frame buffer with its 4 × 4 ordered dither, and the gamma curve on the
  way out.

What it does not model: two TMUs, mipmaps, the LFB, alpha and depth bias. Its arithmetic is
float where the chip's is fixed point; each such place says so in a `Deviation:` comment, and
none of it has been compared with a real card yet.

The constants are the values of the Glide 2.x SDK's `glide.h`, as in GameFrame's
`wrappers/glide`, whose OpenGL shader follows the same combine rules. Texel formats come from
`gameframe-gpu-1997`.

It comes from the POD: Planet of Death (1997) project, whose 3dfx build (`WPOD3DFX.EXE`)
Un-POD draws with it.

## Use it

```toml
[dependencies]
gameframe-voodoo = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_voodoo::{consts::*, Combine, Frame, State, Vertex, triangle};

let mut frame = Frame::new(640, 480);
let mut state = State::default();
state.depth_mode = DEPTHBUFFER_WBUFFER;
state.depth_mask = true;
state.colour = Combine::colour_function(CC_ITRGB).unwrap();
frame.clear(0, 0xffff, state.depth_mode);
let v = |x, y| Vertex { x, y, oow: 1.0, r: 255.0, g: 128.0, b: 0.0, a: 255.0, ..Vertex::default() };
triangle(&mut frame, &state, None, [&v(10.0, 10.0), &v(200.0, 20.0), &v(50.0, 300.0)]);
let rgba = frame.rgba(1.0);
assert_eq!(rgba.len(), 640 * 480 * 4);
```

## Build and test

```bash
cargo test --manifest-path crates/Cargo.toml -p gameframe-voodoo
```

Licence: Apache 2.0.
