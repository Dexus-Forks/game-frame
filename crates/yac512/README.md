# gameframe-yac512

A clean Rust model of the Yamaha YAC512's digital serial receiver and channel input latches. It
accepts the YMF262's 16-bit, least-significant-bit-first offset-binary stream, decodes the FSEL
format at each latch, and holds the last captured word for each channel.

The behavior specification comes from Yamaha's [YAC512 datasheet](https://www.quarter-dev.info/archives/yamaha/YAC512.pdf)
and the [YMF262 datasheet](https://bitsavers.trailing-edge.com/components/yamaha/YMF262_199411.pdf).
Yamaha documents that falling SMP1 latches channel 2 and falling SMP2 latches channel 1. The YMF262
stream's bit order, 16-bit word size, offset-binary encoding and timing come from Yamaha's YMF262
datasheet and this [published timing measurement](https://pcm1723.hateblo.jp/entry/20101221/1292958950).
The implementation uses these specifications and does not use source code from another emulator.

## Use it

```toml
[dependencies]
gameframe-yac512 = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_yac512::Yac512;

let mut dac = Yac512::new(); // FSEL high: offset-binary input
for bit in 0x8001_u16.to_le_bytes() {
    for shift in 0..8 {
        dac.clock_rising_edge((bit >> shift) & 1 != 0);
    }
}

// A falling SMP2 edge latches channel 1.
dac.set_sample_pins(false, true);
dac.set_sample_pins(false, false);
let channel1 = dac.channels().channel1.unwrap();
assert_eq!(channel1.raw, 0x8001);
assert_eq!(channel1.signed_input(), 1);
```

`clock_rising_edge()` receives one DIN level per CLK rising edge. `set_sample_pins()` detects
falling edges and captures the current serial shift register. FSEL is offset binary when high and
two's complement when low. The captured word keeps the format that was active at the latch.

## Coverage and limits

- The serial receiver shifts LSB-first input into a 16-bit word.
- SMP1 falling captures channel 2. SMP2 falling captures channel 1. Each channel holds its most
  recent captured digital input word.
- FSEL high decodes offset binary; FSEL low decodes two's complement. `LatchedWord::signed_input()`
  exposes this input-format interpretation.
- The YAC512 converts the word to a floating representation with a 10-bit DAC mantissa and a
  seven-step analog shifter. Yamaha's accessible datasheet describes those blocks but does not give
  the conversion formula. This crate does not invent that formula, simulate analog switch timing,
  model capacitor droop, or produce voltage-level output.
- Board wiring can swap which YAC512 channel receives each YMF262 serial word. Connect the modeled
  strobes to the correct YAC512 pins for the card being represented.
- No comparison with a physical DAC or a named software reference is recorded. Physical hardware
  testing is optional. Suggested software checks are in the [YAC512 open questions](../../docs/hardware/ymf262-open-questions.md#yac512-floating-dac-transfer).
- `cargo test -p gameframe-yac512` checks serial bit order, falling-edge channel mapping, held
  channel values and FSEL capture across latches. These checks cover the digital model only.

## Licence

Apache 2.0.
