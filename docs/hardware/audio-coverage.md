# PC audio coverage

GameFrame's audio work has four layers. A chip model produces samples. A card model adds the
registers, memory and timing the game probes. A library replacement lets an unchanged game
reach the card through the interface it already calls. The host audio backend plays the
finished samples. Keeping these layers separate makes one chip useful to AdLib, Sound Blaster
and later compatible cards.

This page covers common IBM PC audio from the PC speaker through the 1990s Windows cards. It
tracks what GameFrame has, what an existing runtime can provide while observing a game, and the
next reusable pieces to build. DOSBox-X's [sound card guide](https://dosbox-x.com/wiki/Guide%3ASound-card-support-in-DOSBox%E2%80%90X)
has a broader device table and configuration details. Use [86Box](../observe/86box.md) when
observations need a complete period PC and configurable card resources.

## Device families and GameFrame coverage

| Era | Device family | What the game talks to | GameFrame coverage |
|---|---|---|---|
| 1981 onward | PC speaker | PIT channel 2 and the speaker gate; some games use rapid toggles for samples | DOSBox-X can run and record these games. No reusable PC speaker and PIT audio crate yet. |
| 1984 onward | IBM PCjr and Tandy 1000 | TI SN76496 tone/noise; Tandy models also use NCR 8496, and later PSSJ hardware adds an 8-bit DAC | `gameframe-sn76496` models the TI profile, PCjr `C0h` and original Tandy `C0h`–`C7h` writes, PCjr and Tandy `61h` output muxes and Tandy PIT-2 input. The Tandy mux's fourth input, NCR 8496, PSSJ/DAC, noise details and analog behavior remain open. |
| 1987 onward | Covox Speech Thing and Disney Sound Source | Writes to an LPT port drive an 8-bit DAC; the Disney device also has a small FIFO | DOSBox-X can provide LPT DAC emulation. No GameFrame DAC or FIFO model yet. |
| 1987 onward | Creative Music System / Game Blaster | Two SAA1099 chips provide square-wave voices, noise and stereo output | `gameframe-saa1099` models register timing, envelope boundaries, the chip ports and common CMS detection ports. Noise seed, combined mixing, exact custom-chip behavior, bus waits and analog output have software-only follow-up steps. See the [CMS/SAA1099 findings](cms-saa1099.md). |
| 1987 onward | AdLib and Sound Blaster 1.x/2.0 | YM3812 OPL2 register writes for music; Sound Blaster adds DSP commands, DMA and IRQ-driven PCM | `gameframe-opl2` models the YM3812 and has sample-for-sample evidence from Frontier: Elite II. `gameframe-sound-blaster` routes the OPL2 into its SB 1.5, SB 2.0 and SB 2.0 CD profiles, alongside documented DSP reset/status, 8-bit transfers, 2.x auto-init and high-speed modes, IRQ state and mixer registers. Its CT1741 firmware-derived decoder expands 2-, 3- and 4-bit ADPCM to PCM; generated-vector parity remains open. The host supplies ISA DMA and PIC behavior. Analog mixing and exact timing remain open. See the [Sound Blaster findings](sound-blaster-dsp.md). |
| 1991 onward | Sound Blaster Pro, Pro 2 and Sound Blaster 16; AdLib Gold | Stereo PCM and mixers; Pro 1 uses two OPL2 chips, later models use YMF262 OPL3; SB16 adds 16-bit DMA | `gameframe-sound-blaster` routes Pro 1's two OPL2 chips to stereo outputs and exposes Pro 2/SB16 YMF262 banks through card ports. `gameframe-opl3` models four-operator synthesis and four output words. The Sound Blaster mixer register maps, DSP, MIDI, SB16 MPU-401 UART and firmware-derived ADPCM decoder are covered. ADPCM sample parity remains open. The host supplies ISA DMA and PIC behavior. Analog mixer effects and exact bus timing remain open with software-only checks documented. `gameframe-yac512`, `gameframe-ymz263` and `gameframe-adlib-gold` model the separate AdLib Gold path; its remaining analog behaviors are in the [AdLib Gold findings](adlib-gold-gssopl3.md). Physical testing is not a release blocker. See the [Sound Blaster findings](sound-blaster-dsp.md). |
| 1992 onward | Gravis UltraSound, MAX and Plug and Play | GF1-family hardware voices read sample data from card RAM; drivers also use DMA and IRQs | DOSBox-X can run games configured for GUS. No reusable GF1/InterWave model yet. |
| 1991 onward | Pro AudioSpectrum, Windows Sound System and compatible ESS/Aztech cards | Stereo PCM through a codec; some models add OPL3, Sound Blaster compatibility or WSS registers | DOSBox-X can run several models. No GameFrame codec or card model yet. Record the exact board because compatible models vary. |
| 1987 onward | MPU-401 with Roland MT-32/CM-32L or General MIDI modules such as SC-55 | UART or intelligent-mode MIDI events; the attached synthesizer supplies the instruments and sound | The Maxi Sound wrapper acknowledges the limited MPU commands used by POD. Munt provides an external MT-32/CM-32L engine; GameFrame has no MIDI interface or synth crate. |
| 1990s DOS | Miles, HMI SOS, DIGPAK and similar sound libraries | A game calls middleware, which selects a card driver or music provider | The DOSBox-X harness can provide Miles and HMI SOS for running the original. No GameFrame middleware replacement yet. |
| 1992 onward | AWE32/AWE64 and other wavetable cards | EMU8000 voices, card RAM and SoundFont or bank uploads, often alongside Sound Blaster PCM | No AWE/SoundFont Manager replacement or wavetable voice model yet. |
| 1995 onward | Windows multimedia audio | WinMM `waveOut`/`midiOut`, DirectSound, DirectMusic and their buffers, streams and callbacks | Wine and Windows provide the standard APIs. `wrappers/maxi94` redirects one card library's wave streams through WinMM. |
| 1996 onward | Aureal A3D, Creative EAX, Sensaura, QSound and card-specific 3D audio | COM interfaces or DirectSound property sets add positional audio, occlusion and environmental effects | DSOAL already replaces DirectSound and supports EAX through OpenAL Soft. A3D COM compatibility and other proprietary interfaces are open. |
| 1996 onward | PCI cards such as Ensoniq AudioPCI, ESS AudioDrive and Yamaha DS-XG | Usually DirectSound/WinMM plus vendor extensions, software wavetable synths or legacy DOS compatibility | Use the host APIs for standard playback. Add a vendor layer only when a game's imports or call sites require it. |
| 1996–2000 | Dream SAM9407 cards, including Guillemot Maxi Sound 64 | `94DLL32.dll` exposes wave streams, sample banks, voices, mixer controls and MPU commands | `wrappers/maxi94` implements six exports used by POD and routes its two PCM streams to WinMM. It does not implement the card's banks, synth voices or effects. |

The dates mark the broad appearance of a device family. Later cards in the same family can use
different chips. Choose an emulator after recording the exact model, chip revision, driver, API
and behavior the game uses.

## What exists now

| Reusable piece | Where it runs | Scope and evidence |
|---|---|---|
| `gameframe-opl2` | In a port | YM3812 register writes to deterministic PCM samples. Its README documents what is modeled and the comparison against the original AdLib driver. |
| `gameframe-opl3` | In a port | YMF262 register and synthesis model: both banks, 18 channels, six four-operator pairs and their four algorithms, eight waveforms, OPL2 compatibility, shared note-select, timers/status, four sample outputs, serial DAC frames and indexed ports. It follows Yamaha's datasheet and the AdLib Gold driver's register use. Independent routing of carrier groups in four-operator mode and YAC512 analog conversion remain open. |
| `gameframe-saa1099` | In a port | Philips SAA1099 register and sample model, plus the CMS/Game Blaster two-chip port window and compatible detection-port profile. Static traces of `CMSDRV.COM` and `CMS35.BIN` establish address selection, port stepping, default `220h` base and a `base+0Ah` probe. Remaining noise, mixing, custom-chip, bus-wait and analog behavior have software-only follow-up checks. |
| `gameframe-sn76496` | In progress | TI SN76496 core with input-clock timing, PCjr `C0h` and original Tandy `C0h`–`C7h` adapters, PCjr and Tandy `61h` output muxes, Tandy PIT-2 input and approximate 42 write waits. Sierra driver traces show tone latch/data writes, attenuation writes, startup mutes and PSG-source selection. TI noise taps/seed, period zero, the Tandy mux's fourth input, NCR 8496/PSSJ variants, CPU wait mapping and analog behavior have software-only follow-up checks. |
| `gameframe-sound-blaster` | In a port | Creative DSP generations 1.xx–4.xx, 8-bit and SB16 16-bit digital I/O, mixer register files, DSP MIDI, SB16 MPU-401 UART and CT1741 firmware-derived 2/3/4-bit ADPCM decoding. Sample parity, analog mixing and host DMA/PIC behavior are documented limits. |
| `gameframe-ymz263` | In a port | YMZ263B port access, 128-byte PCM/ADPCM FIFOs, documented PCM rates/layouts, sampling timers, DMA request state, MIDI FIFOs and timestamped UART output. ADPCM-specific rounding, interleave and some status/threshold semantics need software comparison. Analog ADC/DAC filtering, game-port timing and external DMA/IRQ controllers are outside the chip model. |
| `gameframe-adlib-gold` | In a port | AdLib Gold/GSS port overlay for YMF262, YMZ263B and the documented Control Chip register map, active-low interrupt status, busy intervals, EEPROM commands, option identity, audio-block relocation and linear source mixer. TDA8425 tone/spatial transfer, analog filters, external DMA/IRQ delivery and revision-specific reset/EEPROM defaults remain open. |
| `gameframe-yac512` | In a port | YAC512 LSB-first serial input, FSEL decoding, falling-edge channel latches and held digital words. Floating DAC conversion and analog output remain open. |
| `wrappers/maxi94` | Unchanged 32-bit Windows game under Wine | Six `94DLL32.dll` exports used by POD; hooks the game's WinMM wave and mixer imports. A recorded race verified that the game discovers both streams and writes non-silent samples. |
| DOSBox-X harness module | Running and observing DOS games | Provides Sound Blaster, Gravis UltraSound, Miles and HMI SOS capabilities while the original runs. |
| 86Box | Running and observing DOS and Windows 3.x/9x games | Provides a configurable full PC, including period systems and supported ISA/PCI sound cards with per-card resource settings. Use it as a versioned software reference, not as physical-hardware evidence; see the [86Box observation guide](../observe/86box.md). |
| Wine/Windows audio APIs | Running Windows games | Supplies common WinMM and DirectSound paths. DSOAL is a separate DirectSound replacement that maps to OpenAL and supports EAX. |
| Munt `libmt32emu` | Running MIDI through an external synthesizer | Emulates MT-32, CM-32L and LAPC-I. The library is LGPL-2.1-or-later and requires compatible ROM images supplied separately. |

See [crates](../crates/README.md), [replacement libraries](replacement-libraries.md), and the
[Windows sound-card API survey](sound-card-apis.md) for their detailed limits.

## Build in layers

1. **Chip crates** turn timestamped register or MIDI events into samples. Keep chip state and
   integer timing independent of Windows, DOS and the audio device. A host resampler belongs at
   the output boundary.
2. **Card models** add the registers, memory, mixer, DMA and interrupt behavior that games
   probe. Reuse chip crates instead of copying their synthesis code.
3. **Compatibility libraries** implement the functions, COM interfaces or port behavior that
   one game actually calls. Export the original names and calling conventions, log unsupported
   calls, and measure the game's response to every answer.
4. **Audio backends** send PCM to WinMM, DirectSound, a host callback or a file. The backend
   must not decide which historical card the game believes it has.

The graphics wrappers provide a good pattern for DLL exports, stubs, logs and tests. Audio
needs a separate pattern for ordered writes, sample timestamps, IRQ/DMA callbacks, MIDI events
and streaming buffers. A successful stub can make a game disable sound; first trace how the game
uses each answer. The [Maxi Sound empty-map finding](../knowhow/sound-card-library-empty-answer-silences.md)
shows this failure in a measured game.

## Clean implementation and licensing

1. Identify the exact chip, card revision, driver and game-facing interface.
2. Gather public datasheets, manuals, errata and patents. Where an original card driver or
   firmware binary is available, inspect its imports, strings, tables and disassembly to recover
   the interface and behavior. Record addresses and evidence in a behavior specification.
3. Write the implementation from that specification. Do not read, copy, adapt or link another
   emulator's source code. Use another emulator only as a black-box comparison target when the
   original hardware or driver cannot answer a question. Label that evidence as emulated.
4. Build tests from generated inputs and measured outputs. Keep game binaries, game captures,
   card firmware and ROM or SoundFont data out of this repository.

Preserving compatibility for discontinued cards can keep old software usable. That purpose informs
project scope, but does not decide the effect of a license clause by itself. See the
[interoperability guidance](../LEGAL.md#interoperability) before choosing a method for a particular
driver package.

Phoenix Technologies' IBM-compatible BIOS work is a documented historical example of a clean-room
process: one group wrote functional specifications, and a separate group implemented from them.
This is an industry precedent, not a court ruling or a guarantee about a particular license. See
the [clean-room background](../LEGAL.md#clean-room-or-not). The Sound Blaster work is a clean
implementation, not a strict clean room: the same project team can study functional evidence and
write the implementation. A future strict clean-room effort would need separate people and a
written specification.

GameFrame-authored code is currently Apache-2.0. The owner can choose to relicense existing work,
including as `Apache-2.0 OR MIT`, or choose a different scope for new crates. Record the choice
for each affected crate in its Cargo metadata and README, and update the repository's license
files and summaries when the owner selects a project-wide change. Preserve the terms and notices
for any third-party material. This clean-implementation path adds no emulator source code.

## Priority for new reusable work

| Priority | Piece | Why it comes next | Main proof needed |
|---|---|---|---|
| 1 | Finish remaining AdLib Gold behavior | The card now applies documented linear source gains, channel selection, master volume, forced mono and mute. TDA8425 tone/spatial curves, some YMZ263 mode/status details, external DMA/IRQ delivery, analog input/output and four-operator carrier routing remain open. | Compare the original GSS Wave/MIDI driver against generated PCM, timer, UART and mixer cases in a software NE emulator. Resolve analog and uncertain chip behavior from a software black-box reference or reliable specification. Physical testing is not a release blocker. Keep Sound Blaster DSP/DMA findings in its own card model. |
| 2 | PC speaker, Tandy PSG variants and LPT DAC components | Covers early machines and low-cost/sample-DAC options that predate or avoid Sound Blaster. The TI core is in place; NCR 8496 and PSSJ require separate profiles. | Match documented clocks and register behavior, then compare generated tones and DAC streams with versioned software-emulator captures. Physical testing is optional. |
| 3 | Finish Sound Blaster behavior | The card model now includes OPL2/OPL3 FM ports, Pro 1 dual-OPL2 routing, Creative DSP command generations, firmware-derived ADPCM decoding, mixer registers and SB16 MPU UART. ADPCM sample parity, analog mixer effects, clone extensions and machine-level DMA/PIC integration remain. | Compare generated ADPCM vectors with the CT1741 firmware in a Ghidra 8051 emulator or an allowed software reference. Run generated DOS port traces and check transfer counts, IRQ acknowledgement, mixer masks and a fake DMA/PIC adapter. Physical testing is optional. |
| 4 | GF1 and InterWave voice model | Adds GUS-specific sample playback, memory and voice behavior used by trackers and some games. | Compare card RAM access, voice interpolation, panning, volume ramps, IRQs and driver initialization. Keep classic, MAX and PnP differences visible. |
| 5 | MPU-401 bridge and synth adapters | Enables MIDI routing to MT-32 and General MIDI engines without coupling the sequencer to one synthesizer. | Preserve event timing and SysEx uploads. Compare MIDI output against captures. Keep ROM and SoundFont files outside GameFrame unless their rights allow redistribution. |
| 6 | Windows-specific libraries on demand | Add A3D COM or an AWE SoundFont Manager only when a game's imports and call sites show a need. Reuse DirectSound/EAX implementations such as DSOAL when they already meet the game's path. | Identify the exact library version and interfaces, capture the game's calls, and test the fallback behavior as well as playback. |

## Existing projects for black-box comparison

These projects cover useful chips and libraries. Their docs identify possible comparison
targets. Their source is outside the clean-implementation path. GameFrame has not checked their
output against the original hardware or an accepted reference capture.

| Project | Reported coverage | Use in a clean-room comparison |
|---|---|---|
| [86Box](https://github.com/86Box/86Box) | Full PC emulation with supported period sound cards | Run a game's original driver against a recorded machine and card profile. Compare behavior as an emulated reference, not physical-hardware evidence; keep 86Box source outside a source-blind implementation task. |
| [oplon](https://docs.rs/oplon/latest/oplon/) | Rust `no_std` OPL2/OPL3 core | Its docs describe a clean-room core and say its log-sine and exponent ROMs and envelope increments match canonical values within ±1 LSB. Compare a built binary's output and timer behavior against hardware or a measured reference. |
| [nuked-opl3-rs](https://github.com/tgies/nuked-opl3-rs) | Rust OPL3 core based on Nuked-OPL3 | Its project reports bit-identical output to its C reference. Treat that as a project claim; compare binary output without using its code. |
| [ymfm](https://github.com/aaronsgiles/ymfm) | Yamaha FM cores, including YMF262 | Its project states an audibly indistinguishable accuracy goal. A compiled build can provide a comparison signal; it is not GameFrame source. |
| [Munt](https://github.com/munt/munt) | MT-32, CM-32L and LAPC-I | Use output from a built engine as an emulated comparison. Keep its required ROM images separate. |
| [moont](https://crates.io/crates/moont) | Rust CM-32L emulator based on Munt | Its docs describe comparisons against Munt. Compare rendered output as an emulated reference; it covers CM-32L only. |
| [DSOAL](https://github.com/kcat/dsoal) | DirectSound and EAX through OpenAL | Use its DLL as a Windows runtime option or black-box reference for the API path. |

For OPL3, compare behavior against the original chip or a named black-box reference before
writing a new core. Keep chip synthesis separate from card-level status, timers, interrupts and
mixer state. Do not treat OPL3 as OPL2 with extra registers.

Keep Munt and CM-32L ROM images separate from GameFrame's source and release. DSOAL already
provides the DirectSound and EAX path. A GameFrame wrapper should implement a missing API such
as A3D's COM contract.

## Per-game acceptance record

For every card or library added, keep a small table in the relevant crate or wrapper README:

- exact card, chip, driver and library versions;
- calls or port writes the game actually makes;
- initialization answers and what the game does with them;
- clocks, buffer formats, DMA/IRQ cadence or MIDI event times;
- source of the reference output and whether it is measured, emulated or inferred;
- implemented calls and modes, known differences and a sample-based test with no game files.

The reference configuration belongs in the game's project `AGENTS.md`. Keep game binaries,
captures and copyrighted ROM or SoundFont data outside GameFrame.

Licence: CC0-1.0.
