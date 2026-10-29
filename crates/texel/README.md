# gameframe-texel

The texel layouts the PC 3D chips of the 1990s used, decoded to 8-bit RGBA, and the 2:1
point-sampled reduction that loaders of the time used to fit texture memory. Nothing here
belongs to one chip: each chip's own crate (`gameframe-voodoo`, `gameframe-rage`,
`gameframe-virge`) maps its API's format numbers onto these layouts and supplies what is its
own, such as 3dfx's NCC table for the YIQ layouts (the `Yiq` trait).

Colour channels narrower than 8 bits widen by bit replication (5 bits `abcde` become
`abcdeabc`), as the chips' own colour paths did (the SST-1 specification's texture format
table, section 5.50, shows it for the 3dfx formats).

No dependencies and no `unsafe` code; builds for WebAssembly.

```rust
use gameframe_texel::{decode, Tables, TexelFormat};

let picture = decode(TexelFormat::Rgb565, 1, 1, &[0x00, 0xf8], Tables::default()).unwrap();
assert_eq!(picture.pixels, [255, 0, 0, 255]);
let half = gameframe_texel::resample::halve_point(&[1, 2, 3, 4], 2, 2, 1);
assert_eq!(half, [1]);
```

Licence: Apache 2.0.
