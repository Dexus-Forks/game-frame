# Crates

These crates provide generic Rust implementations of hardware and runtime pieces used by games.
Each one comes from case-study code or source inspection. Its README says what was compared with an
original and what remains open. Its tests use no game's files. All crates are Apache 2.0.

| Crate | What it is | Comes from |
|---|---|---|
| [gameframe-opl2](../../crates/opl2/README.md) | The Yamaha YM3812 (OPL2) FM chip of the AdLib and Sound Blaster cards, at its own 49,716 Hz, with a band-limited resampler | Frontier: Elite II, where its output matches the original AdLib driver sample for sample |
| [gameframe-opl3](../../crates/opl3/README.md) | Yamaha YMF262 with 18 channels, six four-operator pairs, eight waveforms, four output words, timers, compatibility mode, indexed ports and serial DAC frames | Extends GameFrame's OPL2 core from Yamaha's datasheet and an original AdLib Gold driver trace; independent four-operator carrier routing, YAC512 analog conversion and card adapters remain open |
| [gameframe-saa1099](../../crates/saa1099/README.md) | Philips SAA1099 stereo sound generator and the two-chip Creative Music System/Game Blaster port model | Philips's data handbook and static traces of the original CMS and Sierra drivers; noise taps, tone/noise mixing, analog output and card detection remain open |
| [gameframe-sn76496](../../crates/sn76496/README.md) | TI SN76496 three-tone and noise generator, PCjr and original Tandy port adapters and audio muxes | TI and machine documentation plus static traces of Sierra's `JR.DRV` and `TANDY.DRV`; TI noise feedback, the Tandy mux's fourth input, NCR/PSSJ variants, CPU wait mapping and analog output remain open |
| [gameframe-yac512](../../crates/yac512/README.md) | Yamaha YAC512 serial input, FSEL format decoding and two channel latches | Yamaha's YAC512 and YMF262 datasheets plus a published YMF262 serial timing measurement; the analog transfer curve remains open |
| [gameframe-ymz263](../../crates/ymz263/README.md) | Yamaha YMZ263B/MMA dual PCM/ADPCM channels, FIFOs, timers, DMA request state, MIDI FIFOs and UART timing | Yamaha data sheet, Ad Lib Gold Developer Toolkit v1.01 and static GSS Wave/MIDI driver trace; exact ADPCM rounding, interleave and some status semantics need software comparisons |
| [gameframe-adlib-gold](../../crates/adlib-gold/README.md) | AdLib Gold/GSS YMF262 and YMZ263 ports, Control Chip register access, busy status, EEPROM commands, audio-port relocation and linear source mixing | Ad Lib Gold Developer Toolkit v1.01 and original OPL3/Wave-MIDI driver traces; TDA8425 processing, analog filters, external DMA/IRQ and revision defaults remain open |
| [gameframe-sound-blaster](../../crates/sound-blaster/README.md) | Creative Sound Blaster DSP generations 1.xx–4.xx, generation-specific mixers, DSP MIDI and SB16 MPU-401 UART | Creative's hardware programming reference and a static CT1741 firmware 4.05 trace; existing software tests cover the port model, 8/16-bit DMA paths and IRQ behavior. ADPCM sample parity and machine-level DMA/PIC remain open |
| [gameframe-wma](../../crates/wma/README.md) | A Windows Media Audio v2 decoder for xWMA and ASF streams, with no dependencies, building for WebAssembly | Un-Banished, where it decodes every sound of the game within 60 dB of FFmpeg |
| [gameframe-msvc-crt](../../crates/msvc-crt/README.md) | MSVC math, `qsort` and `rand`, plus safe byte, string and integer text routines | Un-Banished's runtime math and sort; the added x86 CRT routines are source-inspected and await runtime comparison |
| [gameframe-mmx](../../crates/mmx/README.md) | The Intel MMX instructions bit for bit: wrapping and saturating packed arithmetic, `pmulhw`, `pmaddwd`, packs, unpacks and shifts | POD: Planet of Death (1997), where a model of the game's MMX mixer built on it matched the original on 500 random channels |
| [gameframe-gpu-1997](../../crates/gpu-1997/README.md) | The texture formats and API constants of 3dfx Glide 2.x (with NCC), ATI3DCIF and the S3D Toolkit, one texel decoder, and what each card accepts | POD: Planet of Death (1997), whose 3dfx, ATI and S3 builds were traced with it |
| [gameframe-scale](../../crates/scale/README.md) | Whole-multiple scaling of a finished frame: nearest, Scale2x (EPX) and a CRT-style look, for any pixel type | POD: Planet of Death (1997), whose replacement Glide and DirectDraw libraries show the game's frame scaled this way |
| [gameframe-turbo-pascal](../../crates/turbo-pascal/README.md) | Turbo Pascal's 6-byte `Real` with its exact arithmetic, `Random`, and Pascal data layouts | Tyrian 2026, matching the original runtime on 40,000 operands per routine |
| [gameframe-x87](../../crates/x87/README.md) | The x87 FPU's 80-bit arithmetic in integers: `fadd` to `fsqrt` under any precision and rounding control, `Double` and `Single` stores, `fist` and `frndint`, for Borland Pascal and Delphi `{$N+}`, Watcom or MSVC code | Holiday Island (1997, Delphi 1); equal to real x87 instructions under GameFrame's emulator on 215,696 values |
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
matches your game. The [PC audio coverage map](../hardware/audio-coverage.md) records hardware
gaps and external projects that may provide black-box comparison output. The current code
license is Apache-2.0. The owner may relicense owned work, including a crate as
`Apache-2.0 OR MIT`, and should record the selected scope in that crate's metadata and README.

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
