# gameframe-gpu-1997

The texture and pixel formats of the PC 3D chips of 1997, with the constants their APIs used,
so you can read a game's textures and its API calls without the original SDKs:

- **3dfx Glide 2.x**: every `GR_TEXFMT_*` format, the NCC ("narrow channel compression")
  tables of `YIQ_422` and `AYIQ_8422`, the `GR_LOD_*` and `GR_ASPECT_*` sizes, mip-chain
  sizes and palette entries.
- **ATI 3D Rage, ATI3DCIF**: `C3D_ETF_*` texture formats, `C3D_EPF_*` surface formats,
  `C3D_ERS_*` render-state names and `C3D_EPRIM_*` primitives.
- **S3 ViRGE, S3D Toolkit**: texture and video formats, surface flags, rendering types, alpha
  modes, `S3DTK_STATEKEY` names and the slots of `S3DTK_FUNCTIONLIST`.

One decoder turns any of these texel layouts into 8-bit RGBA. `card` lists what each chip
family accepts, for viewers and tools that show the same scene the way each card would have
drawn it. `resample` halves a texture by point sampling, as loaders of the time did to fit
texture memory. No dependencies and no `unsafe` code; builds for WebAssembly.

It comes from the POD: Planet of Death (1997) project, whose 3dfx, ATI and S3 builds were traced
with it. The game's 16-bit track textures decode with `Rgb565` to the same pictures the 3dfx
build uploads (*observed*: the game hands the file's texels to `grTexDownloadMipMap` unchanged).

## Use it

```toml
[dependencies]
gameframe-gpu-1997 = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_gpu_1997::texel::{decode, Tables};
use gameframe_gpu_1997::{ati, glide, s3};

// A Glide texture: format and size from the game's GrTexInfo.
let format = glide::Format::from_u32(0xa).unwrap(); // GR_TEXFMT_RGB_565
let (w, h) = glide::size(1, 3).unwrap(); // GR_LOD_128, GR_ASPECT_1x1
let texels = vec![0u8; format.texel().bytes(w, h)];
let picture = decode(format.texel(), w, h, &texels, Tables::default()).unwrap();
assert_eq!((picture.width, picture.height), (128, 128));

// Name the state constants a game pushes before ATI3DCIF_ContextSetState and S3DTK_SetState.
assert_eq!(ati::render_state_name(14), Some("SURF_DRAW_PITCH"));
assert_eq!(s3::state_name(10), Some("RENDERINGTYPE"));
```

## What it does not model

- Rasterisation: filtering, dithering, fog, blending and the chips' 16-bit colour pipelines.
  `docs/hardware/graphics-3d.md` lists the effects the player saw.
- YUV 4:2:2 textures (ATI) and ATI's VQ compression: `ati::TexFormat::texel` returns `None`.
- Glide 3, the Voodoo2's multi-unit setup, PowerVR, Rendition and Matrox APIs.
- Alignment that `grTexCalcMemRequired` may add: `glide::chain_bytes` is the data size.

## Check it against your game

1. List the card API's imports (`tools/pe/peinfo.py`) and find the texture upload: the call
   to `grTexDownloadMipMap`, `ATI3DCIF_TextureReg` or the code that fills an `S3DTK_SURFACE`.
2. Read the format constant at the call and the texels it uploads, under the emulator or from
   the game's files.
3. Decode them with the crate and compare with a frame captured from the original
   (`docs/observe/drawing.md`).

## Sources

- 3dfx, Glide 2.x SDK, `glide.h` (format, LOD and aspect enumerations; `GuNccTable`).
- ATI Technologies, *3D RAGE Windows 95 Programmer's Guide*, SDK-C02700 rev. 1.30, 1997:
  `C3D_ERSID` (p. 5-34), `C3D_EPIXFMT` (p. 5-31), `C3D_ETEXFMT` (p. 5-39).
- S3, S3D ToolKit SDK 2.6, `S3DTK.H` (published copy: https://github.com/kytulendu/S3DTK).

## Licence

Apache 2.0, as the rest of GameFrame's code.
