# Sound Blaster DSP, mixer and MPU-401 model

`gameframe-sound-blaster` adds a reusable port-level model for common Creative Sound Blaster
cards. It covers documented DSP generations 1.xx through 4.xx, the CT1335, CT1345 and CT1745
register interfaces, and the SB16 MPU-401 UART. It gives DOS software a card-facing interface and
exposes timestamped work for a host machine to complete.

This page records the interface evidence and the implementation boundary. It does not claim
compatibility with every DSP revision or clone.

## Reference and provenance

The interface reference is Creative's *Developer Kit for Sound Blaster Series, Second Edition,
Hardware Programming Reference*. It documents the DSP, mixer, SB-MIDI and MPU-401 UART ports and
command families for SB1.5, SB2.0, SB2CD, SB Pro and SB16. The linked PDF includes Creative's
license terms. This repository stores no copy of the manual, driver binaries, card firmware, game
files or captures.

The Creative `SBBASIC.EXE` driver package was located as a possible compatibility target. Its
included license restricts reverse engineering, so its binaries and code were not inspected or
used. The model was written from the published port and command interface. It does not use another
emulator's source.

## Card profiles

| Constructor | DSP response | Mixer | MPU-401 |
|---|---:|---|---|
| `sb_15()` | 1.5 | None | None |
| `sb_20()` | 2.1 | None | None |
| `sb_2_cd()` | 2.1 | CT1335 | None |
| `sb_pro_2()` | 3.2 | CT1345 | None |
| `sb16()` | 4.5 | CT1745 | Base `330h` |

These constructors choose common profiles. `SoundBlasterConfig` can set a different base port,
DSP version, mixer and MPU base. The Creative reference classifies cards by DSP generation, but
the specific minor values above are crate defaults.

## Port and command behavior

The DSP uses offsets from the configured base port. The model covers reset at `+6`, response data
at `+A`, command/data writes and write status at `+C`, read status at `+E`, and the SB16 16-bit IRQ
acknowledge at `+F`. Reset needs a high pulse of at least 3 microseconds in this model. After a
valid low edge it makes `AAh` available after 100 microseconds. These values follow the timing
guidance in Creative's chapter 2; software checks exercise just below and at both boundaries.

The DSP command model includes version query, speaker control, direct 8-bit samples, time-constant
and SB16 rate setup, 8-bit single-cycle and auto-init transfers, 2.01+ high-speed transfers, SB Pro
stereo selection, SB16 8/16-bit signed or unsigned mono/stereo DMA, pause/resume, block size,
auto-init exit, MIDI commands and ADPCM transfer commands. Length parameters use the documented
`N - 1` convention. Rate commands `41h` and `42h` set separate output and input rates. The bytes
are read high byte first as specified by the command description.

High-speed mode stops normal command handling until the DSP is reset. DSP write busy time is
configurable with `Dsp::set_write_busy_ns`. Its default is zero because the Creative reference says
to poll the status bit but does not set a byte duration.

The host machine owns ISA DMA and the PIC. `DmaRequest` identifies when the card asks for a
memory-to-card transfer. The host supplies the word through `service_dma_read`. Capture data is
exposed as `DmaWrite` events. For older auto-init transfers without a DSP block-size parameter,
the host reports DMA terminal count through `dma_terminal_count_at`. IRQ events and source status
are exposed for a host PIC adapter. The crate does not emulate DMA pages, channel masks, bus
arbitration, physical DMA memory or PIC edge/EOI behavior.

## Mixer and MIDI

The indexed mixer model covers the documented CT1335, CT1345 and CT1745 registers used by the
profiles. It stores register values, applies writable masks and exposes CT1745 interrupt status
`82h`. It does not apply analog source gain, tone controls, input selection, filters, ADC behavior
or amplifier response to the PCM samples.

The DSP's Sound Blaster MIDI commands provide MIDI input and output events. The optional SB16
MPU-401 model implements reset, UART entry, acknowledgement, ready status and byte input/output.
It raises a card IRQ event when UART mode is entered and when received data arrives. The host can
configure its output-ready interval; the default is zero because the reference gives no timing
duration. MPU-401 intelligent mode is not implemented.

## Open behavior

| Behavior | Current model | Useful software check |
|---|---|---|
| ADPCM | Recognizes 2-, 3- and 4-bit commands, transfer length and the initial reference byte. It passes compressed bytes through and does not decode PCM. The Creative reference describes the reference byte and command sequence, but not the predictor recurrence. | Generate compressed vectors from known PCM and compare against an independently documented bitstream implementation or an allowed black-box device model. Compare every output sample and state reset across reference and continuation blocks. |
| DSP command timing | Configurable busy interval with a zero default. Reset readiness is modeled from the manual's approximate timing. | Run generated reset, status-polling and command sequences in a DOS virtual machine. Compare port reads and accepted writes at boundary timestamps. |
| DMA and IRQ timing | Timestamped requests, captured writes and DSP-side IRQ state. DMA controller and PIC remain host responsibilities. | Connect a fake 8237-style DMA adapter and PIC. Check exact transfer counts, terminal count, 64 KiB page behavior, IRQ source bits, acknowledgement and auto-init exit. |
| SB16 FIFO and stereo cadence | Separate output/input rates and left/right PCM words. FIFO depth and exact bus cadence are not modeled. | Drive the same generated stereo stream through a fake DMA adapter at several rates. Compare request count, timestamps, channel order and block IRQs with a documented software reference. |
| Mixer audio path | Register storage and masks only. | Generate one source at a time, exercise every documented register mask, then compare gain/filter output with a software reference if the project needs analog-like samples. |
| DSP clones and AWE models | Creative command set only. No clone extensions, ASP/CSP execution, EMU8000 wavetable or SoundFont Manager. | Add a profile only after a game's driver probes show which extension it needs. Compare generated probe responses against an allowed software target. |

These checks can run without a physical card. The crate's current tests cover reset/version,
8-bit and 16-bit transfers, capture, rate selection, high-speed command gating, IRQ acknowledgement,
mixer register profiles, ADPCM reference-byte marking and MPU-401 UART behavior. A future DOS
virtual-machine differential harness can compare port traces without making hardware access a
release gate.

## Source

- Creative, [*Developer Kit for Sound Blaster Series, Second Edition, Hardware Programming
  Reference*](https://www.ardent-tool.com/sound/Dev_Kit_SB_Series_Hardware_Programming_Ref_DSA0085515.pdf):
  chapters 2–6 and appendix A.
- Creative, [SBBASIC.EXE support listing](https://cn.creative.com/support/downloads/download.asp?DriverLang=2052&searchString=SBBASIC.EXE):
  package identified for later compatibility work; its included license prevents reverse
  engineering.

Licence: CC0-1.0.
