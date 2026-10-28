# gameframe-x87

The x87 floating-point unit's arithmetic, done in integers so that it gives the same bits on
every machine, WebAssembly included. Every `fadd`, `fsub`, `fmul`, `fdiv` and `fsqrt` rounds its
exact result once, to the precision and by the rounding mode of the control word; `fst qword`
and `fst dword` round that again to a `Double` or a `Single`; `fist` and `frndint` round to an
integer by the rounding mode. No dependencies, no `unsafe`, `no_std`.

| Item | What it reproduces |
|---|---|
| `Ext` | A finite 80-bit extended value, with `fadd`, `fsub`, `fmul`, `fdiv`, `fsqrt`, `fchs`, `fabs`, `fcom` |
| `Ext::from_i16/i32/i64`, `from_f32/f64`, `from_bytes` | `fild` and `fld`: exact, denormals included |
| `Ext::to_f64`, `to_f32`, `to_bytes` | `fst`/`fstp` to a `Double`, a `Single` or ten bytes, with denormals and overflow to infinity |
| `Ext::fist_i16/i32/i64`, `frndint`, `round_i32`, `trunc_i32` | Rounding to an integer; out of range, the "integer indefinite" (`0x8000…`) |
| `Control`, `Precision`, `Rounding` | The control word's precision (24, 53 or 64 bits) and rounding (nearest, down, up, toward zero) |

## Why a port needs it

Code compiled for the x87 keeps intermediate results in 80-bit registers with a 64-bit
significand, the default precision, and rounds to a `Double` only when it stores a variable. A
port that computes in `f64` rounds every step to 53 bits instead. Most results agree, but some
differ in the last bit. Once it is truncated, scaled or compared, that bit moves a pixel, takes
the other branch or changes a random draw. A division of two integers already shows it: x87 code
rounds the quotient to 64 bits and then to 53, so it can differ from the `f64` quotient, which is
rounded once.

Compilers that use the x87 this way include Borland Pascal 7 and Delphi with `{$N+}` (the
`Single`, `Double`, `Extended` and `Comp` types; the 6-byte `Real` is software, see
[gameframe-turbo-pascal](../turbo-pascal/README.md)), Borland C++, Watcom C and MSVC before SSE2.

## Use it

```toml
[dependencies]
gameframe-x87 = { git = "https://github.com/tanepiper/game-frame" }
```

Port each instruction as one call, and store where the original stores:

```rust
use gameframe_x87::{Control, Ext};

// Delphi: Scale := Width / Count;  (two LongInts, a Double)
// The code is fild Width; fild Count; fdivp; fstp qword [Scale].
let scale = Ext::from_i32(17616).fdiv(Ext::from_i32(-23007)).to_f64();
assert_ne!(scale, 17616.0 / -23007.0); // rounded twice: the last bit differs from f64

// N := Round(A * B + C);  (Doubles) fld A; fmul B; fadd C; fistp dword [N]
let n = Ext::from_f64(1.25).fmul(Ext::from_f64(2.0)).fadd(Ext::from_f64(0.0)).round_i32();
assert_eq!(n, 2); // 2.5: a half goes to the even neighbour, not away from zero

// A game that loads its own control word: 24 bits, toward zero.
let cw = Control::from_word(0x0c7f);
let third = Ext::from_i32(1).checked_fdiv_with(Ext::from_i32(3), cw).unwrap();
assert_eq!(third.to_f32_with(cw).to_bits(), 0x3eaa_aaaa); // to nearest it would end in ...ab
```

Read the instructions, not the source line: where the code stores a temporary and loads it
again matters. Borland's compilers spill to ten-byte temporaries (exact); MSVC spills to eight
bytes (`fstp qword`, a rounding). A `Double` parameter passed on the stack is rounded when it is
pushed.

## The control word

`fninit` leaves `0x037f`: 64 bits, to nearest. Many programs change it. Delphi's start-up code
loads its `Default8087CW`, `$1332` from Delphi 2 on: 64 bits, to nearest, some exceptions
unmasked. MSVC's runtime on Windows NT starts at 53 bits (`0x027f`). Direct3D sets 24 bits
unless told not to. Games load words such as `0x0c7f` (24 bits, toward zero). Read every `fldcw`
first ([context-x87-control-word](../../docs/knowhow/context-x87-control-word.md)), then pass
the word that is in force with the `_with` methods.

## What it does not model

- Infinities, NaNs and the exceptions that make them: `from_f64` panics on one, `fdiv` panics on a
  zero divisor, `fsqrt` on a negative value (`checked_fdiv_with` and `checked_fsqrt_with` return
  `None`). With the exceptions unmasked, as Delphi sets them, the original stops there too.
- The extended exponent range: results keep their exponent however large or small, where the
  x87 would give an infinity, a denormal or 0. `to_bytes` returns `None` for them. Stores to a
  `Double` or a `Single` do handle overflow and denormals.
- The status word: exception flags, the C1 "rounded up" bit, the stack and its tags.
- Transcendental instructions (`fsin`, `fcos`, `fptan`, `fpatan`, `f2xm1`, `fyl2x`), `fprem`,
  `fscale`, `fxtract`, `fbld` and `fbstp`. Their results differ between FPU generations; measure
  your game's.
- Software x87 emulators. A program run without a coprocessor uses one (Windows 3.x's
  `WIN87EM.DLL`, Borland's emulator linked into a DOS program), and its rounding may differ from
  the hardware's. Nothing here has measured them.

## How it was checked

`tests/vectors.txt` holds 1,920 cases made by `tests/make_vectors.py`: a small hand-assembled
routine runs `faddp`, `fsubp`, `fmulp`, `fdivp`, `fst`, `fist`, `frndint` and `fsqrt` under
GameFrame's 32-bit emulator ([pe32_emu](../../tools/emu/pe32_emu.py), Unicorn) at eight control
words: each precision with nearest, and down, up and toward zero. The operands are drawn to hit
the hard cases: ties at 53 and 24 bits, cancellation, operands far apart, denormal and
overflowing stores, integers and halves. The crate equals the emulator on every value, and on
215,696 values of a 72,000-case file (`--cases 1500 --seed 42`). A model of division without
guard bits, which truncates instead of rounding, fails the same file in the last bit.

The emulator's x87 is QEMU's software FPU. It was not compared with a hardware FPU here.

It comes from the Holiday Island (1997, Delphi 1) port, where it reproduces the original's map
generator and a money graph under the same emulator (*measured*).

## Check it against your game

1. Find the routine's x87 instructions and every `fldcw` that runs before it.
2. Run the routine under GameFrame's emulator on many inputs (`re-emulate`: `pe32_emu` for
   32-bit code, `dos_emu` for 16-bit code that uses real x87 instructions, not emulator
   interrupts).
3. Port it instruction by instruction with this crate and compare the stored bits.

## Tests

```bash
cargo test --release -p gameframe-x87
.venv/bin/python crates/x87/tests/make_vectors.py /tmp/x87.txt --cases 1500 --seed 42
GAMEFRAME_X87_VECTORS=/tmp/x87.txt cargo test --release -p gameframe-x87 -- --nocapture
```

The second and third lines make and check a larger file; they need Unicorn in GameFrame's
Python environment.

## Licence

Apache 2.0.
