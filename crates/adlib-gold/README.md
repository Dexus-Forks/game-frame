# gameframe-adlib-gold

A clean Rust model of the AdLib Gold/GSS audio port block around Yamaha's YMF262 and YMZ263B. It
combines `gameframe-opl3`, `gameframe-ymz263` and the card's documented Control Chip access,
registers, status bits, EEPROM save/restore, and audio-port relocation.

The port and register behavior comes from the Ad Lib Gold Developer Toolkit v1.01, Hardware
Reference 4.1. The AdLib Gold `GSSOPL3.DRV` v1.01 binary provides an independent static trace of
the `0xFF`/`0xFE` bank switch, busy polling, register reads and OPL3 setup. No source code from an
emulator was used. The toolkit's `0xFF`/`0xFE` protocol resolves what the driver trace alone could
not.

## Use it

```toml
[dependencies]
gameframe-adlib-gold = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_adlib_gold::{AdlibGold, AdlibGoldConfig};

let mut card = AdlibGold::new(AdlibGoldConfig::default());
let t = 0_u64; // YMF262 master-clock cycles from the host's emulated timeline
card.write_port_at(0x38a, 0xff, t).unwrap(); // enter Control Chip bank
card.write_port_at(0x38a, 0x09, t + 32).unwrap(); // select left FM volume
card.write_port_at(0x38b, 0x3f, t + 64).unwrap(); // left FM volume code
let status = card.read_port_at(0x38a, t + 160).unwrap().unwrap();
assert_eq!(status & 0x40, 0); // no Control Chip write is pending
card.write_port_at(0x38a, 0xfe, t + 192).unwrap(); // return to OPL3 bank one
```

Pass one monotonic YMF262 master-clock timestamp for every I/O operation. OPL3 accesses enforce
the documented 32-clock minimum. Control Chip reads expose busy status; writes attempted while
the chip reports busy return an error so the machine model can advance time and retry. `read_port_at`
and `write_port_at` return `None`/`false` for ports or data paths this crate does not implement.
Render FM output through `chip()` or `chip_mut()`. MMA ports occupy `base + 4` through `base + 7`;
the adapter converts card timestamps to the YMZ263B clock. Drain generated PCM through
`mma_mut().drain_output_events()` and MIDI through `drain_midi_events()`.

## Implemented card behavior

- Ports `base + 0` through `base + 3` route the two YMF262 index/data pairs.
- Ports `base + 4` through `base + 7` route the two YMZ263B MMA index/data pairs. The MMA model
  includes two sample FIFOs, PCM/ADPCM cadence, timer events, MIDI FIFOs and UART output timing.
  `dma_request()` exposes per-channel request state for a machine-level DMA controller.
- Writing `0xff` to `base + 2` enters the Control Chip bank. Writing `0xfe` there returns to FM
  bank one. In Control Chip mode, `base + 2` selects a register and reads status; `base + 3`
  reads or writes the selected register.
- Control status reports active-low FM, sampling, telephone and SCSI interrupt inputs in bits
  0–3, the register-write busy interval in bit 6, and EEPROM activity in bit 7. Interrupt inputs
  can be set by later card subsystems through `set_interrupt_pending`.
- Registers `0x00` through `0x18` expose the documented board ID/options, telephone state, sampling
  gains, output and per-source controls, IRQ/DMA selections, port relocation and surround level.
  Register `0x12` is unused. Register `0x00` stores/restores the writable register image when its
  ST/RT command bits are set.
- Register `0x15` moves the eight-port audio block to the selected 8-byte-aligned base. The new
  address applies immediately after the data write.
- `mix_sources()` applies the documented signed linear gains for FM, sampling, auxiliary,
  microphone and telephone inputs, PC-speaker enable, left/right channel selection, forced mono,
  mute and final output levels. Telephone input is mixed only when the telephone option is
  present and Control Chip register `0x01` has its line-engage bit set.
- Writes to registers `0x04`–`0x08` assert the documented 450 μs busy window. Writes to
  `0x09`–`0x16` assert the documented 5 μs window. EEPROM write latency is configurable because
  the toolkit requires polling bit 7 but gives no duration. Restore uses 2499 μs as the
  conservative implementation of the toolkit's “a bit less than 2.5 ms” wait; this is not a
  measured chip timing.

`mix_sources()` reports the selected bass/treble codes and pseudo/spatial mode but does not apply
their analog transfer. The exact TDA8425 frequency response, card anti-alias filters, YAC512
floating DAC conversion, joystick RC timing, the external DMA controller and the host interrupt
controller remain outside the card's sample mixer. The exact YMZ263 ADPCM update and FIFO threshold
transition behavior still need software-driver comparisons. Revision-specific reset defaults
remain unknown. The default register
image sets the documented `0x388` port address;
other reset values and option straps must be selected for the card revision the game expects.
Supply initial register and EEPROM images
through `AdlibGoldConfig` when those values are known. See the [AdLib Gold findings](../../docs/hardware/adlib-gold-gssopl3.md)
and the software-only checks there. Physical hardware testing is optional.

## Licence

Apache 2.0.
