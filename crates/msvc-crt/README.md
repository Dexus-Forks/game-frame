# gameframe-msvc-crt

Parts of the Microsoft C runtime (MSVC CRT) that a game's results can depend on, rebuilt in Rust
so a port gives the same numbers as the original program.

| Module | What it reproduces | Why it matters |
|---|---|---|
| `crt` | `sinf`, `cosf`, `acosf`, `atan2f` of the x64 runtime, operation by operation | They are not correctly rounded. Rust's `f32::sin` can differ in the last bit, which can change a simulation |
| `qsort` | `qsort`'s algorithm: median-of-three quicksort with a selection sort for 8 or fewer elements | It is unstable. The order of equal elements is part of the game's behaviour |
| `rand` | `rand` and `srand`: the linear congruential generator with `RAND_MAX` 32,767 | Many games use it for their random numbers, or a library they link uses it |

`powf`, `floorf` and `sqrtf` are correctly rounded in the runtime and in Rust, so Rust's own
functions already match them.

It comes from [Un-Banished](https://github.com/tanepiper/unbanished). There the float functions
match Banished's copies of the runtime on 200,000 calls each, and the sort matches the original's
order of equal elements on recorded sorts (*measured* with the harness's emulator).

## Use it

```toml
[dependencies]
gameframe-msvc-crt = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_msvc_crt::{crt, qsort, rand::Rand};

let heading = crt::atan2f(1.0, 2.0);
let mut r = Rand::new(1);
let first = r.rand(); // 41
let mut scores = vec![(3, "a"), (1, "b"), (3, "c")];
qsort::qsort_by(&mut scores, |x, y| x.0 - y.0);
assert!(heading > 0.46 && first == 41 && scores[0].0 == 1);
```

## Check your game uses these

Different compilers and runtime versions have different maths functions. Before you rely on this
crate, compare it with your game's own copy:

1. Find the game's `sinf` (or the others) in the decompile: the import from `msvcrt.dll` or
   `ucrtbase.dll`, or a statically linked copy.
2. Run it under the harness's emulator on many inputs (`re-emulate`).
3. Compare the results with this crate bit for bit. If they differ, the game uses another
   version, and its algorithm must be traced.

The 32-bit runtime's x87 versions of these functions are different code with different results.

## Tests

```bash
cargo test --release -p gameframe-msvc-crt
```

The tests check plain values, that each function is within one unit in the last place of the
true result, the arctangent table, `rand`'s known first values from seed 1, and that the sort
sorts and is unstable.

## Licence

Apache 2.0.
