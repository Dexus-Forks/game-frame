# gameframe-sound-blaster

A clean Rust device model for the Creative Sound Blaster DSP generations 1.xx through 4.xx,
their CT1335/CT1345/CT1745 mixer register interfaces, the SB16 MPU-401 UART ports, and the card's
YM3812/YMF262 FM synthesizer. It reuses GameFrame's `gameframe-opl2` and `gameframe-opl3` crates.

The command and port model follows Creative's *Developer Kit for Sound Blaster Series, Second
Edition, Hardware Programming Reference*. The original Creative DOS/Windows driver package was
located as an interface reference. Its bundled license restricts reverse engineering, so this
crate does not use its code or contain its binaries. No other emulator source is used.

## Use it

```toml
[dependencies]
gameframe-sound-blaster = { git = "https://github.com/tanepiper/game-frame" }
```

```rust
use gameframe_sound_blaster::{DspEvent, SoundBlaster, SoundBlasterConfig};

let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
card.write_port_at(0x226, 1, 0).unwrap();
card.write_port_at(0x226, 0, 3_000).unwrap();
card.advance_to(103_000).unwrap();
assert_eq!(card.read_port_at(0x22E, 103_000).unwrap(), Some(0x80));
assert_eq!(card.read_port_at(0x22A, 103_000).unwrap(), Some(0xAA));

// Route DmaRequest events to the machine's ISA DMA controller. The returned PCM can go to a
// host mixer or resampler. This card model does not implement the ISA controller or PIC.
let _events: Vec<DspEvent> = card.advance_to(200_000).unwrap();
```

`SoundBlasterConfig` provides SB 1.5, SB 2.0, SB 2.0 CD, the dual-OPL2 SB Pro 1, SB Pro 2 and SB16
profiles. A custom configuration selects another DSP version, base port, mixer chip, FM chip and
SB16 MPU port. Every time argument is a monotonic virtual nanosecond timestamp. The host services playback requests through
`service_dma_read`, sends capture input through `push_capture_sample`, forwards raised IRQ events
to its PIC, and calls `dma_terminal_count_at` for legacy auto-initialize blocks whose DSP command
does not carry a block length.

## Implemented surfaces

- DSP reset pulse, ready signature, version query, write/read status, response FIFO and port-level
  interrupt acknowledgement.
- Direct 8-bit PCM output and capture; single-cycle 8-bit DMA; 2.x auto-initialize; 2.01+ 8-bit
  high-speed transfers; Pro stereo input/output controls; and SB16 8/16-bit signed or unsigned,
  mono or stereo, single-cycle or auto-initialize DMA commands.
- Transfer-rate scheduling from the legacy time constant or SB16 sampling-rate commands, DMA
  request events, capture writes, playback sample conversion, pause/resume, per-block interrupts,
  and deferred auto-init exit.
- ADPCM command recognition for 2-, 3- and 4-bit streams. It marks the initial reference byte and
  emits compressed input bytes with the selected format. It does not decode them to PCM.
- DSP MIDI polling/interrupt and UART command modes, timestamped input, and MIDI output events.
- OPL2 FM ports at `388h`/`389h` and the card's legacy `base + 8` ports. OPL3 profiles expose both
  YMF262 banks at `388h` through `38Bh`. Pro and SB16 profiles also expose their FM ports at
  `base + 0` through `base + 3`; the original Pro profile routes its two YM3812 chips to separate
  stereo channels, while Pro 2 and SB16 route to one YMF262.
  `FmSynth::generate` returns mono-duplicated or stereo frames for the host mixer.
- YMF262 register-write spacing on SB Pro 2/SB16 ports. The OPL2 profile currently applies no
  timed write rejection because its port busy interval is not established here.
- CT1335, CT1345 and CT1745 indexed register maps, documented reset values and writable masks.
  SB16 mixer registers 80h, 81h and 82h expose the selected IRQ, DMA and interrupt source state.
- SB16 MPU-401 UART reset, UART-mode acknowledgement, readiness status, input/output and interrupt
  state. Its output-ready interval can be configured with `set_write_busy_ns`; the default is zero
  because the programming reference gives no byte duration.
- High-speed DSP mode rejects further command-port writes until reset. Physical DMA page boundaries,
  DMA arbitration and PIC delivery belong to the host machine.

## Limits and software checks to add

The public reference describes ADPCM reference-byte and block-command handling, but not the
predictor recurrence or update tables. Commands and transfer counts are recognized, and the initial
reference byte is marked, but compressed bytes are not decoded to PCM. Compare generated vectors
against an independently documented bitstream reference before adding the decoder.

The public reference gives approximate reset readiness but not per-command DSP busy durations, DMA
FIFO depth, bus arbitration, all DMA terminal-count edge timing, or exact SB16 stereo word/frame
cadence. DSP and MPU output-busy durations are configurable and default to zero. Mixer tone,
low-pass filters, analog gain, source mixing and the output amplifier are not applied to PCM. These
details do not require physical hardware to proceed. Software checks can run generated port
sequences under an instrumented DOS virtual machine, compare reset/version/status/IRQ results
against this crate, and send looped capture/playback vectors through a fake DMA/PIC adapter. The
Creative driver bundle is not needed for these checks.

The card does not contain an ISA DMA controller, PIC, MPU-401 intelligent mode, joystick/Game
Control Adapter, CD interface, C/MS SAA1099 chips, CSP/ASP signal processor, EMU8000 wavetable
synthesizer or analog circuitry. DSP clones may expose extensions beyond the Creative command set.
The machine model combines these devices as required. Physical hardware testing is optional.

## Licence

Apache 2.0.
