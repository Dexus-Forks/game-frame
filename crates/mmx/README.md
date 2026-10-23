# gameframe-mmx

The Intel MMX instructions (Pentium with MMX technology, 1997) as plain Rust functions on a
64-bit value. Each function does what the instruction of the same name does, bit for bit:

- wrapping adds and subtracts on bytes, words and doublewords (`paddb` … `psubd`);
- signed and unsigned saturating adds and subtracts (`paddsw`, `paddusb` and the rest);
- the multiplies `pmullw`, `pmulhw` and `pmaddwd`, with `pmaddwd`'s one overflow case;
- compares, packs with saturation, unpacks, logic, and shifts, where a count at or above the
  lane width gives 0 (or the sign, for arithmetic shifts).

Games of 1997 to 1999 used MMX in hand-written assembly: sound mixers, software texture
mappers, alpha blending, screen clears, colour conversion and video decoders. A port written
with this crate gives the same bits as the original, on every platform, WebAssembly included.
No dependencies and no `unsafe` code.

It comes from the POD: Planet of Death (1997) project. There, a model of the game's MMX sound
mixer built from these semantics matched the original routine, run under GameFrame's emulator,
on 500 random channels of 500 (*measured*).

## Use it

```toml
[dependencies]
gameframe-mmx = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_mmx::{paddsw, pmulhw, punpckhwd, punpcklwd, Mm};

// One step of a typical 1997 stereo mixer: four mono samples, a left and a right volume.
let samples = Mm::from_swords([1000, -2000, 3000, -4000]);
let left = pmulhw(samples, Mm::splat_w(0x4000)); // (s * 0x4000) >> 16
let right = pmulhw(samples, Mm::splat_w(0x2000));
let mix = [Mm::default(); 2];
let mix = [paddsw(mix[0], punpcklwd(left, right)), paddsw(mix[1], punpckhwd(left, right))];
assert_eq!(mix[0].swords(), [250, 125, -500, -250]);
```

Port each instruction of the original as one call, keeping the register names as variable
names. Optimise only after the port matches.

## What it does not model

- Timing, pairing and the cost of `emms`: the crate computes values, not cycles.
- The sharing of the MMX registers with the x87 stack. The port must still order its
  floating-point code as the original did, but no value passes between the two.
- 3DNow!, SSE and later extensions. Their float estimates (`pfrcp`, `rcpss`) differ between
  CPU makers and models (`docs/hardware/cpus.md`).

## Check it against your game

1. List the MMX routines: `tools/triage/insns.py GAME.EXE '^p[a-z]+ mm|^emms|^movq|^movd' --functions functions.tsv`.
2. Run one under `tools/emu/pe32_emu.py` on random inputs, as `docs/hardware/simd.md` shows.
3. Run your port on the same inputs and compare the outputs byte for byte.

Look out for scalar code around the MMX: a volume made with an 8-bit `imul`, a shift count
taken from memory. Those are where ports go wrong, not the MMX itself.

## Licence

Apache 2.0, as the rest of GameFrame's code.
