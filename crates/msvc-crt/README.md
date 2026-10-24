# gameframe-msvc-crt

Rust ports of Microsoft C runtime (MSVC CRT) functions that can affect a game's results. The
math, sort and random-number routines are measured against Un-Banished. The new safe memory,
string and integer text routines are source-derived and still need runtime comparison.

| Module | What it reproduces | Why it matters |
|---|---|---|
| `crt` | `sinf`, `cosf`, `acosf` and `atan2f` from the x64 runtime, operation by operation | They are not correctly rounded. Rust's `f32::sin` can differ in the last bit, which can change a simulation |
| `qsort` | Median-of-three quicksort and shortsort for partitions of 8 elements or fewer | Equal elements can change order and affect a game's results |
| `rand` | `rand` and `srand`, with the 32-bit linear congruential state and `RAND_MAX` 32,767 | Games and libraries often use it for random numbers |
| `memory` | Bounded byte versions of `memcpy`, `memmove`, `memset`, `memcmp` and `memchr` | Games use these to copy, clear and compare their state and data |
| `string` | C byte-string comparisons, searches, spans, copies and appends | Names, paths, configuration and resource keys often use C strings |
| `parse` | `strtol`, `strtoul`, `strtoll`, `strtoull` and their common decimal wrappers | Games use them to read configuration, command lines and text data |
| `convert` | `_itoa`, `_ultoa`, `_i64toa` and `_ui64toa` style integer conversion | Games use them to build labels, paths and diagnostic text |

The memory and string modules take byte slices. They check bounds and return offsets rather than
raw pointers. A string slice stops at its first zero byte. If it has no zero byte, its end acts as
an implicit terminator. The case-insensitive functions use ASCII folding. They do not model other
Windows locales or code pages.

The integer parser uses the six whitespace bytes from the C locale. It returns overflow and
invalid-base flags instead of changing the CRT's thread-local `errno`. Windows `long` stays 32 bits
on x64, so `strtol` and `strtoul` return `i32` and `u32`.

It comes from [Un-Banished](https://github.com/tanepiper/unbanished). There the float functions match Banished's copies of the runtime on 200,000 calls each. The sort matches the original's
order of equal elements on recorded sorts (*measured* with GameFrame's emulator).

The added memory, string and integer conversion routines are based on exported functions and
selected decompiles in Microsoft x86 CRT DLLs from 2008, 2010, 2012 and 2013. The `rand` bodies
in those runtimes use the same constants and state update (*observed*). Their `qsort` routines use
the same median-of-three partition and selection-sort helper for 8 or fewer elements (*observed*).
The new Rust routines have passed release-profile compile checks for x86 Windows, x64 macOS and
WebAssembly. They have not yet been compared call by call with each DLL. Check them against the
runtime your game uses before relying on exact edge-case behavior. Local DLLs and Ghidra exports
live in the ignored `analysis/msvc-crt-libraries/` directory.

## Use it

```toml
[dependencies]
gameframe-msvc-crt = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_msvc_crt::{crt, parse, qsort, rand::Rand, string};

let heading = crt::atan2f(1.0, 2.0);
let mut r = Rand::new(1);
let first = r.rand(); // 41
let mut scores = vec![(3, "a"), (1, "b"), (3, "c")];
qsort::qsort_by(&mut scores, |x, y| x.0 - y.0);
let count = parse::strtol(b"  42 units", 10);
assert!(heading > 0.46 && first == 41 && scores[0].0 == 1);
assert_eq!((count.value, count.end), (42, 4));
assert_eq!(string::strcmp(b"game\0", b"game\0"), 0);
```

## Check your game uses these

Different compilers, runtime versions, CPU paths and locales can change CRT behavior. Before you
rely on this crate for exact results, compare it with your game's own copy:

1. Find the game's imported runtime (`msvcrt.dll`, `msvcr*.dll`, `ucrtbase.dll`) or a statically
   linked copy.
2. Run it under GameFrame's emulator on many inputs (`re-emulate`).
3. Compare the results with this crate bit for bit. If they differ, the game uses another version. Trace its algorithm.

The existing math functions reproduce the measured x64 path. The x86 CRT has x87 and SSE2 paths
with different code and results. The byte and string APIs model ASCII C-locale behavior. Locale,
`errno`, raw-pointer faults and undefined overlapping `memcpy` calls are outside their contract.

The crate does not yet model floating-point text conversion, `printf` or `scanf`, allocation,
files, clocks, locale changes, or wide-character functions. Those routines depend on the runtime
version or process environment, so check the game's own library before adding a matching port.

## Tests

```bash
cargo test --release -p gameframe-msvc-crt
```

The tests check plain values, that each function is within one unit in the last place of the
true result, the arctangent table, `rand`'s known first values from seed 1, and that the sort
sorts and is unstable.

## Licence

Apache 2.0.
