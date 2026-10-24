# PC audio coverage

GameFrame's audio work has four layers. A chip model produces samples. A card model adds the
registers, memory and timing the game probes. A library replacement lets an unchanged game
reach the card through the interface it already calls. The host audio backend plays the
finished samples. Keeping these layers separate makes one chip useful to AdLib, Sound Blaster
and later compatible cards.

This page covers common IBM PC audio from the PC speaker through the 1990s Windows cards. It
tracks what GameFrame has, what an existing runtime can provide while observing a game, and the
next reusable pieces to build. DOSBox-X's [sound card guide](https://dosbox-x.com/wiki/Guide%3ASound-card-support-in-DOSBox%E2%80%90X)
has a broader device table and configuration details.

## Device families and GameFrame coverage

| Era | Device family | What the game talks to | GameFrame coverage |
|---|---|---|---|
| 1981 onward | PC speaker | PIT channel 2 and the speaker gate; some games use rapid toggles for samples | DOSBox-X can run and record these games. No reusable PC speaker and PIT audio crate yet. |
| 1984 onward | IBM PCjr and Tandy 1000 | SN76496/SN76489 tone and noise registers; later Tandy models add an 8-bit DAC | Covered by DOSBox-X machine emulation while observing the original. No GameFrame device crate yet. |
| 1987 onward | Covox Speech Thing and Disney Sound Source | Writes to an LPT port drive an 8-bit DAC; the Disney device also has a small FIFO | DOSBox-X can provide LPT DAC emulation. No GameFrame DAC or FIFO model yet. |
| 1987 onward | Creative Music System / Game Blaster | Two SAA1099 chips provide square-wave voices, noise and stereo output | No GameFrame chip or card model yet. |
| 1987 onward | AdLib and Sound Blaster 1.x/2.0 | YM3812 OPL2 register writes for music; Sound Blaster adds DSP commands, DMA and IRQ-driven PCM | `gameframe-opl2` models the YM3812 and has sample-for-sample evidence from Frontier: Elite II. No reusable Sound Blaster DSP, DMA or IRQ model yet. |
| 1991 onward | Sound Blaster Pro, Pro 2 and Sound Blaster 16; AdLib Gold | Stereo PCM and mixers; early Pro uses two OPL2 chips, later models use YMF262 OPL3; SB16 adds 16-bit DMA | OPL2 is available. OPL3, card-level DSP/DMA behavior and mixer models are open. |
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
| `wrappers/maxi94` | Unchanged 32-bit Windows game under Wine | Six `94DLL32.dll` exports used by POD; hooks the game's WinMM wave and mixer imports. A recorded race verified that the game discovers both streams and writes non-silent samples. |
| DOSBox-X harness module | Running and observing DOS games | Provides Sound Blaster, Gravis UltraSound, Miles and HMI SOS capabilities while the original runs. |
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

## Priority for new reusable work

| Priority | Piece | Why it comes next | Main proof needed |
|---|---|---|---|
| 1 | YMF262 OPL3 chip crate or adapter | Extends the existing OPL2 path to AdLib Gold, Sound Blaster Pro 2 and SB16-era FM. Keep OPL2 behavior and OPL3 stereo, four-operator channels and extra waveforms distinct. | Compare register reads, timers and output against a named reference implementation or real card. Include four-operator, rhythm, stereo, reset and unsupported-mode cases. |
| 2 | PC speaker, Tandy PSG, CMS/SAA1099 and LPT DAC components | Covers early machines and low-cost/sample-DAC options that predate or avoid Sound Blaster. These can be separate small crates or modules when their state and clocks differ. | Match documented clocks and register behavior, then compare generated tones and DAC streams with emulator or hardware captures. |
| 3 | Sound Blaster DSP and DMA/IRQ model | The broadest DOS PCM interface. Model DSP generation differences instead of treating every Sound Blaster as one device. Use OPL2/OPL3 components for FM. | Trace reset/version handshakes, command completion, auto-init and single-cycle DMA, sample rate, IRQ timing, stereo and 8/16-bit paths. |
| 4 | GF1 and InterWave voice model | Adds GUS-specific sample playback, memory and voice behavior used by trackers and some games. | Compare card RAM access, voice interpolation, panning, volume ramps, IRQs and driver initialization. Keep classic, MAX and PnP differences visible. |
| 5 | MPU-401 bridge and synth adapters | Enables MIDI routing to MT-32 and General MIDI engines without coupling the sequencer to one synthesizer. | Preserve event timing and SysEx uploads. Compare MIDI output against captures. Keep ROM and SoundFont files outside GameFrame unless their rights allow redistribution. |
| 6 | Windows-specific libraries on demand | Add A3D COM or an AWE SoundFont Manager only when a game's imports and call sites show a need. Reuse DirectSound/EAX implementations such as DSOAL when they already meet the game's path. | Identify the exact library version and interfaces, capture the game's calls, and test the fallback behavior as well as playback. |

## External engines to evaluate

These projects cover useful chips and libraries. GameFrame has not checked their output against
the original hardware or an accepted reference capture.

| Project | Coverage | Fit and checks before adoption |
|---|---|---|
| [oplon](https://docs.rs/oplon/latest/oplon/) | Rust `no_std` OPL2/OPL3 core; MIT | Its Rust API may fit the crate. Review its source for the no-`unsafe` requirement. Its documentation says its log-sine and exponent ROMs and envelope increments match canonical values within ±1 LSB. Compare chip output and timers against a trusted YMF262 reference and GameFrame's OPL2 behavior. |
| [nuked-opl3-rs](https://github.com/tgies/nuked-opl3-rs) | Pure Rust port of Nuked-OPL3; LGPL-2.1-or-later | Its README reports bit-identical output to the C implementation and includes optional C reference tests. Check unsafe use, build requirements, timer coverage and how the LGPL dependency is delivered. |
| [ymfm](https://github.com/aaronsgiles/ymfm) | Yamaha FM cores, including YMF262; BSD-3-Clause | C++14 requires a wrapper and a build decision. Its stated accuracy target is audibly indistinguishable rather than cycle- or bit-exact. |
| [Munt](https://github.com/munt/munt) | MT-32, CM-32L and LAPC-I; LGPL-2.1-or-later | C++ library with a C API. It requires compatible ROM images supplied separately. |
| [moont](https://crates.io/crates/moont) | Rust CM-32L emulator based on Munt; LGPL-2.1-or-later | `no_std` crate with runtime ROM loading. Its README describes comparison tests against Munt; GameFrame has not run those tests. It models CM-32L and has limited model coverage. |
| [DSOAL](https://github.com/kcat/dsoal) | DirectSound and EAX through OpenAL; LGPL-2.1 | Existing Windows DLL replacement. Check whether it covers the game's DirectSound path before adding a new wrapper. |

For OPL3, evaluate the pure Rust candidates against a named reference before writing a new core.
Each candidate handles chip synthesis differently from card-level status, timers, interrupts
and mixer state. Keep those layers separate. Do not treat OPL3 as OPL2 with extra registers.

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
