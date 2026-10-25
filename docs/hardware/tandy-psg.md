# PCjr and Tandy 1000 PSG

This note covers the TI SN76496 sound generator and its use by early PCjr and Tandy 1000 games.
It records the initial GameFrame implementation, traces of Sierra's shipped DOS drivers, and
software checks that can close the remaining unknowns. It does not require a physical sound card.

## Hardware and crate

The TI SN76496 has three programmable tone channels, a noise channel, four attenuation controls,
a serial write interface and an external audio input. Its tone counters advance at one sixteenth
of the reference clock. Tone output toggles when a counter expires. Noise rates are derived from
the same reference clock or from tone generator 3. A write to the noise control register clears
the shift register. TI documents a 32-clock READY transfer interval and a feedback network that
must not lock at zero. The manuals do not give the feedback taps or power-up contents.

The first GameFrame implementation lives in [`gameframe-sn76496`](../../crates/sn76496/README.md).
It uses a 3,579,545 Hz reference clock, models the serial register protocol, generates normalized
mono samples, and exposes the TI READY countdown. The crate maps PCjr writes to `C0h` and Tandy
writes to `C0h`–`C7h`. Tone period zero, the noise sequence and the initial attenuation state are
implementation choices pending software comparison.

The Tandy 1000 service manual maps writes to the SN76496 at `C0h`–`C7h`. Those addresses alias the
same device. The service manual says the chip select is write-only and about 42 CPU wait states
are inserted. The card adapter returns `Undriven` for a read in that range. The host bus model
chooses the open-bus value. `TANDY_PSG_WRITE_WAIT_STATES` exposes the approximate wait count for a
machine adapter to apply.

The Tandy board also routes PIT channel 2 into the PSG's external audio input. The service manuals
describe speaker and external-output controls in the machine-level interface. The crate accepts a
normalized input sample but does not model PIT timing, the speaker gate, source mux, amplifier or
analog filter.

## Sierra driver evidence

The KQ4 SCI demo archive contains the original `JR.DRV` and `TANDY.DRV` binaries. The archive
listing identifies it as a 275 KB non-playable DOS demo. The ZIP records `JR.DRV` as 2,918 bytes,
dated 1989-03-13 11:36, and `TANDY.DRV` as 3,316 bytes, dated 1989-05-04 16:43. The hashes below
identify the exact files inspected. Static instruction decoding used Capstone in 16-bit x86 mode
on each COM image, with the COM load address set to `0100h`. The entry jump at `0100h` reaches
`03E1h` in `JR.DRV` and `03E7h` in `TANDY.DRV`. Ghidra was not configured on this host, so this is
an instruction trace, not Ghidra pseudocode. No code from another sound emulator was consulted.

| Driver | SHA-256 | Address | Observed instructions and behavior |
|---|---|---:|---|
| `JR.DRV` | `5182dad63d7cefa7214751d7b8edaa71efed0ad0174705321f107928122dad7f` | `0431h`–`0435h` | Reads port `61h`, sets bits 5 and 6, and writes the value back during device initialization. |
| `JR.DRV` | same | `0465h`–`046Bh` | Loops over voices 0–2 and writes `9Fh`, `BFh` and `DFh` to port `C0h`, setting each tone attenuation to off. |
| `JR.DRV` | same | `08D7h`–`08E0h` | Forms a latch byte and a continuation byte, then writes both to port `C0h`. The continuation byte is masked to six bits. |
| `JR.DRV` | same | `0AE4h`–`0AF1h` | Forms a channel attenuation latch with `90h` plus the voice number and writes it to port `C0h`. |
| `TANDY.DRV` | `ba1fb26147c9190df05fda15551ec4f78603ad79103b3c0d84ad483655c6c2a4` | `0467h`–`046Bh` | Sets bits 5 and 6 at port `61h` during initialization. |
| `TANDY.DRV` | same | `049Bh`–`04A1h` | Loops over voices 0–2 and writes their attenuation-off latch bytes to port `C0h`. |
| `TANDY.DRV` | same | `09D7h`–`09E0h` | Writes a tone latch byte and a six-bit continuation byte to port `C0h`. |
| `TANDY.DRV` | same | `0BE6h`–`0BF1h` | Forms and writes one channel attenuation latch to port `C0h`. |

These traces show the driver using the PSG's standard serial protocol. They do not establish the
chip's internal timing, analog output, exact channel volume curve or noise sequence. The Tandy
service manual documents port aliases even though these driver paths write to `C0h`.

The source archive is the [DOS Games Archive KQ4 SCI demo page](https://www.dosgamesarchive.com/file/kings-quest-iv-the-perils-of-rosella/kq4scid).
The Sierra archive page also lists the [original KQ4 SCI demo ZIP](https://sierrahelp.com/Misc/Demos.html).
Hardware references are the [TI SN76489AN data sheet](https://map.grauw.nl/resources/sound/texas_instruments_sn76489an.pdf),
the [Tandy 1000 service manual](https://ftp.oldskool.org/pub/drivers/Tandy/1000/Tandy_1000_Service_Manual.pdf),
and the [IBM PCjr Technical Reference](https://bitsavers.trailing-edge.com/pdf/ibm/pc/pc_jr/PCjr_Technical_Reference_Nov83.pdf).

## Software-only checks

Run these in a fake machine adapter or an existing PCjr/Tandy software emulator. Physical testing
is optional.

1. Replay the driver's initialization and a small set of note and attenuation writes. Capture the
   I/O log and confirm the port, byte order and timing in both `JR.DRV` and `TANDY.DRV` paths.
2. Use a tiny DOS test program to write the same bytes through the PCjr `C0h` port and every Tandy
   alias from `C0h` to `C7h`. Confirm they update one register stream and expose about 42 host CPU
   wait states per Tandy write. A read should leave the PSG data bus undriven; let the host's bus
   model determine the returned value.
3. Test tone periods `0`, `1`, `2`, `0x3FF`, and midrange values. Count transitions against a
   3,579,545 Hz input clock. Resolve period zero and the first-counter phase against a named
   software PCjr/Tandy reference.
4. Test all four fixed noise rates, the tone-3 rate, periodic and white noise, and repeated noise
   register writes. Capture output samples from DOSBox-X's PCjr/Tandy machine model and compare
   the sequence to determine the feedback taps, output bit, seed and divider phase.
5. Feed a timer-2 square wave through the Tandy board input path, then change the documented
   speaker and external-output controls. Compare the digital mix against DOSBox-X output to
   resolve gain, gating, routing and filter limits.

## Open questions

- Which SN76496 feedback taps and output bit does the Tandy board use, and what is the reset seed?
- What tone output does a zero period produce on the Tandy SN76496, and what is its initial phase?
- Does the 32-clock TI READY interval exactly account for the board's approximate 42 CPU waits, or
  does the Tandy bus add extra cycles under some CPU profiles?
- What gain and filtering does the Tandy external input and output path apply?
- The KQ4 driver path gives useful register writes but not a complete model of Sierra's SCI timing
  or the board-level audio route. A software port harness can capture those missing inputs.
