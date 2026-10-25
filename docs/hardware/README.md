# Hardware of the era

![A circuit board with a CPU, a video chip, a sound chip and memory](../assets/diagrams/banner-hardware.svg)

![A timeline of PC graphics, sound and CPU features, 1981 to 1999](../assets/diagrams/pc-timeline.svg)

A PC game from 1985 to 2005 uses its hardware far more directly than a modern program. DOS
games program the timer, the video card, the sound card and the joystick port themselves.
Windows games of the late 1990s use DirectX or a card maker's own 3D API. The hardware a game was
written for affects much of its code: the CPU's floating-point unit, the video
mode, the sound chip, the 3D card's features.

When you rebuild such a game, you replace the hardware with modern equivalents. To do that
exactly, you need to know what each piece of hardware did, how the game used it, and which of
its details the player can see or hear.

## Ask your agent

Find the hardware layer. Copy this prompt into your coding agent:

```text
In my GameFrame project, find every place the game touches hardware.

1. Read docs/hardware/README.md and the pages for this game's devices.
2. In the disassembly, find IN, OUT and INT instructions (tools/triage/insns.py lists them)
   and writes to A000, B800 and 0040.
   For a Windows game, list the imports from DirectX, Glide, OpenGL and WinMM.
3. Name the driver functions (set a mode, set a palette, play a sound, read the joystick).
4. List the values the game reads back from the hardware. They are inputs.
5. Propose one reference configuration (video, sound, input) and ask me to confirm it.
```

## Pages

- [CPUs and floating point](cpus.md): the x86 generations, the x87 FPU, MMX, 3DNow! and SSE,
  fixed point, timing loops, self-modifying code.
- [2D graphics](graphics-2d.md): CGA, EGA, VGA, Mode 13h, Mode X, VESA, palettes and the
  vertical retrace.
- [3D accelerators](graphics-3d.md): 3dfx and Glide, Rendition, PowerVR, S3, ATI, Matrox,
  NVIDIA, Direct3D and OpenGL, and software renderers.
- [The card makers' 3D APIs](card-apis.md): Glide 2.x, ATI3DCIF and the S3D Toolkit in detail:
  reading their calls, how textures reach each card, and games with one build per card.
- [Replacement libraries for a 1990s game](replacement-libraries.md): probes, exact exports with
  `zcc.sh`, what the game assumes (a window, threads, the frame buffer, port I/O, any display
  mode), and measuring what the card is given.
- [Replacement 3D-card libraries](wrappers.md): GameFrame's stand-in DLLs for Glide 2, PowerVR
  SGL, ATI3DCIF, the S3D Toolkit and DirectDraw. The page says what each draws, how to build
  and use them, and how to add a game.
- [MMX and other SIMD code](simd.md): finding hand-written MMX routines, matching them to the
  plain build, measuring both, and porting them exactly.
- [Sound](sound.md): the PC speaker, Tandy, Covox, Game Blaster, AdLib and OPL FM synthesis,
  Sound Blaster, Gravis Ultrasound, MIDI devices, DirectSound, A3D and EAX.
- [PC audio coverage](audio-coverage.md): the reusable chip, card, library and backend layers;
  GameFrame coverage today and priorities for the next hardware models and replacements.
- [Sound Blaster DSP, mixer and MPU-401](sound-blaster-dsp.md): the new Creative card model,
  its programming reference, tested interface paths, remaining limits and software-only checks.
- [Creative SBBASIC driver package](sbbasic-driver-interface.md): package provenance and the
  installed DOS 6.22 and Windows 3.11 setup.
- [Win16 SB16 auxiliary volume contract](sbbasic-driver-behavior.md): a bounded query and update
  slice, with unknown behavior and software-only verification suggestions.
- [Win16 SB16 wave-output contract](sb16-wave-output-contract.md): a bounded logical contract for
  format opening and buffer submission. The accepted PCM matrix, callback behavior, pause and
  restart semantics, reset behavior and hardware effects remain unknown.
- [Win16 SB16 wave-completion contract](sb16-wave-completion-contract.md): a bounded logical
  contract for queue draining, header updates, callback delivery, and pause, restart and reset.
  Callback timing and order, reset handling for queued headers, buffer limits, and lower-service
  behavior remain unknown.
- [Win16 SB16 wave-input contract](sb16-wave-input-contract.md): a bounded logical contract for
  input open, capture queues and completion callbacks. PCM limits, callback timing, overrun,
  stream concurrency and control-message effects remain unknown.
- [Win16 SB16 MIDI-port output contract](sb16-midi-output-contract.md): a bounded logical
  contract for port open and short-message sends. Port routing, system-message edge cases,
  lower-service protocol, and output timing remain unknown.
- [Win16 SB16 MIDI short-message contract](sb16-midi-short-message-contract.md): refined
  status-byte handling and a one-level reentrancy guard. Packed argument layout and the
  lower-service event format remain unknown.
- [Win16 SB16 mixer volume-details contract](sb16-mixer-volume-contract.md): a bounded view of
  the descriptor-driven control-details read and write paths. The master selector, channel
  shape and numeric scale remain unknown.
- [Win16 SB16 FM synth note contract](sb16-fm-note-contract.md): a bounded trace of the Win16
  FM synth wrapper's open and short-message paths. The lower-service event format and
  note-to-OPL conversion remain unknown.
- [The sound card makers' APIs](sound-card-apis.md): the Maxi Sound 64, A3D, EAX, the AWE's
  SoundFont Manager, Sensaura and QSound: how a game finds each, what Wine offers, and which
  can be stood in for.
- [Shaders](shaders.md): finding compiled and source shaders with `shaders.py`, reading them,
  capturing frames, and porting them exactly.
- [Pixel-exact software rendering](software-rendering.md): fill rules, edge stepping, lines,
  texture mapping, lookup tables and drawing order, and how to measure each in the original.
- [Working with the real hardware](real-hardware.md): dumping your own media and firmware,
  capturing video and sound, logic analysers, and working safely.
- [Memory, storage, input and networks](other.md): EMS, XMS and DOS extenders, CD audio, the
  game port, IPX, serial links and modems.

## General rules for hardware code

1. **Find every place the game touches hardware.** In DOS games, search the disassembly for
   `IN`, `OUT`, `INT` and writes to segments `A000`, `B800` and `0040`
   (`tools/triage/insns.py BINARY '^(in|out) ' '^int '` lists them). In Windows games, list
   the imports from DirectX, Glide, OpenGL, WinMM and the card makers' DLLs.
2. **Label the driver layer.** Most games wrap hardware access in a small set of functions
   (set a mode, set a palette, play a sample, read the joystick). Name them, and treat the rest
   of the game as their callers.
3. **Decide what the port must copy.** It must copy the picture and the sound the player
   perceives. It usually does not need to copy how the game produced them on the card. A frame drawn by Glide can be drawn
   by Vulkan, as long as it looks the same.
4. **Keep what the game logic reads back.** Some games read values from the hardware and use
   them in their rules: a timer count, the retrace state, a joystick reading, a sound
   position. Those values are inputs ([Inputs](../observe/inputs.md)).
5. **Choose one reference configuration.** The original looked and sounded different on
   different cards. Pick the configuration your project reproduces (for example "VGA, Sound
   Blaster 16, General MIDI through a Roland SC-55") and write it in the project's `AGENTS.md`.
