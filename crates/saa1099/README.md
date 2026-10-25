# gameframe-saa1099

A clean Rust model of Philips's SAA1099 stereo sound generator and the two-chip Creative Music
System/Game Blaster card interface.

- Six tone channels, eight octave ranges and 256 tone values per octave.
- Six left/right amplitude controls, frequency and noise enables, two noise clocks, two envelopes,
  global sound enable and synchronized frequency reset.
- The SAA1099 write-only address/data interface. The selected register remains active across data
  writes, and register addresses repeat every 32 bytes.
- The CMS four-port window at `220h` by default. The first chip uses `220h` for data and `221h` for
  register selection. The second uses `222h` and `223h`.
- Stereo samples at one output step per 128 master clocks. The nominal Philips clock is 8 MHz. The
  CMS profile uses a reported 7.15909 MHz clock and can be configured for another board clock.

The register model follows Philips's 1986 data handbook. The CMS port order and default base come
from a static trace of Creative's `CMSDRV.COM`, Sierra's `CMS35.BIN`, and the driver disk notes.
The driver files and Ghidra exports stay outside this repository. The full evidence and comparison
plan are in [`docs/hardware/cms-saa1099.md`](../../docs/hardware/cms-saa1099.md).

## Open software comparisons

The Philips data sheet does not state the noise shift-register taps or starting state. This model
uses a deterministic 18-bit candidate sequence so noise-enabled programs produce repeatable output.
The combined tone/noise gate is inferred. The output is a normalized digital stream; it does not
reproduce the chip's PWM carrier, analog current-sink levels or the card's output filter. Card
detection reads and bus wait states remain open.

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
