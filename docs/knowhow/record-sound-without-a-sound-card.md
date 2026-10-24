---
id: record-sound-without-a-sound-card
title: Record a Windows game's sound in a VM that has no sound card
kind: context
status: context
era: [late-1990s]
platform: [windows-9x, linux]
runtime: [wine, virtual-machine]
compiler: []
hardware: []
genre: []
symptom: []
tags: [sound, waveout, wine, pulseaudio, null-sink, parec, qemu, recording, compare]
tools: []
skills: [re-observe, re-verify]
docs: [docs/hardware/sound.md, docs/observe/running-the-original.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Record a Windows game's sound in a VM that has no sound card

## What to know

A game under Wine in a VM can play into a sound server instead of a device, so QEMU needs no
sound card (adding one means a restart, and kernel drivers that a cloud kernel may lack).
Inside the guest, start PulseAudio with only a null sink at the game's own format and record
its monitor:

```sh
pulseaudio -D -n --exit-idle-time=-1 --load=module-native-protocol-unix \
  --load="module-null-sink sink_name=rec rate=22050 format=s16le channels=2" --load=module-always-sink
parec -d rec.monitor --rate=22050 --format=s16le --channels=2 --raw out.raw
```

The 32-bit game needs the i386 ALSA and Pulse libraries (`libasound2-plugins:i386`; Wine's
`libpulse0:i386` is usually there). Check with `pactl list sink-inputs` that the game's stream
has the sink's rate, so nothing is resampled. In POD the game then starts without its "no
sound driver" question.

What the recording is good for: levels, pans, which sound plays when. In POD the menu sound
came out with a gain of 1.00006 against the port and 94% of samples within ±1. What it is not
good for: sample-for-sample checks. Wine's audio path adds ±1 noise (*inferred*: it converts
to floating point and back), and an emulated PC (QEMU's TCG) misses deadlines, leaving gaps
of silence in the recording. Line the recording up with the port window by window
(`measure/sound_recorded.py`), not once for the whole sound, and keep the emulator's
measurement of the mixer as the exact evidence.

Two traps on a VM shared by several sessions: `pgrep -x Xvfb` finds another session's display
and a start script then skips starting its own; and `pkill -f "Xvfb :1"` run over SSH matches
the SSH command itself. Match the process name with `pgrep -a -x` and filter on the display.
