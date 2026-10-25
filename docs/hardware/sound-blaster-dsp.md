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

| Constructor | DSP response | Mixer | FM | MPU-401 |
|---|---:|---|---|---|
| `sb_15()` | 1.5 | None | One YM3812 | None |
| `sb_20()` | 2.1 | None | One YM3812 | None |
| `sb_2_cd()` | 2.1 | CT1335 | One YM3812 | None |
| `sb_pro()` | 3.1 | CT1345 | Two YM3812 chips, left and right | None |
| `sb_pro_2()` | 3.2 | CT1345 | One YMF262 | None |
| `sb16()` | 4.5 | CT1745 | One YMF262 | Base `330h` |

These constructors choose common profiles. `SoundBlasterConfig` can set a different base port,
DSP version, mixer, FM chip and MPU base. The Creative reference classifies cards by DSP generation,
but the specific minor values above are crate defaults.

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

The FM component reuses GameFrame's OPL2 and OPL3 chip crates. It maps legacy FM writes at `388h`
and `base + 8`; OPL3 profiles map both YMF262 banks at `388h` through `38Bh`. Pro and SB16
profiles also map the card ports at `base + 0` through `base + 3`. The Pro 1 profile routes the
two YM3812 outputs left and right. Pro 2 and SB16 expose both YMF262 register banks. The YMF262
secondary address port is write-only. The host pulls FM frames through `FmSynth::generate` and
combines them with PCM and other card sources. This crate does not apply the analog mixer path to
those samples.

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

The DSP's Sound Blaster MIDI commands provide MIDI input and output events. Timestamp mode places
three low-to-high millisecond bytes before each MIDI byte. Creative does not specify the timestamp
epoch; this implementation measures from virtual time zero. Resetting DSP UART mode restores the
previous MIDI mode and keeps other DSP settings. The optional SB16 MPU-401 model implements reset,
UART entry, acknowledgement, ready status and byte input/output. It raises a card IRQ event when
UART mode is entered and when received data arrives. Writes while the interface reports busy are
rejected. The host can configure its output-ready interval; the default is zero because the
reference gives no byte duration. MPU-401 intelligent mode is not implemented.

## Open behavior

| Behavior | Current model | Useful software check |
|---|---|---|
| ADPCM | Decodes 2-, 3- and 4-bit transfers to unsigned PCM samples. Reference bytes set the predictor and step. DMA requests follow the decoded sample cadence. The implementation follows CT1741 firmware 4.05 static disassembly; sample parity is not yet measured. A no-reference transfer before any prior reference starts from predictor zero, which remains unconfirmed. | Run the three decode helpers in a Ghidra 8051 p-code harness with a stub sample port. Record predictor, step and each output byte. Compare sign and magnitude combinations, predictor clamps, step limits, the packed 3-bit tail, nibble order, request cadence and continuation blocks. This software-only check does not need a physical card. |
| DSP and OPL2 port timing | DSP command busy time and YM3812 port busy time are not specified by the reference used here. DSP busy time defaults to zero. YMF262 writes enforce its documented spacing. | Run generated reset, status-polling and FM register sequences in a DOS virtual machine. Compare port reads and accepted writes at boundary timestamps. |
| DMA and IRQ timing | Timestamped requests, captured writes and DSP-side IRQ state. DMA controller and PIC remain host responsibilities. | Connect a fake 8237-style DMA adapter and PIC. Check exact transfer counts, terminal count, 64 KiB page behavior, IRQ source bits, acknowledgement and auto-init exit. |
| SB16 FIFO and stereo cadence | Separate output/input rates and left/right PCM words. FIFO depth and exact bus cadence are not modeled. | Drive the same generated stereo stream through a fake DMA adapter at several rates. Compare request count, timestamps, channel order and block IRQs with a documented software reference. |
| Mixer audio path | Register storage and masks only. | Generate one source at a time, exercise every documented register mask, then compare gain/filter output with a software reference if the project needs analog-like samples. |
| DSP clones and AWE models | Creative command set only. No clone extensions, ASP/CSP execution, EMU8000 wavetable or SoundFont Manager. | Add a profile only after a game's driver probes show which extension it needs. Compare generated probe responses against an allowed software target. |

These checks can run without a physical card. The crate's tests cover reset/version, 8-bit and
16-bit transfers, capture, rate selection, high-speed command gating, IRQ acknowledgement, mixer
register profiles, ADPCM reference-byte marking, timestamped SB-MIDI, UART reset restoration,
MPU-401 readiness and the OPL2/OPL3 card port maps. A DOS virtual-machine differential harness can
compare port traces without making hardware access a release gate.

## CT1741 firmware trace

A static Ghidra trace of CT1741 DSP firmware 4.05 recovered the ADPCM entry paths and their
byte-level state operations. The file used for the trace has SHA-256
`40cca2ff1a8caea58c0348bc6fa4191ed98bd31de8301719e6223b45ee4f22ab`. The [firmware archive](https://github.com/S95Sedan/CT1741_DSP)
identifies the image as Creative Labs property. The firmware, its assembly listing and Ghidra
output are not included here. The trace was static. It was not executed, and its sample output was
not compared with a software or hardware reference.

| Firmware address | Observed behavior |
|---|---|
| `0016`, `0186`, `022c`, `02d2` | The interrupt dispatcher selects separate 2-, 3- and 4-bit decode paths. Each path consumes sign and magnitude bits from a byte reservoir and writes the resulting byte to the external sample port. |
| `1056` | The 2-bit path has its own predictor and step branches. Its zero-magnitude cases differ by sign, so a generic ADPCM formula should not replace this path. |
| `10f8` | The 3-bit path reads a 2-bit tail when its counter reaches one. This gives the final sample in a packed byte fewer magnitude bits. |
| `10ad` | The 4-bit path reads the high nibble, then the low nibble. It forms a delta from the magnitude, step and half-step, then clamps the predictor to one byte. |
| `0dce`, `0e52` | The reference-byte paths initialize the predictor from the input byte and set the step to one. |

The static disassembly supports the current decoder contract. It does not prove sample parity.
`1056` reads a sign bit followed by one magnitude bit. A zero magnitude uses half the step and
halves the step with a minimum of one. A one magnitude uses one-and-a-half times the step and
doubles the step up to 32. The predictor clamps to an unsigned byte.

`10ad` consumes the high nibble before the low nibble. The high bit is sign and the next three bits
are magnitude. Its delta is `magnitude * step + floor(step / 2)`. Magnitude zero halves the step
with a minimum of one. Magnitudes five through seven double it, capped at eight. The predictor
clamps to an unsigned byte.

`10f8` consumes two 3-bit codes and a final 2-bit code from each input byte. Each code starts with
sign. The remaining bits are magnitude. The delta is `magnitude * step + floor(step / 2)`. A zero
magnitude halves the step with a minimum of one. Magnitudes one and two set the step to that
magnitude. Magnitude three doubles the step, except a step of 16 changes to 3. The predictor
clamps to an unsigned byte.

The DSP model requests a reference byte on the first output tick when that command includes one.
It requests the first compressed byte on the next output tick, then requests each later compressed
byte every four, three or two output ticks for 2-, 3- or 4-bit mode. Each `AdpcmByte` event returns
the decoded samples in order and the output rate. The formulas above come from static instructions
and have not been compared with executed firmware. A software-only check should run each helper with
the firmware register bank and a stub sample port. Vectors should cover both signs, every
magnitude, predictor values near zero and 255, step changes, byte boundaries, the 3-bit tail,
both nibbles, continuation blocks and DMA request intervals. Physical testing is optional.

## Source

- Creative, [*Developer Kit for Sound Blaster Series, Second Edition, Hardware Programming
  Reference*](https://www.ardent-tool.com/sound/Dev_Kit_SB_Series_Hardware_Programming_Ref_DSA0085515.pdf):
  chapters 2–6 and appendix A.
- Creative, [*Sound Blaster Series Hardware Programming Guide*](https://www.ardent-tool.com/sound/Sound_Blaster_HW_Programming_Guide_1st.pdf):
  profile identification and Appendix A's Pro, Pro 2 and SB16 FM port maps.
- Creative, [SBBASIC.EXE support listing](https://cn.creative.com/support/downloads/download.asp?DriverLang=2052&searchString=SBBASIC.EXE):
  package identified for later compatibility work; its included license prevents reverse
  engineering.
- [CT1741 DSP firmware archive](https://github.com/S95Sedan/CT1741_DSP): v4.05 firmware binary
  used for the static 8051 trace above; firmware ownership remains with Creative Labs.

Licence: CC0-1.0.
