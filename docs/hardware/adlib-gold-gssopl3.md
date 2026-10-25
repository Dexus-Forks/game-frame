# AdLib Gold OPL3 and MMA Windows drivers

This page records a static analysis of `GSSOPL3.DRV` from the AdLib Gold 1000 driver package, version 1.01. The 16-bit Windows driver binary was inspected with Ghidra pseudocode and disassembly. It was not run under Windows or tested with an AdLib Gold card. The [Ad Lib Gold Developer Toolkit v1.01](https://slartibardfast.github.io/adlib_gold/disks-md/developer-toolkit-v1.01/installed/SDTK.DOC.html), Hardware Reference 4.1, supplies the card-level port and Control Chip specification. Together, these sources establish the bank switch, status and register map. They do not establish analog output or sample parity.

The package is listed by [VGMPF](https://www.vgmpf.com/Wiki/index.php/AdLib_Gold_1000) and [VOGONS Drivers](https://www.vogonsdrivers.com/getfile.php?fileid=641). The inspected ZIP was `AdLib_Gold_1000_v1.01_-_Drivers.zip`, SHA-256 `bf42c2c27e420e8186b19110ac33a28b7862d51136fe389c833b511efd1e72e6`. The extracted `GSSOPL3.DRV` was 20,704 bytes, SHA-256 `9caa323b69a5978242456075b4ce7b71fc4ff0cf45c11730c31bcc271564c3a1`. The binaries and Ghidra exports remain outside this repository.

## Observed driver paths

| Address | Finding | Evidence |
|---|---|---|
| `1000:1402` | The driver accepts one of eight configuration profiles. It selects a profile at `0xE6 + profile * 0x5F`. | The function bounds the selector below 8 and computes the profile address with multiply by `0x5F`. |
| `1000:13b6` | The register writer keeps a shadow byte for each register in each bank. It uses the driver's base I/O port for bank zero and base plus `2` for bank one. It writes the register index, reads that port four times, writes the data at the next port, then reads that port four times. The read values are discarded. | Ghidra disassembly shows the two shadow regions, port selection, `OUT` instructions and eight `IN` instructions. |
| `1000:059a` | This helper changes a software latch once per enable/disable transition. It writes `0xFF` when enabling and `0xFE` when disabling to the second index port, then runs a short delay loop. The Developer Toolkit identifies these values as entering and leaving the Control Chip's phantom-register bank. | The routine branches on a state byte, writes `DAT_1020_00e4 + 2`, and delays. The Toolkit's Hardware Reference 4.1, “Register Access,” documents the `0xFF`/`0xFE` protocol. |
| `1000:05da` | This helper polls bit `0x40` at the second index port for up to 8,191 reads, writes its second argument as an index, waits, writes its first argument as data, then waits again. The Toolkit defines bit 6 as Control Chip busy while writing a register or its mixer/volume devices. | Pseudocode shows the bounded loop, status mask, index/data order and delay loops. The Toolkit's “Status Register” section defines bit 6. |
| `1000:0608` | This helper performs the same bounded poll and index/data writes as `1000:05da`, with a much longer delay after the data write. | The function body has the same port sequence and a `0x63C0`-iteration post-write loop. |
| `1000:0636` | This helper writes an index to the second index port and immediately reads the paired data port. The Toolkit specifies that, in Control Chip mode, this returns the selected register's current value. | The function consists of one `OUT` to `DAT_1020_00e4 + 2` and one `IN` from `DAT_1020_00e4 + 3`. Toolkit “Register Access” documents the address/data ports. |
| `1000:064c` | This helper polls bit `0x40` at the second index port for up to 8,191 reads, writes an index, waits, then reads the paired data port. | Pseudocode shows the bounded poll and read sequence. The Toolkit maps Control Chip registers `0x09` and `0x0A` to left and right FM volume. |
| `1000:1426` | Initialization writes `0x04` to bank-zero register `0xE0` and `0x08` to `0x40`. It writes `0xBD` from a configuration field. It sets bank-one register `0x05` bit 0 before configuring register `0x04`. One configuration path passes a byte from the selected configuration record to `0x104`. The other writes zero to `0x104`, then clears bit 0 of `0x105`. | The function calls the register writer at `1000:13b6` with those values. Yamaha documents `0x104` as the four-operator connection select and `0x105` bit 0 as native OPL3 enable in the [YMF262 datasheet](https://bitsavers.trailing-edge.com/components/yamaha/YMF262_199411.pdf). |
| `1000:1492` | Initialization walks three configured voice pools. The count at profile offset `+4` selects the two-operator pool, which has up to 18 slots. The count at `+2` selects the four-operator pool, which has up to 6 slots. The count at `+6` selects the five rhythm percussion slots. | The three loops call separate programming and note-update routines. `1000:095e` clears storage for 18 melodic records and 6 paired records. The rhythm path addresses the five percussion bits described in Yamaha's datasheet. |
| `1000:151c` | This routine initializes a two-operator voice. It writes the parameter groups for two operators, then the channel connection value. | It is called once per entry in the `+4` pool. Its loop writes two operator registers per parameter group. |
| `1000:1600` | This routine initializes a four-operator voice across a paired channel set. It writes parameter groups for four operators and both channel connection values. | It is called once per entry in the `+2` pool. Its loop writes four operator registers per parameter group. |
| `1000:173e` | This routine initializes one of the rhythm percussion entries. | It is called once per entry in the `+6` pool and is followed by the rhythm-key update at `1000:19aa`. |
| `1000:185a` | The note conversion divides the input note by 12 and combines the octave with a semitone lookup value. | Ghidra pseudocode and disassembly show division and the lookup-table access. The table values were not copied into this repository. |
| `1000:189c` | This routine updates a two-operator voice. A zero note clears its key-on bit. A nonzero note is converted to an F-number and sets the key-on bit. | It indexes the profile's `+4` pool and is called from the MIDI note path. |
| `1000:192c` | This routine updates a four-operator voice with the same zero-note and nonzero-note behavior. | It indexes the profile's `+2` pool and is called from the MIDI note path. |
| `1000:21a8` | This helper brackets a write to bank-one address `0x80 + (value >> 9)` with the `0xFF`/`0xFE` index toggle. It takes the data byte from a record at offset `+0x972`. | The helper calls `1000:059a` to enable and disable around `1000:05da`. The `MODMESSAGE` path calls it for two records. |
| `1000:21de` | This helper brackets reads of Control Chip registers `0x09` and `0x0A` with the same index toggle, then runs a short delay. The toolkit identifies these as left and right FM volume. | The function calls `1000:059a` and `1000:064c` in that order. `1000:217e` stores the return. |

## What this guides

The driver confirms that OPL3 mode setup and pair selection belong in the hardware-facing layer. The `gameframe-opl3` crate models chip registers and sample output. The companion `gameframe-adlib-gold` crate overlays the documented Control Chip on the second OPL3 address/data pair, models its readback and busy status, and moves the eight-port audio block when register `0x15` is written. The Toolkit specifies the register map, active-low interrupt status, 450 μs writes to registers `0x04`–`0x08`, 5 μs writes to `0x09`–`0x16`, and a restore delay just under 2.5 ms. It does not give the EEPROM save duration; the crate makes that duration configurable.

The driver reads Control Chip registers `0x09` and `0x0A`, which the Toolkit identifies as left and right FM volume. The status read at the second index port is the Control Chip status while its bank is selected. The Toolkit also says that the AdLib Gold's D4/D5 stereo-left/right selectors must match between `Cx` and `Cx+3` for four-operator audio to reach that side. Yamaha's four-output `CHA`–`CHD` behavior across paired registers still needs a software reference. The driver trace and Toolkit explain the register reads but do not establish analog output or sample parity.

## MMA driver paths

The same v1.01 driver package contains `GSS.DRV`, a 17,984-byte Windows 3.x NE driver with SHA-256 `74fdb9f470c1dc89609e418c3680228fe28cbc0b8dd183752a46710988a42d60`. Its exports include `WIDMESSAGE`, `WODMESSAGE`, `MIDMESSAGE` and `AUXMESSAGE`. The driver and its Ghidra export remain outside this repository.

| Address | Static driver observation | Evidence limit |
|---|---|---|
| `1000:1975` | Writes a register index to the configured MMA base and data to base + 1. It updates a register shadow. | Static Ghidra pseudocode and disassembly. |
| `1000:19b1` | Writes an index to base + 2 and data to base + 3, with a separate register shadow. | Static Ghidra pseudocode and disassembly. This gives the Windows driver two index/data pairs even though the toolkit's low-level description presents a shared selector. |
| `1000:1460` | Selects one of four sample-rate settings and writes the rate bits to channel register `0x09`. | The branch thresholds and written values are static; no sample-rate output was measured. |
| `1000:14e8`, `1000:15b0` | Configure channel control register `0x09`, format/DMA register `0x0c`, and start/stop channel sampling. | Static calls and register values only. |
| `1000:1630`, `1000:165e` | Pulse bits `0x02` and `0x08` in register `0x0d` to reset the MIDI transmit and receive paths. | The driver names these routines as MIDI reset paths; UART line timing was not measured. |
| `1000:1690` | Initializes timer registers `0x02`–`0x08`, both channel control registers, format register `0x0c` and MIDI control register `0x0d`. | Static initialization sequence only. |
| `1000:17b2`, `1000:17e2`, `1000:1802`, `1000:1876`, `1000:18c8` | Configure and control the timer counters, base counter and timer masks. | Static call paths; no IRQ cadence was measured. |

The Toolkit defines the MMA status byte as overrun in bit 7, timer 2/1/0 in bits 6/5/4, MIDI
transmit-empty and receive-ready in bits 3/2, and channel 1/0 FIFO threshold status in bits 1/0.
Register `0x0d` masks overrun, MIDI, transmit-request and receive-request interrupts. The original
driver's startup value `0x35` sets the four mask bits, while its reset helpers pulse bits `0x02`
and `0x08`.

The `gameframe-ymz263` crate implements the digital FIFO, sample, timer and UART paths from the Yamaha data sheet and toolkit, with driver register use as additional evidence. The card adapter maps the MMA to audio-block offsets `+4` through `+7` and feeds its sampling request into Control Chip interrupt status. The model does not include the external DMA or interrupt controllers.

`gameframe-adlib-gold::mix_sources` now applies the documented source-volume controls at registers `0x09`–`0x10`, PC-speaker enable and microphone feedback at `0x11`, left/right source selection, forced mono, mute and final channel-volume levels at `0x04`–`0x08`. The telephone sample path requires both the telephone option and the register `0x01` line-engage bit. This is a linear digital mixer. The TDA8425 tone and pseudo/spatial phase response, input anti-alias filters and the YAC512 analog output are still not reproduced.

The selected configuration byte passed to register `0x104` gives a binary-backed path into four-operator support. The YMF262 datasheet maps the six pairs to channels `0+3`, `1+4`, `2+5` in each register bank and shows four connection algorithms. The `gameframe-opl3` model now uses that mask and those algorithms. The driver trace confirms separate 18-slot two-operator and 6-slot four-operator pools. It does not provide output-parity evidence. Do not copy the binary's patch data into GameFrame.

## Software-only checks

Current crate tests exercise the documented busy deadlines, EEPROM save/restore, audio-block
relocation, active-low interrupt status, MMA sampling interrupt routing, linear mixer controls,
PCM layouts, FIFO thresholds, timers and MIDI timing. These tests check the software model. They do
not establish parity with the original driver or analog card output.

- Run the original 16-bit driver functions under a port-instrumented NE emulator. Record the
  `0xFF`/`0xFE` bank changes, status byte, selected Control Chip registers and returned data.
  Keep the driver binary and traces outside GameFrame.
- Exercise the card adapter at timestamps around the specified 32-clock YMF262 spacing, 5 μs
  Control Chip busy period, 450 μs mixer/volume busy period and 2.5 ms restore bound. Confirm
  that status bits change at their deadlines and that writes during busy periods are reported.
- Read and write every documented Control Chip register. Check ID option bits, telephone TC/ring
  status, interrupt inputs, the unused `0x12` register, EEPROM save/restore, and `0x15` relocation.
  Use only the new crate and synthetic port sequences for this first check.
- Run the original `WIDMESSAGE`, `WODMESSAGE`, `MIDMESSAGE` and `AUXMESSAGE` handlers in an
  instrumented NE emulator. Record register selection, data reads/writes, DMA buffer positions,
  IRQ callbacks, sample formats and timer cadence. Keep the driver and traces outside GameFrame.
- Exercise both FIFO windows with generated streams at 8-bit and each documented 12-bit layout.
  Check empty/full transitions, threshold points, reset, underflow/overflow, both channel rates,
  PCM and ADPCM playback/recording, channel routing and interleave mode.
- Feed generated FM, sampler, auxiliary, microphone, telephone and PC-speaker samples through
  `mix_sources()`. Check the documented source-volume range (128 silent, 255 unity), polarity for
  lower source codes, telephone option/engage gating, microphone feedback mute, left/right source
  selection, forced mono, master volume and output mute. Check that bass/treble and pseudo/spatial
  selections are surfaced as controls without claiming analog parity.
- Compare one-channel PCM output against a software-instrumented driver capture or a named
  software emulator. Use synthetic timer reloads to compare status and interrupt cadence.
- Fill and drain both 16-byte MIDI FIFOs. Check reset pulses, request masks and the transmitted
  31,250-baud byte schedule against the driver or a timestamped software UART capture.
- Exercise every timestamp boundary: eight YMZ263 master clocks between MMA port accesses and
  conversion from the card's YMF262 clock. Check audio-block relocation and active-low sampling
  status through the Control Chip.

These checks need no physical sound card. If no software reference exposes a card-specific result,
leave that behavior as an open question instead of inferring it.

## Open questions

- Exact EEPROM save duration and reset-time register defaults for each AdLib Gold/GSS revision.
- The analog gain curve, TDA8425 transfer behavior, bass/treble filter response and volume-chip
  settling behavior.
- Card-level routing and sample conversion for the MMA sampler, YAC512 output, PC speaker,
  microphone, telephone and optional surround processor.
- Whether the four-read delays in this Windows driver match the physical card's busy timing on
  every supported machine.
- Whether initialization and MIDI output match an accepted software reference capture.
- YMZ263-specific ADPCM predictor clipping and rounding, exact FIFO threshold transition behavior,
  timer flag clearing, standby clock reduction and joystick RC timing. Interleave follows the toolkit
  description; compare it and the threshold behavior against a software trace before claiming
  parity.
- Analog ADC/DAC conversion, board mixer/filter response, DMA controller transfers and host IRQ
  delivery. Suggested software checks above can settle digital control paths; analog board behavior
  needs a published transfer specification or a software black-box model. Hardware testing is not
  a blocker.

All findings above are static observations unless a question is explicitly left open. No runtime behavior or sample parity has been measured.

## Related Wave/MIDI driver

The configuration helper at `1008:0594` selects the MMA register block at `0x38C`, matching the
Developer Toolkit's default audio-port map. The static trace does not establish full DMA behavior,
PCM output parity or IRQ timing. Those remain software-only follow-up items above.

Licence: CC0-1.0.
