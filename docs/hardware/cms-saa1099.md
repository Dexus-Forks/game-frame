# Creative Music System and SAA1099

This page records a clean model of the Philips SAA1099 and the Creative Music System (CMS), also
sold as Game Blaster. The model follows Philips's 1986 data handbook and static disassembly of
Creative's `CMSDRV.COM` and Sierra's `CMS35.BIN` from the Creative CMS Game Blaster 720KB driver
disk. The binaries were not run. No physical card or sound-output comparison was used.

## Evidence

| Finding | Evidence | Status |
|---|---|---|
| Six tone generators each have eight octave choices and 256 tone values. The data sheet gives a 31 Hz to 7.81 kHz range at an 8 MHz clock. | Philips SAA1099 data handbook, pp. 143 and 152 | Observed in the data sheet |
| The tone register maps to `clock / 512 * 2^octave / (511 - tone)`. | Derived from the documented octave ranges and divider values | Inferred; generated frequency vectors can confirm it |
| Registers 00–05 set stereo amplitudes, 08–0D set tone values, 10–12 set paired octaves, 14 enables tone, 15 enables noise, 16 selects noise clocks, 18–19 control envelopes, and 1C controls generator reset and global sound enable. | Philips SAA1099 data handbook, pp. 151–153 | Observed in the data sheet |
| The write-only address/data interface keeps the selected register active across repeated data writes. Register addresses repeat every 32 bytes. | Philips SAA1099 data handbook, p. 150 | Observed in the data sheet |
| `CMS35.BIN` writes register numbers to the port one above the selected chip's data port, then writes values to its data port. It initializes amplitude, tone, octave, enable and noise registers. | Static disassembly at 1000:0E1A–0F10; base at `CS:0035`, upper data-port bound at `CS:0037` | Observed in the original driver |
| `CMSDRV.COM` uses the base stored at 1000:010F. Its register writer advances the data port by two between chips and uses the adjacent port for register selection. | Static disassembly at 1000:0CE4–0D0D and 0D84–0DAE; bound at 1000:0111 | Observed in the original driver |
| The driver disk documentation names `220h` as the default music-driver I/O address and says `SG-INST.EXE` changes it. | `GAMEDRV/DRV-DOC.TXT`, sections C and D | Observed in the original driver documentation |
| Game Blaster boards use a 7.15909 MHz chip clock. | A [separate SAA1099 hardware-design project](https://github.com/rejunity/tt06-psg-saa1099) reports this board clock; the CMS driver does not establish it | Reported externally; configurable and still awaiting a primary board source |

The [Philips SAA1099 data handbook](https://bitsavers.org/components/philips/_dataBooks/1986_IC01_Philips_Radio_Audio_and_Associated_Systems.pdf)
is the primary chip reference. The [CMS Game Blaster 720KB driver disk entry](https://vogonsdrivers.com/getfile.php?fileid=5&menustate=0)
in the VOGONS Vintage Driver Library identifies the disk used for the static trace. The driver
files and Ghidra output remain outside this repository.

## Model coverage

`gameframe-saa1099` models the chip's register file, write-only address/data interface, six tone
phases, noise-rate controls, two envelope controllers, stereo amplitude controls and global sound
enable. It produces normalized stereo samples at one step per 128 master clocks. The `CmsCard`
adapter maps the two chips into the four-port write window at the configured base. Its default base
is `220h`; each chip's selector is at data-port plus one. The default clock is configurable. The
CMS profile uses 7.15909 MHz based on a secondary [hardware-design report](https://github.com/rejunity/tt06-psg-saa1099);
the driver trace does not establish the oscillator.

The Philips data sheet documents the fixed noise rates, the tone-derived noise rate, the eight
envelope shapes, the internal and external envelope clocks, and the amplitude LSB behavior while an
envelope is enabled. The model applies those rules. It keeps the address latch active across data
writes, as the chip requires.

## Open behavior

- Philips does not publish the noise shift-register polynomial or power-up state. The crate uses a
  deterministic 18-bit candidate generator with a seed of one. Its sequence is not confirmed.
- The data sheet names a tone-plus-noise mixer mode without specifying the gate equation. The crate
  currently uses an AND gate. This is an inference.
- The resolution bit takes effect immediately. Philips does not describe how a mid-envelope change
  maps the current counter value to the new range. The model clamps the visible level until the
  envelope advances.
- The chip uses pulse-width modulation at a minimum rate of 62.5 kHz, followed by an analog current
  sink and an external low-pass filter. The crate returns normalized digital levels. It does not
  model the PWM carrier, current levels, or filter response.
- The CMS driver reads from `base + 0Ah` while checking for a card. The current adapter leaves reads
  open because the driver trace alone does not establish the card-detection transceiver response.
- The SAA1099 data sheet describes `DTACK` wait behavior for closely spaced amplitude writes. The
  CMS port adapter does not model a bus acknowledge pin or CPU wait states.
- The 7.15909 MHz CMS clock comes from a secondary hardware-design report. The driver trace does not
  establish it. Callers can select a board-specific clock with `with_clock`.

## Software-only checks

Physical card testing is optional. It does not block this model. These comparisons can narrow the
open behavior without using another emulator's source code:

1. Run `CMSDRV.COM` and `CMS35.BIN` in DOSBox-X with I/O tracing. Compare the original port and
   byte sequence with `CmsCard::write_port`, including both chip pairs and the relocated base.
2. Generate tone-register vectors for every octave and tone boundary. Compare periods with the
   data-sheet ranges at 8 MHz, then repeat with the Game Blaster clock profile.
3. Drive all eight envelope modes in both resolutions. Include direct enable and resolution
   changes, buffered mode changes, internal clocks from channels 1 and 4, and one address-write
   pulse per external clock.
4. Compare noise-only and tone-plus-noise output against a named, versioned SAA1099 emulator
   binary. Keep the oracle's source unread and record the first differing sample and register
   sequence.
5. Exercise independent writes to both chips and verify that the CMS card mixes their left and
   right sample streams without cross-writing registers.
6. Trace the driver's `base + 0Ah` read sequence and identify the expected card-detection response
   before adding it to the adapter.

The crate currently passes `cargo fmt` and `cargo check`. It has no runtime parity result yet.
