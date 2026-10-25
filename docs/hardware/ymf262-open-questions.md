# YMF262 open questions

This page records chip behavior that still needs reliable evidence. These questions do not block
the documented OPL3 feature set or require a physical card. A software reference can resolve them.

## Four-operator output routing

### What is known

Yamaha's 1994 [YMF262 datasheet](https://bitsavers.trailing-edge.com/components/yamaha/YMF262_199411.pdf)
shows each four-operator algorithm's carrier signals combined at one voice output. The application
manual lists `C0` and `C3` as paired channel settings and says only the feedback field in `C0`–`C2`
is effective in four-operator mode. The [Ad Lib Gold Developer Toolkit](https://slartibardfast.github.io/adlib_gold/disks-md/developer-toolkit-v1.01/installed/SDTK.DOC.html)
says the card's stereo-left and stereo-right bits (D4 and D5 in `Cx`) must match in `Cx+3`; a
four-operator voice is disabled for a side when the bits do not match. This resolves the driver's
stereo rule. Yamaha's datasheet names four output selectors, `CHA` through `CHD`, but does not say
how both `Cx` and `Cx+3` route a combined voice across all four digital outputs.

The current `gameframe-opl3` model sums a four-operator pair into one voice output and applies the
first channel's route nibble. The exact four-output behavior remains unconfirmed. Keep the
documented AdLib Gold left/right rule separate from the unresolved mapping to YMF262 outputs A–D.

### Software reference test

No physical card is needed. Use a YMF262 reference that exposes all four digital outputs, or that
can isolate A/B and C/D into separate stereo captures.

1. Enable native mode with `0x105 = 1` and pair channels 0 and 3 with `0x104 = 1`.
2. Program the four-operator algorithms in `C0` and `C3`. Use algorithms 0, 1, 2 and 3, setting
   the upper connection bit in `C0` and the lower connection bit in `C3`.
3. Set the Toolkit's D4/D5 stereo bits in `C0` and `C3` to match, then mismatch one bit at a time.
   Confirm the documented left/right mute rule. Keep this result distinct from the four-output
   routing check.
4. Set the four Yamaha output selectors in `C0` and `C3` to A, B, C, D, or mismatched pairs.
   Capture each output. Repeat with the selectors equal, swapped and individually muted.
5. For each algorithm, mute selected operators with total level and leave one carrier audible at
   a time. This shows whether output selectors follow the first channel, the second channel, or a
   carrier group.
6. Repeat with a second pair in bank one to check that the result is the same in both banks.

Compare the four output words for each register sequence with `FourChannelSample`. Keep the
reference binary and captures outside GameFrame. Record the reference name, version, settings and
sample hashes before changing the implementation.

## YAC512 floating DAC transfer

The new `gameframe-yac512` crate models the digital boundary. Yamaha's [YAC512 datasheet](https://www.quarter-dev.info/archives/yamaha/YAC512.pdf)
specifies a 16-bit serial input, FSEL high for offset binary and low for two's complement, and
falling SMP1/SMP2 edges that latch channels 2/1. The YMF262 stream sends 16-bit offset-binary
words LSB first. Yamaha describes a 10-bit DAC mantissa and seven-step analog shifter but does not
provide the input-to-floating conversion formula in the available datasheet.

### Software-only checks

1. For both FSEL modes, feed patterns `0x0000`, `0x0001`, `0x7fff`, `0x8000` and `0xffff` one bit
   at a time, least-significant bit first. Check the latched raw word and its signed input
   interpretation.
2. Pulse SMP1 and SMP2 independently. Confirm SMP1 updates channel 2, SMP2 updates channel 1, and
   the other channel keeps its previous word. Change FSEL between two latches and confirm that the
   first word retains its captured format.
3. Feed each YMF262 frame's `DOAB` and `DOCD` serial levels through separate receivers using the
   board's documented SMP wiring. Verify the four raw words and channel ordering across multiple
   frames. The two unspecified clocks per word may use either logic level; the next full 16-bit
   word replaces them before its latch.
4. To implement the floating transfer, first obtain a reliable formula or a named black-box
   reference with YAC512 conversion. Compare all 65,536 input codes in both FSEL modes. If no
   reference or formula is available, keep the analog conversion explicitly open; this does not
   block the digital receiver or the YMF262 chip.

Physical DAC testing is optional. These checks do not depend on a physical sound card.

## Remaining checks

- Compare the YMF262 waveform levels and envelope timing with a named software reference. Physical
  hardware testing is optional.
- Check whether the selected software reference exposes the four native outputs. If it provides
  only stereo, isolate each output pair through the reference's documented DAC wiring.
- Keep YAC512 analog conversion and card-specific filtering separate from the digital YMF262
  model unless a game-facing requirement needs them.
- The AdLib Gold card's MMA (`YMZ263`) sample, MIDI and timer paths are implemented in
  `gameframe-ymz263` and connected through `gameframe-adlib-gold`. Software tests cover generated
  PCM, FIFO thresholds, timers, MIDI timing and card interrupt status. Compare those paths with the
  original GSS Wave/MIDI driver under an instrumented NE emulator. Record port transactions, DMA
  setup, FIFO requests and timer/IRQ cadence, then compare those traces with synthetic sequences.
  No physical card is required.

Licence: CC0-1.0.
