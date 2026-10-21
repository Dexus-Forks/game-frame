# gameframe-turbo-pascal

The parts of the Turbo Pascal and Borland Pascal runtime that a DOS game's results depend on,
rebuilt in Rust so a port gives the same numbers as the original program.

| Item | What it reproduces | Why it matters |
|---|---|---|
| `Real48` | The 6-byte `Real` and the System unit's routines for it: conversion, `*`, `/`, `+`, `-`, compare, `Int`, `Frac`, `Sin`, `Cos`, `Trunc`, `Round` | Programs built without the 8087 option do all `Real` arithmetic in software, with their own rounding. IEEE doubles give different last bits |
| `TurboRandom` | `RandSeed`, `Random`, `Random(range)`, `Randomize` | Games use it for everything random; the order and the values of the draws are behaviour |
| `PascalReader`, `PascalWriter`, `PStr` | Pascal data in memory and in `BlockWrite` files: little-endian integers, one-byte `Boolean`, `Real`, `string[N]` | Records in data files and saves are written field by field in this layout |

It comes from Tyrian 2026, the clean-room reimplementation of Tyrian 2000. There every `Real`
routine here matches the original program on 40,000 random operands, and 90,000 random draws
match bit for bit (*measured* under GameFrame's DOS emulator). `div` is measured only for
divisors of 2.0.

## Use it

```toml
[dependencies]
gameframe-turbo-pascal = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_turbo_pascal::{Real48, TurboRandom};

// Round(x * 0.75) as the game computes it.
let scaled = Real48::from_i32(37).mul(Real48::from_f64(0.75)).round();
assert_eq!(scaled, 28); // 27.75

// The same draws as the original, from the same seed.
let mut rng = TurboRandom::new(0x1234_5678);
let roll = rng.random(6);
assert!(roll < 6);
```

## Check your game uses these

Turbo Pascal 5.5, 6.0 and 7.0 and Borland Pascal 7 share these algorithms, but check your game:

1. Find the System unit's routines in the decompile. With Turbo Debugger symbols,
   `tools/dos/tdinfo.py` names them; otherwise look for the `Real` routines that work on
   `DX:BX:AX` and `CX:SI:DI`.
2. Run them under GameFrame's DOS emulator on many inputs (`re-emulate`, `dos_emu`).
3. Compare with this crate bit for bit.

Programs compiled with `{$N+}` use the 8087 (or its emulator) for `Single`, `Double` and
`Extended`. Those types follow the x87 FPU. This crate does not cover them.

## Tests

```bash
cargo test --release -p gameframe-turbo-pascal
```

The tests check encodings, exact integer arithmetic, `Round` taking halves away from zero, `Int`
and `Frac`, `Sin` and `Cos` against doubles, the generator's formula, and a data round trip. The
bit-for-bit checks against the original runtime need a program compiled with it, so they live in
the game project that has one.

## Licence

Apache 2.0.
