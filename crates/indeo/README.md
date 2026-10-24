# gameframe-indeo

A decoder for Intel Indeo Video Interactive 4.1 (FourCC `IV41`), written in Rust. It decodes
as Intel's own Video for Windows codec `IR41_32.DLL` (R4.11.15.91) does on its Pentium path,
picture for picture: the same bytes in Intel's `IF09` (YVU9) layout.

- No dependencies. Plain Rust: it builds for any host and for 32-bit Windows.
- **Clean room.** Every rule comes from Intel's own `IR41_32.DLL`, read in its code and run
  under GameFrame's emulator (`tools/emu/pe32_emu.py`). No other decoder's source, data or
  description of its internals was read. The work and its evidence are in the
  [POD project](https://github.com/tanepiper/pod-planet-of-death) (`docs/formats/video.md`,
  "The pictures"), where every rule names the address in Intel's DLL it comes from.
- **Checked against Intel.** On all 13,663 pictures of POD Gold's six videos, each picture is
  byte-identical to the one Intel's codec gives under the emulator (*measured*, 2026-10-24).
- **Intel's tables are not in this crate.** The codebooks, run/value maps, scan orders and
  quantiser matrices are Intel's data. `Tables::from_dll` reads them from the bytes of the
  player's own copy of Intel's DLL (it ships on many game discs, often in an `INSTALL` or
  `INDEO` folder). `Tables::from_parts` takes tables of one's own, for streams made with them.

## Use it

```rust
let tables = indeo::Tables::from_dll(&std::fs::read("IR41_32.DLL")?)?;
let mut decoder = indeo::Decoder::new(tables);
for chunk in video_chunks {               // each video chunk of the AVI's movi list
    if let Some(frame) = decoder.decode(chunk)? {
        // frame.y, frame.v, frame.u: the planes; frame.rgba(): RGBA pixels
    }
    // None: no new picture for this chunk (a null or held-back one); the last one stays.
}
```

A chunk can hold two pictures (an intra picture and the predicted picture that follows it);
`decode` returns the one Intel's codec shows for the chunk.

## What it covers

Everything POD Gold's videos use: intra, predicted, bidirectional and droppable pictures,
pictures held back for later display and null pictures; one luma band or four (the Haar
recomposition); Slant 8 × 8 and 4 × 4, Haar 8 × 8, 1 × 8 and 8 × 1, and no transform; motion
in whole and half samples, forward, backward and both. Not yet: custom scans and quantiser
matrices sent in the stream, transparency, locked (keyed) pictures, the other transforms of
Intel's table (each gives `Error::Picture`). Intel's MMX path recomposes four bands slightly
differently; this crate follows the Pentium path.

## Test

```bash
cd crates && cargo test -p gameframe-indeo
```

The tests decode streams made in the test with tables made in the test (`tests/streams.rs`):
no Intel file is read.

Licence: Apache-2.0 (code), CC0-1.0 (this text).
