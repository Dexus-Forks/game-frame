# gameframe-ymz263

A clean, timestamped model of Yamaha's YMZ263B (MMA) used in the AdLib Gold/GSS card. It models
the two PCM/ADPCM FIFOs, sample-rate dividers, register ports, timer counters, DMA request
conditions, MIDI FIFOs and UART transmit timing. The card-level adapter is in
[`gameframe-adlib-gold`](../adlib-gold/README.md).

## Use it

```toml
[dependencies]
gameframe-ymz263 = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_ymz263::Ymz263;

let mut mma = Ymz263::new();
mma.write_port_at(0, 0x0c, 0).unwrap(); // select channel 0 format/control
mma.write_port_at(1, 0x19, 8).unwrap(); // FIFO threshold 16 bytes, DMA enabled
mma.write_port_at(0, 0x0b, 16).unwrap(); // select channel 0 PCM data
for byte in 0..128 {
    mma.write_port_at(1, 0x80, 24 + byte * 8).unwrap(); // fill the FIFO before playback
}
mma.write_port_at(0, 0x09, 1_048).unwrap(); // select channel 0 playback control
mma.write_port_at(1, 0x65, 1_056).unwrap(); // PCM, GO, left and right outputs
mma.advance_to(2_000).unwrap();
let _samples = mma.drain_output_events();
```

Pass monotonically increasing YMZ263B master-clock timestamps. Port accesses enforce the minimum
eight-clock interval derived from the toolkit's 470 ns timing. The four relative ports model the
two index/data pairs used by the original GSS Windows driver. PCM output events preserve each
channel's signed 12-bit sample, volume register and left/right routing bits. The host card model
can consume those events and apply the card mixer.

## Evidence and modeled behavior

The register map and supported modes follow Ad Lib's [Developer Toolkit v1.01](https://slartibardfast.github.io/adlib_gold/disks-md/developer-toolkit-v1.01/installed/SDTK.DOC.html), Hardware Reference
4.3, and Yamaha's [YMZ263B data sheet](https://www.quarter-dev.info/archives/yamaha/YMZ263B.pdf). The `GSS.DRV` v1.01 Windows NE driver confirms writes to
two index/data pairs at offsets `0/1` and `2/3`, use of registers `0x09`, `0x0c` and `0x0d`, and
timer setup through registers `0x02`–`0x08`. The driver file and Ghidra output remain outside this
repository. No emulator source code was consulted.

Implemented behavior:

- Separate 128-byte channel FIFOs and channel register state.
- Signed 8-bit and signed 12-bit PCM byte layouts, PCM and ADPCM sample cadence, play/record
  direction, stereo route bits, volume values and reset/start control.
- FIFO threshold status and DMA request conditions from register `0x0c`.
- Channel 0 controlled alternating interleave mode, enabled only when both channels have DMA
  enabled, plus the documented shared playback/record mode and rate controls.
- Timer zero, base counter, timer one and timer two counters with their documented register
  locations and 32-master-clock counter units.
- Separate 16-byte MIDI transmit and receive FIFOs, reset controls used by the original driver,
  and serial output at 31,250 baud with fractional clock carry.
- The status byte's overrun, timer, MIDI request and FIFO threshold bits. Register `0x0d` mask
  bits gate IRQ output without changing status, as the toolkit specifies.
- Timestamped port access, sample updates, MIDI output and external record-input injection.
- `cargo test -p gameframe-ymz263` checks both 12-bit layouts, signed 8-bit PCM, ADPCM nibble
  order, FIFO thresholds, timer and MIDI timing. These checks exercise the implementation; the
  YMZ263B-specific ADPCM rounding still needs a software reference comparison.

## Limits and software checks

This is a documented digital model, not a claim of complete silicon parity. YMZ263-specific ADPCM
predictor clipping and rounding are inferred from Yamaha's related Y8950 algorithm. The toolkit
does not specify exact timer-flag clear behavior, standby clock reduction or game-port electrical
timing. Interleave and FIFO threshold behavior follow the toolkit description but need comparison against a
software-instrumented driver. A run of the original GSS driver in an instrumented NE emulator can
settle those points. Software checks can cover all FIFO sizes,
formats, rates, timer reloads, MIDI reset and transfer sequences, driver port ordering and
monotonic timestamp boundaries without a physical card.

The crate provides DMA request state but does not emulate a PC DMA controller, interrupt
controller, analog ADC/DAC filters, card mixer, joystick RC circuit or telephone/microphone input
path. Those belong to the machine or card layer. Real hardware testing is not required to make
progress.

## Licence

Apache 2.0.
