# gameframe-sn76496

A clean Rust model of the Texas Instruments SN76496 programmable tone/noise generator and the
PCjr and original Tandy 1000 interfaces documented here.

- Three 10-bit tone period registers and four attenuation registers.
- One noise control register with the documented periodic/white-noise selection and four shift
  rates.
- The SN76496 latched register/data protocol, including tone continuation bytes.
- One normalized mono output sample per internal `clock / 16` counter tick.
- Input-clock advancement that runs chip counters and READY timing independently of rendering.
- An IBM PCjr adapter for the write-only PSG port `C0h`.
- An original Tandy 1000 adapter for writes to the eight decoded ports `C0h` through `C7h`.
- The PCjr port `61h` output mux for timer 2, cassette, I/O-channel audio and PSG output.
- The original Tandy 1000 external Audio Out mux for PIT channel 2, bus audio input and PSG output.
- The service manual's approximate 42 CPU wait states per PSG write, exposed for the host machine
  adapter to apply.
- The TI READY interval of 32 input clock cycles, exposed as `ready()` and
  `ready_clocks_remaining()`.
- A normalized input at the chip's AUDIO IN pin, including the original Tandy 1000 PIT channel 2
  route.

The IBM PCjr Technical Reference documents the SN76496N clock, tone divisor, fixed noise rates,
attenuation steps, register map and READY interval. The Tandy 1000 service manual schematic shows
the same 3.579 MHz clock on its SN76496. The Tandy port range, write-only decode and approximate
wait-state count follow its
[service manual](https://ftp.oldskool.org/pub/drivers/Tandy/1000/Tandy_1000_Service_Manual.pdf).
The implementation follows hardware documents and static traces of the original Sierra drivers.
It contains no copied emulator code. The software-only follow-up plan is in
[`docs/hardware/tandy-psg.md`](../../docs/hardware/tandy-psg.md).

## Known gaps

- The TI and machine manuals describe an XOR feedback network but do not give its tap positions or
  power-up state. The current 15-bit LFSR and seed are deterministic candidates. The Tandy 1000 HX
  manual identifies an NCR 8496; this crate does not yet model that variant.
- TI documents 32 PSG clock cycles for READY. Tandy's board adds CPU wait states and maps the
  signal into its CPU READY circuit. The adapter exposes the chip's clock countdown but does not
  emulate CPU wait-state insertion.
- `TANDY_PSG_WRITE_WAIT_STATES` is the service manual's approximate value. A machine adapter still
  has to turn it into the selected CPU's bus timing.
- The PCjr `PcjrAudioMux` selects one of four normalized sources. Tandy has a separate analog
  output mux and speaker path. The Tandy BIOS documents three mux selections. Its fourth analog
  input is not identified, so `TandyAudioOutputMux::route_sample` returns `None` for that select
  code. The speaker path and filtering remain separate host behavior.
- Later Tandy PSSJ hardware, including its additional 8-bit DAC, is outside this crate.
- Tone period zero is treated as a constant-high tone. This is a deterministic placeholder. The
  manuals do not define its output; the software-only check plan records how to resolve it.
- Output is a normalized digital approximation. It does not model analog current levels or the
  Tandy output filter.

Sierra SCI used `JR.DRV` and `TANDY.DRV` for PCjr/Tandy playback. The KQ4 SCI demo contains both
original binaries; their observed port writes are summarized in
[`docs/hardware/tandy-psg.md`](../../docs/hardware/tandy-psg.md). The software-only comparison plan
is there as well.

## Example

```rust
use gameframe_sn76496::TandyPsgCard;

let mut psg = TandyPsgCard::new();
psg.write_port(0x0c0, 0x85); // latch tone 1 with its low four period bits
psg.write_port(0x0c1, 0x0f); // continuation: tone period high six bits
psg.write_port(0x0c2, 0x90); // mute tone 1
let sample = psg.sample();
```

## Licence

Apache 2.0.
