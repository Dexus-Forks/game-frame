# MMX and other SIMD code in games

Intel announced the Pentium with MMX technology on 8 January 1997, with 57 new instructions
that work on 64-bit packed integers. For two years "MMX-enhanced" sold PCs and games, and many
games of 1997 to 1999 shipped an MMX build or an MMX code path beside the plain one. AMD's
3DNow! (1998) and Intel's SSE (1999) followed with packed floats. This page is about finding
that code, matching it to the plain version, measuring both, and porting it exactly.
[CPUs and floating point](cpus.md) covers the instruction sets themselves; this page is the
method.

## Where the SIMD code is

Compilers of 1997 did not write MMX instructions. Every MMX routine of the period is
hand-written assembly, usually a few modules linked into the game, or a separate build of the
whole program. Look for both:

- **Separate builds.** A disc with `GAME.EXE` and `GAMEMMX.EXE`, or one build per 3D card each
  with an MMX twin. They are linked on the same day and share almost all their code.
- **Code paths in one build.** A `CPUID` test at start-up (`tools/triage/insns.py GAME.EXE
  '^cpuid'`) that sets a flag or fills a table of function pointers.

List the instructions:

```bash
python3 tools/triage/insns.py GAME.EXE '^p[a-z]+ mm|^emms|^movq|^movd' --functions analysis/export/GAME.EXE/functions.tsv
```

Count them per function. A handful of hits in a plain build are stray decodes of data; a few
hundred in one range of addresses are the SIMD modules. Every MMX routine ends with `emms`
before the program uses the FPU again, which is a quick way to find each routine's end.

## What the routines do

The same jobs come up in most games. The instructions they use tell them apart:

| Job | Typical instructions | What to look for |
|---|---|---|
| Sound mixing | `pmulhw`, `punpcklwd`, `paddsw` | A loop over channels; samples times a volume, interleaved into stereo, added with saturation |
| Sound effects | `paddsw`, `psubsw`, `psraw`, `pmaddwd` | A ring buffer with a read offset behind the write offset (echo, reverb, flanger); a run of `pmaddwd` (a filter) |
| Screen clears and fills | `movq` stores only | 8 or 16 `movq` per loop to the frame buffer |
| Coverage or occlusion masks | `psrlq`, `pxor`, `por` on all ones | A bit per pixel; a span turned into a run of set bits |
| Textured spans | `movq` to an 8-byte-aligned destination, `paddw`, `pmulhw`, `punpck*` | The address rounded with `and reg, ~7`; four 16-bit pixels per store |
| Blending and lighting | `pmullw`, `pmulhw`, `paddusb`, `psrlw` | Two pixels unpacked to words, multiplied, packed back with `packuswb` |
| Colour conversion | `pand`, `psrlw`, `psllw`, `por` | Masks such as `0xf800` and `0x07e0` (RGB565) |
| Video decoding | `pmaddwd`, `packsswb`, `paddsw` | An 8 × 8 transform; a codec library linked in |

## Match each routine to its plain twin

GameFrame's `tools/triage/match_functions.py` matches the C code of two builds well, but the
assembly modules differ in size and shape, so it matches few of them. Match them through their
callers instead:

1. Find the C function that calls the SIMD routine (the caller list in the export).
2. Find the same C function in the plain build. `match_functions.py` usually finds it; if not,
   search for its constants: a dispatch on the same values (`ebx & 99`), the same strings, the
   same table addresses relative to each other.
3. The plain build calls its own routine at the same place in that function. That is the twin.

Expect the structure to differ. An MMX build may split one plain routine into several (one per
channel kind), or add work the plain build does not do. Record each pair in `labels/names.tsv`
with how it was matched (same caller, same dispatch value, match by calls).

## Measure: are they the same?

Run both routines under the emulator on the same random inputs and compare. Most SIMD
routines take their inputs in registers (`esi`, `edi`, `ecx`) or as stack arguments, and work on
buffers you can set up in emulated memory. `tools/emu/pe32_emu.py` runs them as they are:

```python
from pe32_emu import Emu32
plain, simd = Emu32('GAME.EXE'), Emu32('GAMEMMX.EXE')
for trial in range(300):
    for emu, addr in ((plain, PLAIN_ROUTINE), (simd, SIMD_ROUTINE)):
        emu.reset()
        buf = emu.alloc(size)
        emu.write(buf, same_random_input)
        emu.call(addr, esi=buf, edx=rows)
        # read and compare what the game uses afterwards
```

Compare what the rest of the game reads, not the raw bytes. Two builds can lay out an internal
buffer differently and still give the same results: a coverage mask kept in 32-bit words in
one build and 64-bit words in the other numbers its bits differently, and each build's clear
routine reads its own layout. Decode each layout into the same meaning before you compare.

The usual results:

- **Integer work** (masks, clears, copies, conversions) is usually identical. Measure it to
  be sure, then port either version.
- **Arithmetic** often differs. A plain mixer may scale with the x87 FPU and a `double` volume
  while the MMX mixer uses a 16-bit `pmulhw`; the samples then differ in the low bits. Pick one
  build as the reference ([CPUs](cpus.md), "Games with several paths") and record it.
- **Rendering** differs in rounding at span ends and in the order texture coordinates are
  stepped. Compare frames with a tolerance, or emulate the span routine on set-up memory and
  compare pixel for pixel.

## Port it exactly

Write the SIMD routine as one function call per instruction with `gameframe-mmx`
([crates](../crates/README.md)), keeping the register names as variable names, and test it
against the emulator on random inputs. Then make it fast, and test again.

The bugs are rarely in the SIMD instructions. They are in the scalar code that feeds them:

- **Byte multiplies.** `imul bl` multiplies two signed bytes. A pan or volume of 128 read
  through it is −128, which flips the sign of the result. Model the 8-bit operands as the code
  has them.
- **Shift counts in registers.** `psrlq mm2, mm3` shifts by the whole 64-bit value of `mm3`.
  A count of 64 or more gives 0, which hand-written code relies on to build "all ones above
  bit n" masks. A port using native shifts must clamp the count.
- **`pmulhw` rounds toward minus infinity**, because it keeps the high half of the product:
  `−1 × 1 >> 16` is −1, not 0.
- **Saturation happens per lane and per step.** Adding three sources with two `paddsw` clips
  after each add; one wide add followed by one clip gives a different answer.
- **Fixed-point positions.** Sample and texture positions are usually 8.8 or 16.16 fixed
  point, and the routine shifts them to an index. Keep the fraction bits.

## 3DNow! and SSE

The same method applies. The difference is that their float results are not always exact:
the reciprocal and square-root estimates (`pfrcp`, `pfrsqrt`, `rcpss`, `rsqrtss`) differ between
CPU makers and models. If game rules depend on them, record which CPU the original ran on and
emulate that CPU's estimate ([CPUs](cpus.md)).

## Checklist

1. List the builds and code paths, and how the game picks one (file name, `CPUID`, an option).
2. List the SIMD routines per build and group them by job.
3. Match each to its plain twin through the callers.
4. Measure the pairs on random inputs; compare what the game reads.
5. Choose the reference build for each job where they differ, and write it in the project's
   `AGENTS.md`.
6. Port with `gameframe-mmx`, one call per instruction, and test against the emulator.
