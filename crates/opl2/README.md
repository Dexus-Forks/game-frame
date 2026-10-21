# gameframe-opl2

An emulator of the Yamaha YM3812 (OPL2), the FM synthesis chip on the AdLib card and the Sound
Blaster family. DOS games play their music by writing to the chip's registers. Give this crate
the same writes at the same times, and it makes the samples the chip would make.

- It runs at the chip's own rate: one sample every 72 master clocks, 3,579,545 / 72 = 49,716
  samples a second.
- It models what the published die and datasheet analyses describe: the log-sine and exponent
  tables, the phase generators with vibrato, the envelope generator with its global counter,
  the four waveforms, feedback, FM and additive connections, rhythm mode, the noise generator,
  and the two timers with the status register.
- Everything is integer and deterministic. The same writes give the same samples on every
  platform, including WebAssembly.
- A band-limited resampler converts the output to 44.1 or 48 kHz without aliasing, and can
  adjust its ratio a little to keep a host's audio buffer steady.
- No dependencies and no `unsafe` code.

It comes from the Frontier: Elite II reimplementation, where it plays the music and effects of
the game's own AdLib driver. Its output is identical sample for sample to the original driver
running on that project's DOS runtime (*measured*).

## Use it

```toml
[dependencies]
gameframe-opl2 = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_opl2::{Opl2, Resampler, SAMPLE_RATE};

let mut opl = Opl2::new();
// A sine tone on channel 0: operator settings, then frequency and key on.
opl.write(0x20, 0x01); // modulator: multiplier 1
opl.write(0x23, 0x01); // carrier: multiplier 1
opl.write(0x40, 0x3f); // modulator silent
opl.write(0x43, 0x00); // carrier at full level
opl.write(0x63, 0xf0); // carrier attack fast, decay 0
opl.write(0x83, 0x00); // carrier sustain at full level
opl.write(0xa0, 0x41); // frequency number, low bits
opl.write(0xb0, 0x32); // key on, block 4, frequency number high bits

let mut chip = vec![0i16; 4_972]; // 0.1 s at the chip's rate
opl.generate(&mut chip);

let mut resampler = Resampler::new(SAMPLE_RATE, 48_000.0);
resampler.push(&chip);
let mut out = Vec::new();
resampler.pull(&mut out); // f32 samples at 48 kHz
assert!(!out.is_empty());
```

## Getting the writes right

The chip is the easy part. The music is only right if the writes reach the chip at the right
times:

1. Capture the original's register writes, with the time of each, from the game running in an
   emulator ([Sound](https://tanepiper.github.io/game-frame/hardware/sound)).
2. Port the game's music driver so it makes the same writes. Compare the two lists write by
   write.
3. Drive this crate from the port's writes, and generate samples between writes to match the
   timing.

The driver's tempo usually comes from a timer interrupt the game programmed. Measure its rate
from the timer programming; a small error changes a song's length.

## Not modelled

- The YM3014 DAC and the card's analogue stage. The output is the chip's 16-bit digital sum.
- CSM speech mode (register 08h bit 7), which games rarely use.
- The OPL3 (YMF262). Its extra channels, waveforms and stereo are not here.

## Tests

```bash
cargo test --release -p gameframe-opl2
```

The tests check the table end points, silence after reset, a sine tone's pitch and peak, the
release to silence after key off, timer 1 and the status register, determinism, and the
resampler.

## Licence

Apache 2.0.
