# Media sample

Tiny files in the standard formats many games ship, for the tests of `tools/decoders/`. They
are our own data, built byte by byte from the formats' published layouts by
`make_media.py`, not by the decoders they test.

```bash
python3 examples/media-sample/make_media.py examples/media-sample/build
.venv/bin/python tools/formats/validate.py tools/decoders/bmp.py 'examples/media-sample/build/*.bmp' --show 1
```

- Nine BMPs of the same 4 x 3 picture: 8 bits bottom-up (`pal8`), 24 bits top-down (`top24`),
  RLE8 with deltas and an absolute run (`rle8`), RLE4 (`rle4`), 1 bit (`mono`, its own
  picture), 4 bits with an OS/2 core header and a short palette (`core4`), 16 bits with 5-6-5
  bit fields (`rgb565`), 32 bits with a V5 header (`v5_32`), and `pal8` cut after its first
  stored row (`short`).
- `pcm16.wav`: 8 frames of 16-bit stereo PCM; `short.wav`: the same with a data size field that
  claims twice the bytes that are there.
- `song.mid`: a format 1 file with a tempo track and a three-note melody that uses running
  status and note-ons of velocity 0; `song.rmi`: the same song in a RIFF RMID.
- `film.avi`: a video and an audio stream, their formats, and two frames and one audio block.

`tests/run.sh` ("Decoders for common formats") checks that every picture decodes to the
generator's picture pixel for pixel, the values of the sound, songs and film, that a BMP with
JPEG compression and a MIDI file with a wrong track count are refused, and that the catalogue
decodes all fourteen files.
