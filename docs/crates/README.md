# Crates

Clean Rust implementations of hardware and runtime pieces that many old games need. Each one
comes from a case-study reimplementation, where it was checked against the original program.
Here they are generic and tested without any game's files. They are free to use under Apache 2.0.

| Crate | What it is | Comes from |
|---|---|---|
| [gameframe-opl2](../../crates/opl2/README.md) | The Yamaha YM3812 (OPL2) FM chip of the AdLib and Sound Blaster cards, at its own 49,716 Hz, with a band-limited resampler | Frontier: Elite II, where its output matches the original AdLib driver sample for sample |
| [gameframe-wma](../../crates/wma/README.md) | A Windows Media Audio v2 decoder for xWMA and ASF streams, with no dependencies, building for WebAssembly | Un-Banished, where it decodes every sound of the game within 60 dB of FFmpeg |
| [gameframe-msvc-crt](../../crates/msvc-crt/README.md) | The Microsoft C runtime's `sinf`, `cosf`, `acosf`, `atan2f`, `qsort` order and `rand`, bit for bit | Un-Banished, matching the game's runtime on 200,000 calls per function |
| [gameframe-mmx](../../crates/mmx/README.md) | The Intel MMX instructions bit for bit: wrapping and saturating packed arithmetic, `pmulhw`, `pmaddwd`, packs, unpacks and shifts | POD: Planet of Death (1997), where a model of the game's MMX mixer built on it matched the original on 500 random channels |
| [gameframe-gpu-1997](../../crates/gpu-1997/README.md) | The texture formats and API constants of 3dfx Glide 2.x (with NCC), ATI3DCIF and the S3D Toolkit, one texel decoder, and what each card accepts | POD: Planet of Death (1997), whose 3dfx, ATI and S3 builds were traced with it |
| [gameframe-scale](../../crates/scale/README.md) | Whole-multiple scaling of a finished frame: nearest, Scale2x (EPX) and a CRT-style look, for any pixel type | POD: Planet of Death (1997), whose replacement Glide and DirectDraw libraries show the game's frame scaled this way |
| [gameframe-turbo-pascal](../../crates/turbo-pascal/README.md) | Turbo Pascal's 6-byte `Real` with its exact arithmetic, `Random`, and Pascal data layouts | Tyrian 2026, matching the original runtime on 40,000 operands per routine |
| [gameframe-indeo](../../crates/indeo/README.md) | An Intel Indeo Video Interactive 4.1 (`IV41`) decoder, clean room, with Intel's tables read from the player's own `IR41_32.DLL` | POD: Planet of Death (1997), whose 13,663 video pictures it decodes byte for byte as Intel's codec does under the emulator |

## Ask your agent

Check a crate against my game. Copy this prompt into your coding agent:

```text
In my GameFrame project, check whether <crate> matches this game.

1. Read the crate's README in GameFrame's crates/ folder.
2. Find the game's own version of the code: the runtime functions, the music driver or the
   decoder.
3. Run the original under GameFrame's emulator on many inputs (the re-emulate skill).
4. Compare the results with the crate bit for bit and report the first difference.
5. If they match, add the crate as a dependency. If not, trace the game's version.
```

## Use them

The crates live in the repository's `crates/` folder, a Cargo workspace. Add one as a git
dependency:

```toml
[dependencies]
gameframe-opl2 = { git = "https://github.com/tanepiper/game-frame" }
```

Each crate's page shows its API, an example, what it does not model, and how to check that it
matches your game.

## When to use a crate, and when to check it first

A crate reproduces one program's version of a chip or a runtime. Your game may use another
version: a later compiler's maths functions, a different Pascal runtime, an OPL3 instead of an
OPL2. Before you rely on a crate for exact results:

1. Find the matching code in your game: the runtime functions, the music driver, the decoder.
2. Run the original on many inputs under GameFrame's emulator (`re-emulate`).
3. Compare the results with the crate bit for bit.

If they match, use the crate. If they differ, trace your game's version and port it.

## Rules for adding a crate

- It comes from a reimplementation where it was checked against the original program. Its README says how it was checked.
- It is generic: no game's names, formats or rules in its code.
- Its tests run without any game's files. Tests that need a tool (such as FFmpeg) skip when the
  tool is missing.
- It has no `unsafe` code, few dependencies, and builds for `wasm32-unknown-unknown`.
- Its README follows [Writing](../working/writing.md): what it is, how to use it, what it does
  not model, how to check it against your game, and its licence.
- Code from another project keeps that project's licence notice (as `gameframe-wma` does for its
  codec tables).
