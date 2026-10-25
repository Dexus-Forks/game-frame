# Creative Music System and SAA1099

This page records the SAA1099 chip model and the Creative Music System (CMS), also sold as Game
Blaster. The model follows Philips documentation and static disassembly of Creative's `CMSDRV.COM`
and Sierra's `CMS35.BIN` from the Creative CMS Game Blaster driver disk. The binaries were not run.
No physical card or sound-output comparison was used, and neither is required to keep implementing
the model.

## Evidence

| Finding | Evidence | Status |
|---|---|---|
| Six tone generators each have eight octave choices and 256 tone values. The data sheet gives a 31 Hz to 7.81 kHz range at an 8 MHz clock. | [Philips SAA1099 data handbook](https://bitsavers.org/components/philips/_dataBooks/1986_IC01_Philips_Radio_Audio_and_Associated_Systems.pdf), pp. 143 and 152 | Observed in the data sheet |
| Tone and octave writes take effect on a transition of the associated generator, up to half a period after the write. Write tone before octave when both change. With reset asserted, pending tone and octave writes wait for half a period of the old frequency after reset is released. | Philips, [“Microprocessor controlled stereo sound generator for sound effects and music synthesis”](https://www.worldradiohistory.com/Archive-Company-Publications/Philips-Technical-Review/Electronic-Compoinents/Electronic-Components-%26-Apps-Vol-8-No-1.pdf), *Electronic Components and Applications*, Vol. 8 No. 1 (1986), pp. 25–26 | Observed in Philips documentation |
| The tone formula is `clock / 512 * 2^octave / (511 - tone)`. | Derived from the documented octave ranges and divider values | Inferred; generated frequency vectors can confirm it |
| Registers 00–05 set stereo amplitudes, 08–0D set tone values, 10–12 set paired octaves, 14 enables tone, 15 enables noise, 16 selects noise clocks, 18–19 control envelopes, and 1C controls generator reset and global sound enable. | Philips SAA1099 data handbook, pp. 151–153 | Observed in the data sheet |
| Envelope enable and resolution act immediately. Waveform, clock-source and inversion controls are buffered until the current envelope reaches its boundary. External clock pulses occur only when address 18h or 19h is written; the data byte is irrelevant. | Philips data handbook, pp. 153–155; Philips ECA article, pp. 25–26 | Observed in Philips documentation |
| `CMS35.BIN` writes register numbers to the port one above the selected chip's data port, then writes values to its data port. It initializes amplitude, tone, octave, enable and noise registers. | Static disassembly at 1000:0E1A–0F10; base at `CS:0035`, upper data-port bound at `CS:0037` | Observed in the original driver |
| `CMSDRV.COM` uses the base stored at 1000:010F. Its register writer advances the data port by two between chips and uses the adjacent port for register selection. Its detection path reads from base+0Ah. | Static disassembly at 1000:0CE4–0D0D, 0D84–0DAE, 0683–06CA and 08A8–08D2 | Observed in the original driver |
| The driver disk documentation names 220h as the default music-driver I/O address and says `SG-INST.EXE` changes it. | `GAMEDRV/DRV-DOC.TXT`, sections C and D | Observed in the original driver documentation |
| A compatible card manual documents base+4 as 7Fh and auxiliary writes at base+6/+7 echoed from base+0Ah/+0Bh. It says the original custom IC's exact behavior is not fully known. | [TexElec SAAYM manual](https://www.opl3.com/wp-content/uploads/2020/05/TexElec-SAAYM-Manual.pdf), pp. 3–4 | Secondary compatibility evidence; implemented as a profile, not a confirmed original-IC model |
| A Game Blaster-compatible card manual reports a 7.15909 MHz chip clock; Philips gives 8 MHz as the nominal chip clock. | TexElec SAAYM manual, pp. 2–3; Philips data handbook | Board-specific report; configurable |
| An 18-bit noise generator with polynomial `x^18 + x^11 + x` is reported as measured against an SAA1099P. | [SAA1099 hardware-measurement project](https://github.com/rejunity/tt06-psg-saa1099) | External measurement report; the model's starting seed remains unknown |

The Philips data handbook is the primary chip reference. The [CMS Game Blaster driver disk entry](https://vogonsdrivers.com/getfile.php?fileid=5&menustate=0)
in the VOGONS Vintage Driver Library identifies the disk used for the static trace. Driver files
and Ghidra output remain outside this repository.

## Model coverage

`gameframe-saa1099` models the 32-register write-only interface, six tone generators, the documented
frequency formula, two noise generators, six stereo amplitude controls, mixer enables, two envelope
controllers, reset, global sound enable and normalized stereo output. Tone and octave values are
latched at the next generator transition. The transition is resolved at the next output step, so
timing is quantized to `clock_hz / 128` rather than to individual master-clock ticks.

The `CmsCard` adapter maps two chips into the four-port window at the configured base. Its default
base is 220h, matching the Creative driver notes, and its default clock uses the reported CMS board
clock. Callers can select another base or clock. The adapter also models base+4 as 7Fh and the
base+6/+7 to base+0Ah/+0Bh latch echo described by the compatible-card manual. This common probe
behavior is useful for software compatibility, but it does not reconstruct the original custom IC
state machine.

The envelope model applies immediate enable and resolution controls and buffers waveform,
clock-source and inversion changes until an envelope boundary. Internal clocks use frequency
generator 1 or 4. External clocks use address writes to register 18h or 19h. The 18-bit noise
polynomial follows the external hardware-measurement report; the seed of one is a deterministic
choice. The combined tone/noise gate uses AND as an inference.

## Open behavior

- The Philips data sheet does not state the noise generator's power-up seed. Compare candidate seeds
  against a named software output or available recording before treating one as original behavior.
- Philips names a combined tone/noise mode but does not specify the Boolean gate. Compare all four
  mixer selections against a versioned software reference and record the first differing output
  step.
- The original CMS custom detection IC is not fully documented. The adapter implements the common
  latch echo and 7Fh status response in a compatibility profile. A driver trace can check the
  `base+0Ah` probe, but it cannot prove every original board revision's response.
- The SAA1099 has a DTACK output for bus timing, including waits for closely spaced amplitude
  writes. The chip and card API do not yet accept CPU cycle timestamps, so this adapter cannot add
  ISA wait states. A future timestamped I/O wrapper can model the data-sheet timing without
  requiring a physical card.
- The chip's PWM carrier, current-sink levels, analog filter and CMS board output circuit are not
  modeled. The crate returns normalized digital samples at `clock_hz / 128`.
- The tone/noise and sample models do not yet resolve transitions at individual master clocks.
  Output-step timing is adequate for audio generation but can move a register-change edge by up to
  one output interval.

## Software-only checks

Physical card testing is optional. These checks can advance the open points without it:

1. Generate tone vectors for all octave and tone boundaries. Check the periods against the Philips
   formula at 8 MHz and at the CMS profile clock.
2. Write tone and octave values while a tone is active. Confirm output keeps the old frequency
   through the next half-period transition. Repeat with reset held, overwrite the values several
   times, release reset, and confirm only the final values are heard after the old frequency's
   half-period.
3. Drive all eight envelope modes in both resolutions. Change buffered fields midway through each
   one-shot and repeating shape, and check that changes take effect at the boundaries in Philips
   Fig. 5. Write addresses 18h, 19h and a non-envelope address to check external clock pulses.
4. Run the original CMS driver under DOSBox-X or another instrumented DOS emulator. Record writes
   and reads for both chip pairs, the relocated base, and the `base+0Ah` detection sequence. Feed the
   same sequence to `CmsCard` and compare the driver's branch/result.
5. Compare noise-only and mixed tone/noise output with a named, versioned SAA1099 emulator binary.
   Keep its source unread and record the register stream, sample rate and first differing sample.
6. Use a cycle-counting fake I/O bus to exercise address and amplitude writes against the
   data-sheet DTACK requirements when a timestamped card API is added.
7. Compare normalized stereo samples with a software reference to estimate whether PWM averaging
   or a low-pass output stage is needed for the intended host audio path.

The current code has compile and formatting checks only. No runtime parity result is claimed.
