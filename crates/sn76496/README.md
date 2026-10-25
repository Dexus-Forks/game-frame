# gameframe-sn76496

A clean Rust model of the Texas Instruments SN76496 programmable tone/noise generator and its
IBM PCjr and Tandy 1000 I/O ports.

- Three 10-bit tone period registers and four attenuation registers.
- One noise control register with the documented periodic/white-noise selection and four shift
  rates.
- The SN76496 latched register/data protocol, including tone continuation bytes.
- One normalized mono output sample per internal `clock / 16` counter tick.
- An IBM PCjr adapter for the write-only PSG port `C0h`.
- A Tandy 1000 adapter for writes to the eight decoded ports `C0h` through `C7h`.
- The service manual's approximate 42 CPU wait states per PSG write, exposed for the host machine
  adapter to apply.
- The TI READY interval of 32 input clock cycles, exposed as `ready()` and
  `ready_clocks_remaining()`.
- A normalized external audio input for Tandy PIT channel 2.

The chip register map, counter divisor, attenuation steps, clock and write-ready behavior follow
the [TI SN76489AN data sheet](https://map.grauw.nl/resources/sound/texas_instruments_sn76489an.pdf)
and the IBM PCjr and Tandy 1000 technical references. The Tandy 1000 port range, write-only
decode and approximate wait-state count follow its
[service manual](https://ftp.oldskool.org/pub/drivers/Tandy/1000/Tandy_1000_Service_Manual.pdf).
The implementation follows hardware documentation and the original Sierra driver traces. It is
not a port of another sound emulator. The software-only follow-up plan is in
[`docs/hardware/tandy-psg.md`](../../docs/hardware/tandy-psg.md).

## Known gaps

- The TI and machine manuals describe an XOR feedback network but do not give its tap positions or
  power-up state. The current 15-bit LFSR and seed are deterministic candidates. Compare them with
  a named software output before treating them as silicon facts.
- TI documents 32 PSG clock cycles for READY. Tandy's board adds CPU wait states and maps the
  signal into its CPU READY circuit. The adapter exposes the chip's clock countdown but does not
  emulate CPU wait-state insertion.
- The Tandy board mixes PIT channel 2 into the chip's audio input. The caller must supply that
  normalized signal with `set_timer2_audio_input`.
- `TANDY_PSG_WRITE_WAIT_STATES` is the service manual's approximate value. A machine adapter still
  has to turn it into the selected CPU's bus timing.
- The Tandy board's internal speaker gate, external source mux, output amplifier and filter are not
  modeled yet.
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
