# Numbers, text and libraries

A reimplementation runs on a different CPU, a different compiler and often a different language
from the original. Most of what the original computes comes out the same anyway. A few things
do not. Integer edge cases, floating-point details, the C runtime's own algorithms and the
handling of text all depend on the platform. Each of them can change what the player sees.

These pages list those differences, how to find which behaviour the original has, and how to
get the same behaviour in the port. They add to [Calling conventions and floating
point](../ABI.md), which covers the instructions themselves.

## Pages

- [Integers on every platform](integers.md): division by zero and overflow on each CPU, shift
  counts, saturating and BCD arithmetic, and the same results in Rust, JavaScript, WebAssembly
  and C#.
- [The same floats on every platform](floats.md): what IEEE 754 makes exact, what still differs
  between CPUs, consoles, GPUs and WebAssembly, and how to check it.
- [Library behaviour that shows in results](libraries.md): `rand`, `qsort`, `printf`, `clock`,
  hash tables and other library functions whose results differ between runtimes.
- [Text and encodings](text.md): code pages, Shift-JIS, lengths in bytes and in characters,
  locales, case, sorting, fonts and line breaks.
