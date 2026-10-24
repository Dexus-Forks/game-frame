# maxi94: the Maxi Sound 64 library on Wine's wave outputs

`94DLL32.dll` replaces the library of Guillemot's Maxi Sound 64 card ("Dream thunk for 9407
products", Dream S.A. 1996). The original is a 32-bit thunk down to `94DLL16.DLL` and the
driver of the card's Dream SAM9407 chip. Wine has neither, and the card is long gone. This
stand-in answers the game's calls as the card would, and plays the game's sound through Wine's
wave outputs. It was made for POD Gold's two "64" builds, `PODMAX64.EXE` (software) and
`PODFXM64.EXE` (3dfx). They import six calls by ordinal (`maxi94.def`).

## What POD sends the card

Read in `PODMAX64.EXE` (its source file is `srcson\maxi64\sndplay.cxx`):

- **No MIDI and no sample banks.** The game mixes all its sound itself into two stereo wave
  outputs, 22,050 Hz and 16 bits, opened on the wave mapper (`FUN_004ba304`). It does not
  import the card's `DownLoad`, `bank*` or `mod*` calls. The music is CD audio.
- **The card's memory map.** On the card each wave output holds one of the SAM9407's wave
  streams. The game reads the map (`GetMemMapAddress`, then `UpLoad` of `0xc0` words) before
  and after it opens each output. The stream that appeared is that output's. When it finds
  none it turns its sound off. An empty map is therefore silent.
- **Each stream's volumes.** `waveSetMainVolume` and `waveSetAuxVolume` take a stream, left
  and right (0 to `0xffff`). With the game's setting at `0x4fdf70` on, one stream gets main
  only and the other aux only. With it off, both get both (`FUN_004bbe0c`). Main is the front
  outputs and aux the rear (*inferred*).
- **The mixer.** The effect and music volumes are the mixer's WAVEOUT and COMPACTDISC source
  lines (`FUN_004b9710`). The game reads the wave line back to set the streams.
- **One card command.** `mpuMessageAc(0x60)` with one byte, `0x7f` when the sound starts and
  `0` when it stops. What it does on the chip is *unknown*.

## What the stand-in does

| Call | Answer |
|---|---|
| `GetMemMapAddress`, `UpLoad` | The map lists one wave stream for each wave output the game has open, stream `n` at entry `n` |
| `waveSetMainVolume`, `waveSetAuxVolume` | Kept per stream; the stream's wave output plays at the louder of the two (`waveOutSetVolume`) |
| `mpuMessage`, `mpuMessageAc` | Acknowledged and logged |

To know which outputs are open, it hooks the game's `waveOutOpen`, `waveOutClose` and
`waveOutWrite` in its import table (`shim::iat`). It also hooks the game's mixer calls and
answers for the WAVEOUT and COMPACTDISC lines itself. Wine's mixer has no CD line, and its wave
line's volume reads 0 (*measured*, Wine 11 on macOS). The game would take 0 as its effect
volume.

Differences from the card (`Deviation:` in the code):

- Two speakers only. A stream set to the rear plays on the front pair too.
- The CD line's volume is kept and read back. It does not change the CD's music.

## Run

Copy `maxi94.dll` next to the game as `94DLL32.dll` and tell Wine to use it:
`WINEDLLOVERRIDES="94dll32=n"`. POD's 64 builds also import `DDRAW.dll` (use `ddraw.dll`) and,
for `PODFXM64.EXE`, `glide2x.dll`.

| Variable | Meaning | Default |
|---|---|---|
| `MAXI94_LOG` | A file to log to (`C:\maxi94.log`) | standard error |

The log names each stream as it opens, every volume the game sets, and the first buffer of
each stream that is not silence:

```text
maxi94: waveOutOpen 0xff00: stream 0
maxi94: waveOutOpen 0xff01: stream 1
maxi94: stream 0 main 0xffff,0xffff: wave output 0xff00 at 0xffffffff (0)
maxi94: stream 1 aux 0xffff,0xffff: wave output 0xff01 at 0xffffffff (0)
```

## Not handled

The card's other 31 calls are not exported: sample banks and SoundFont-style instruments
(`bank*`, `DownLoad`, `mem*`), the chip's own voices (`mod*`), the effects (`WideEffect`,
`Equalizer`, `surroundSetVolume`) and the firmware. A game that imports them fails to load;
add them with what that game does with them.

Licence: Apache-2.0 (code), CC0-1.0 (this text).
