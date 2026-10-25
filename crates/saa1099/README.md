# gameframe-saa1099

A clean Rust model of Philips's SAA1099 stereo sound generator and the two-chip Creative Music
System/Game Blaster card interface.

- Six tone channels, eight octave ranges and 256 tone values per octave.
- Six left/right amplitude controls, frequency and noise enables, two noise clocks, two envelopes,
  global sound enable and synchronized frequency reset.
- Tone and octave writes become active at the next transition of their frequency generator. The
  output model applies that transition at its `clock_hz / 128` sample step.
- Buffered envelope waveform, clock-source and inversion controls change at the active envelope's
  boundary. External envelope clocks come from address writes to register `18h` or `19h`.
- The SAA1099 write-only address/data interface. The selected register remains active across data
  writes, and register addresses repeat every 32 bytes.
- The CMS four-port window at `220h` by default. The first chip uses `220h` for data and `221h` for
  register selection. The second uses `222h` and `223h`.
- CMS auxiliary status and detection ports: `base+4` reads as `7Fh`; writes to `base+6/+7` are
  returned at `base+0A/+0B` as a compatibility model of the card's custom detection IC.
- Stereo samples at one output step per 128 master clocks. The nominal Philips clock is 8 MHz. The
  CMS profile uses a reported 7.15909 MHz clock and can be configured for another board clock.

The register model follows Philips's 1986 data handbook and its SAA1099 article in *Electronic
Components and Applications*. The CMS chip-port order and default base come from static traces of
Creative's `CMSDRV.COM`, Sierra's `CMS35.BIN`, and the driver disk notes. Auxiliary read behavior
follows a modern Game Blaster-compatible card manual because the original custom IC is not fully
documented. The driver files and Ghidra exports stay outside this repository. The full evidence and
software-only comparison plan are in [`docs/hardware/cms-saa1099.md`](../../docs/hardware/cms-saa1099.md).

## Open software comparisons

An external SAA1099 hardware-measurement project reports the 18-bit noise polynomial used here.
The Philips data sheet does not state the power-up seed, so this model uses seed one for repeatable
output. The combined tone/noise gate remains inferred. The custom detection IC's full state machine,
DTACK bus waits, PWM carrier, analog current levels and card output filter are not modeled. The CMS
detection ports implement the common compatibility behavior described by the card manual.

```rust
use gameframe_saa1099::CmsCard;

let mut cms = CmsCard::new();
cms.write_port(0x221, 0x08); // select tone register 0 on chip 0
cms.write_port(0x220, 0x80); // set its tone value
cms.write_port(0x221, 0x1c); // select global sound control
cms.write_port(0x220, 0x01); // enable sound
let sample = cms.sample();
```

## Licence

Apache 2.0.
