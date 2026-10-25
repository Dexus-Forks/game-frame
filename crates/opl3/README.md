# gameframe-opl3

An implementation of Yamaha's YMF262 (OPL3) chip for GameFrame. It builds on the project's
YM3812 operator engine and models both register banks, 18 channels, six selectable
four-operator pairs, eight native waveforms, shared note-select control, four routed output
channels, indexed I/O ports and serial DAC frames.

The behavior specification comes from Yamaha's [YMF262 datasheet, 1994](https://bitsavers.trailing-edge.com/components/yamaha/YMF262_199411.pdf), especially the register map, sample-rate formula, output pins and waveform diagrams. Serial encoding follows Yamaha's [YMF262 application manual](https://www.quarter-dev.info/archives/yamaha/YMF262_Application_Manual.pdf). The 18-clock slots and sample/hold windows follow a [published timing measurement](https://pcm1723.hateblo.jp/entry/20101221/1292958950). The 1994 [OPL3 Programmer's Guide](https://www.asvcorp.ru/darch/docs/sound/opl3/index.html) supplies software-visible status and compatibility behavior. This crate does not use source code from another emulator.

The [AdLib Gold `GSSOPL3.DRV` trace](../../docs/hardware/adlib-gold-gssopl3.md) records the original Windows driver's banked register writes, voice pools and `0x104`/`0x105` initialization path. That binary informs the card-facing register use. It does not provide output-parity evidence for this chip crate.

## Use it

```toml
[dependencies]
gameframe-opl3 = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_opl3::{Opl3, StereoSample, OPL3_SAMPLE_RATE};

let mut opl = Opl3::new();
opl.write(0x105, 0x01); // enable native OPL3 mode
opl.write(0x020, 0x01); // channel 0 modulator frequency multiplier
opl.write(0x023, 0x01); // channel 0 carrier frequency multiplier
opl.write(0x040, 0x3f); // modulator silent
opl.write(0x043, 0x00); // carrier at full level
opl.write(0x060, 0xf0); // fast attack
opl.write(0x063, 0xf0);
opl.write(0x080, 0x0f);
opl.write(0x083, 0x0f);
opl.write(0x0c0, 0x30); // route channel 0 to left and right
opl.write(0x0a0, 0x41);
opl.write(0x0b0, 0x32); // key on

let mut frames = vec![StereoSample::default(); 4_972];
opl.generate(&mut frames);
let _duration_seconds = frames.len() as f64 / OPL3_SAMPLE_RATE;
```

For card-specific output wiring, use the four native output words directly:

```rust
use gameframe_opl3::{Opl3, Opl3Output};

let mut opl = Opl3::new();
opl.write(0x105, 1);
let output = opl.sample_output();
if let Opl3Output::Native(channels) = output {
    let _dac_words = [channels.a, channels.b, channels.c, channels.d];
    let serial_pins = channels.to_serial_frame();
    let _doab_lsb = serial_pins.doab[0];
}
```

`Ymf262DacFrame` describes the four native channel words on DOAB and DOCD, plus SMPAC and SMPBD
over 36 φSY rising-edge slots. The serial words are 16-bit offset binary, LSB first. Each channel
uses an 18-clock slot; the two unspecified data clocks in each slot are `None`. This matches the
YMF262/YAC512 digital connection. The companion [`gameframe-yac512`](../yac512/README.md) crate
models the YAC512's input shift register, FSEL decoding and sample latches. Its analog floating-DAC
conversion and sample-and-hold output remain open.

## Indexed I/O ports

The [AdLib Gold driver trace](../../docs/hardware/adlib-gold-gssopl3.md) shows the driver using
two index/data port pairs. `Ymf262Ports` routes those writes to the chip and returns status from
the primary index port. The secondary index and both data ports are write-only.

```rust
use gameframe_opl3::{StereoSample, Ymf262Ports};

let mut ports = Ymf262Ports::new(0x388);
ports.write_port(0x38a, 0x05); // bank one, register index
ports.write_port(0x38b, 0x01); // enable native OPL3 mode
let _status = ports.read_port(0x388);

let mut audio = vec![StereoSample::default(); 256];
ports.chip_mut().generate(&mut audio);
```

The base port is configurable. `write_port_at()` accepts absolute master-clock timestamps and
rejects writes that arrive less than 32 clocks after the previous write. `write_port()` is an
immediate convenience method for callers that already schedule access timing. Port methods do
not schedule φSY edges; `Ymf262DacFrame` provides the serial words and strobes for one audio frame.

Pass the chip's timestamped register writes in order. The `0x000..=0x0ff` addresses select bank
zero. The `0x100..=0x1ff` addresses select bank one. Register `0x105` bit 0 switches from the
YM3812-compatible startup mode to native OPL3 mode. In native mode, channel registers `C0h`–`C8h`
route each voice to output A, B, C and D with bits 4–7. A channel with no output bit set is
silent. While native mode is off, writes to the second register array and `0x104` are ignored.
Register `0x104` selects the six four-operator pairs. The two `CNT` bits in each paired channel
register select one of Yamaha's four connection algorithms.

The native output rate is one frame per 288 master clocks, the same 49,716 Hz rate as the YM3812
core's 3.579545 MHz / 72 clock. `sample_output()` returns either compatibility-mode mono or four
independent native output words. `sample()` is a stereo convenience adapter. It repeats mono in
compatibility mode and folds A+C left, B+D right in native mode. Use `FourChannelSample` when a
card has different DAC or mixer wiring. `generate()` fills a caller-owned stereo buffer;
`generate_outputs()` fills a caller-owned native-output buffer.

## Chip coverage and limits

- Both register banks, all 18 two-operator channels, all six selectable four-operator pairs,
  four connection algorithms, eight waveforms, OPL2 compatibility mode, shared note-select,
  timers and status, four output routing channels, and indexed port selection are modeled.
- The YMF262 ignores register `0x01`'s YM3812 waveform-select-enable bit. Compatibility mode
  selects waveforms 0–3; native mode selects all eight. Its status register has zeroes in bits
  0–4 in both modes, so a mode switch does not change the status signature.
- Internal LSI test-register behavior is ignored.
- The four-operator pair map and algorithms follow Yamaha's datasheet. The AdLib Gold Windows
  driver confirms real use of the `0x104` mask and separate 18-slot and 6-slot voice pools.
- In four-operator mode, the pair currently produces one combined sample routed with the first
  channel's selector. Yamaha documents both connection registers but does not clearly specify
  whether their output selectors route separate carrier groups. This behavior remains unconfirmed;
  see the [software reference test plan](../../docs/hardware/ymf262-open-questions.md).
- YMF262 applies bank-zero register `0x08` note-select to both operator banks. It ignores bit 7:
  the YMF262 omits the YM3812's CSM mode.
- Waveforms 4–7 follow the datasheet diagrams. Square and sawtooth use ideal digital levels;
  their internal YMF262 quantization has not been measured against hardware.
- Output values are 16-bit sample words for channels A–D. The DOAB/DOCD serial data and SMPAC/SMPBD
  sample-and-hold timing are modeled as φSY rising-edge slots. Pin transition delays, YAC512 analog
  conversion, analog sample-and-hold circuitry and card-level wiring remain outside these crates.
- `write_port_at()` enforces the datasheet's minimum 32 master clocks between writes when the
  caller supplies timestamps. The immediate `write_port()` method does not. Neither method
  models CPU bus-cycle costs. The crate also does not model an AdLib Gold control chip or Sound
  Blaster DSP, DMA, IRQ and mixer.
- `cargo test -p gameframe-opl3` checks native output routing and the mode-gated four-operator
  register. These software tests do not establish output parity with a physical chip or a named
  reference.
- Timer counters advance on the sampled chip clock; sub-sample timer and operator timing is not
  modeled.
- No register or output parity capture against a physical YMF262 or named software reference is
  recorded yet. This evidence gap is not a release blocker; physical hardware testing is optional.
  The existing `gameframe-opl2` core has separate sample parity evidence for the YM3812.

## Licence

Apache 2.0.
