---
id: x87-one-multiply-two-channels
title: A software mixer packs both stereo gains into one double
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [unicorn]
compiler: [watcom]
hardware: [x86-cpu]
genre: [racing-sports]
symptom: []
tags: [x87, fpu, mixer, sound, magic-number, double-rounding, fnsave, stereo, port, measure]
tools: [pe32_emu]
skills: [re-trace, re-emulate, re-port]
docs: [docs/hardware/sound.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A software mixer packs both stereo gains into one double

## What to know

Hand-written x87 mixers of the mid-1990s can make one `fmul` serve both speakers. POD's
(`FUN_004df88a`, Ubi Soft's sound library) keeps per channel one `double`
`left_gain × 2^32 + right_gain`, multiplies each 16-bit sample by it, adds 3 × 2^65 and stores
the `double`: the mantissa then holds the product in units of 2^14, the right side in bits 0
to 15 and the left in bits 32 to 47. The code reads the two 32-bit halves of the stored value
and adds their low words to the mix with a saturating fix-up on `jo`.

To port it exactly:

- Compute the product in a 128-bit integer (it is exact: under 2^62).
- Round twice, as the x87 does: the `fadd` to the 80-bit format's step at that size (2^3
  here), then the `fstp qword` to the double's (2^14). One rounding is off by one step in a
  few samples (1 case in 400 random blocks in POD).
- A negative right-side value borrows from the left word: a sound panned fully right puts −1
  into the left side. Keep it.
- Check what control word the routine runs under. POD's mixer starts with `fnsave`, which also
  resets the FPU (round to nearest, 64-bit precision), so the caller's mode does not matter.

The loop around it was pipelined by hand (multiply one sample ahead, add the previous
product), so the sample in flight when the block fills is dropped. Read the loop in the
disassembly rather than in the decompiler, and measure it under the emulator on random channel
tables (`measure/sound_mixer.py`).
