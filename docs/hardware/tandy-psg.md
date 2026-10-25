# PCjr and Tandy 1000 PSG

This note covers the TI SN76496 in the IBM PCjr and original Tandy 1000. It records the GameFrame
implementation, traces of Sierra's shipped DOS drivers, and software checks for the remaining
unknowns. Physical sound cards are not required.

## Hardware and crate

The TI SN76496 has three programmable tone channels, a noise channel, four attenuation controls,
a serial write interface and an AUDIO IN pin. Tone counters advance at one sixteenth of the
reference clock. For a non-zero period n, the output frequency is N/(32n). The three fixed noise
rates are N/512, N/1024 and N/2048; the fourth rate follows tone generator 3. A write to the noise
control register clears the shift register. TI specifies 32 input clocks for READY and an XOR
feedback network that must not lock at zero. The available TI and machine manuals do not give the
feedback taps, zero-period behavior or power-up contents.

| Finding | Primary source | Evidence |
|---|---|---|
| Tone counters use the reference clock divided by 16; nonzero period n gives N/(32n). Fixed noise rates are N/512, N/1024 and N/2048. | [TI SN76489AN data sheet](https://map.grauw.nl/resources/sound/texas_instruments_sn76489an.pdf) | Operation: tone generators, noise generator and CPU interface |
| PCjr port 61h bits 5–6 select timer 2, cassette input, I/O-channel audio or the SN76496. | [IBM PCjr Technical Reference](https://bitsavers.trailing-edge.com/pdf/ibm/pc/pc_jr/PCjr_Technical_Reference_Nov83.pdf) | PPI Port B and Sound Subsystem |
| The original Tandy 1000 routes PIT channel 2 into the sound chip's external input and uses a separate external-output mux controlled by 8255 PB5/PB6. | [Tandy 1000 service manual](https://ftp.oldskool.org/pub/drivers/Tandy/1000/Tandy_1000_Service_Manual.pdf) | Keyboard/Timer/Sound Circuits, pp. 32–35 |
| The Tandy 1000 HX identifies an NCR 8496 and describes its output stage. | [Tandy 1000 HX Technical Reference](https://www.manualslib.com/manual/1366743/Tandy-1000-Hx.html) | Sound Generator, §2.6, p. 323 |

The GameFrame implementation lives in [`gameframe-sn76496`](../../crates/sn76496/README.md).
It uses a 3,579,545 Hz reference clock, models the serial register protocol, generates normalized
mono samples and advances chip/READY state by input-clock cycles. The PCjr adapter maps writes to
`C0h`; the original Tandy 1000 adapter maps writes to `C0h`–`C7h`. Tone period zero, the noise
sequence and the initial attenuation state remain implementation choices pending comparison.

Port `61h` bits 5 and 6 select the output sent to the PCjr audio outputs. The new `PcjrAudioMux`
implements this digital selection: `00b` selects timer 2, `01b` cassette input, `10b` I/O-channel
audio and `11b` the PSG. The Tandy 1000 service manual shows a separate external-output mux
controlled by port 61h bits 5 and 6. It identifies the PSG, timer 2 and external audio as sources,
but does not provide the full selection table. Both Sierra drivers set bits 5 and 6 during sound
initialization. Other port 61h behavior, source generation, analog gain and filtering remain with
the host machine model.

The Tandy 1000 service manual maps writes to the SN76496 at `C0h`–`C7h`. Those addresses alias the
same device. The service manual says the chip select is write-only and about 42 CPU wait states
are inserted. The card adapter returns `Undriven` for a read in that range. The host bus model
chooses the open-bus value. `TANDY_PSG_WRITE_WAIT_STATES` exposes the approximate wait count for a
machine adapter to apply.

The SN76496 AUDIO IN pin can be mixed with its four generated channels. The crate accepts a
normalized sample at that pin. The Tandy 1000 service manual routes PIT channel 2 to this input.
The board's external-output mux is a separate path. The PCjr source table is implemented; the
Tandy table remains open.

The original Tandy 1000 service manual identifies an SN76496 and documents its port range and wait
states. The Tandy 1000 HX Technical Reference identifies an NCR 8496 instead. The TI profile here
does not claim to cover the NCR noise generator or later Tandy PSSJ devices.

## Sierra driver evidence

The KQ4 SCI demo archive contains the original `JR.DRV` and `TANDY.DRV` binaries. The archive
listing identifies it as a 275 KB non-playable DOS demo. The ZIP records `JR.DRV` as 2,918 bytes,
dated 1989-04-26 10:55, and `TANDY.DRV` as 3,316 bytes, dated 1989-05-04 16:43. The hashes below
identify the exact files inspected. Static instruction decoding used Capstone in 16-bit x86 mode
on each COM image, with the COM load address set to `0100h`. The entry jump at `0100h` reaches
`03E1h` in `JR.DRV` and `03E7h` in `TANDY.DRV`. Ghidra was not configured on this host, so this is
an instruction trace, not Ghidra pseudocode. The implementation is independent code based on
hardware documents and these driver traces.

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

These traces show the driver using the PSG's standard serial protocol. They also show sound
initialization selecting the PSG output through port 61h. They do not establish the chip's internal
timing, analog output or noise sequence. The Tandy service manual documents port aliases even
though these driver paths write to `C0h`.

The source archive is the [DOS Games Archive KQ4 SCI demo page](https://www.dosgamesarchive.com/file/kings-quest-iv-the-perils-of-rosella/kq4scid).
The Sierra archive page also lists the [original KQ4 SCI demo ZIP](https://sierrahelp.com/Misc/Demos.html).
Hardware references are the [TI SN76489AN data sheet](https://map.grauw.nl/resources/sound/texas_instruments_sn76489an.pdf),
the [Tandy 1000 service manual](https://ftp.oldskool.org/pub/drivers/Tandy/1000/Tandy_1000_Service_Manual.pdf),
the [IBM PCjr Technical Reference](https://bitsavers.trailing-edge.com/pdf/ibm/pc/pc_jr/PCjr_Technical_Reference_Nov83.pdf),
and the [Tandy 1000 HX Technical Reference](https://www.manualslib.com/manual/1366743/Tandy-1000-Hx.html).

## Software-only checks

Run these in a fake machine adapter or an existing PCjr/Tandy software emulator. Physical testing
is optional.

1. Run `JR.DRV` and `TANDY.DRV` in software PCjr/Tandy harnesses. Record I/O writes and verify
   port 61h source selection, PSG port, byte order and host wait accounting.
2. Use a small DOS probe to write identical bytes through PCjr `C0h` and every Tandy alias
   `C0h`–`C7h`. Compare the chip register stream, undriven reads and Tandy bus waits.
3. Generate tones at periods `0`, `1`, `2`, `0x3FF` and midrange values. Compare transition
   counts and phase against the TI clock formula, then capture the same cases from a named,
   versioned PCjr/Tandy software emulator.
4. Exercise all four fixed noise rates, tone-3 clocking, periodic and white noise, and repeated
   noise-register writes. Compare captured samples with software-emulator output; record the
   reference profile because TI does not publish the taps or seed.
5. Verify the PCjr port 61h mux with four distinct generated sources. For Tandy, capture each
   port 61h selection and compare the PSG, timer 2 and external audio routes against a software
   Tandy model. Compare the Tandy 1000 HX profile separately for its NCR 8496 behavior. Keep output
   gain and filtering separate from digital chip state.

## Open questions

- Which TI SN76496 feedback taps and output bit are used, and what is the reset seed?
- What tone output does zero period produce, and what is the initial phase?
- Does the 32-clock TI READY interval exactly account for the board's approximate 42 CPU waits, or
  does the Tandy bus add extra cycles under some CPU profiles?
- How does the NCR 8496 differ from TI for noise reset, feedback taps and output polarity?
- Which later Tandy models use the PSSJ audio block, and what are its DAC and port behaviors?
- What gain and filtering does each board's input and output path apply?
- Sierra's driver trace gives useful writes but not complete SCI timing. A software port harness
  can capture that path without using physical hardware.
