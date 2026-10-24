# Recording the original, and replaying it

A port that must behave like the original needs reference runs. A reference run holds the state
of every actor over a whole level, a race or a match, with the game's own results to check
against. This page covers how to record such a run from the running game's memory and how to
read the recording. It also covers how to replay the recording into the port, so that the test
checks every sample from the start to the end. The method comes from the POD: Planet of Death
project. There a recorded race checked the port's rules against every word of every race record
([case study](../case-studies/pod.md)).

## Record from outside

The simplest recorder reads the game's memory from another process. It injects nothing and
changes nothing in the game. The game runs at its own speed. On Linux, including Wine on Linux
and Wine inside a VM, `/proc/PID/mem` gives root that access.

```bash
.venv/bin/python tools/live/memrec.py snap --name GAME.EXE --out derived/snap/1   # before
.venv/bin/python tools/live/memrec.py snap --name GAME.EXE --out derived/snap/2   # after a change
.venv/bin/python tools/triage/memscan.py derived/snap/1/0x500000.bin:any derived/snap/2/0x500000.bin:changed --base 0x500000
.venv/bin/python tools/live/memrec.py record spec.txt --name GAME.EXE --rate 10 \
  --wait 0x5920a4=2 --out derived/race/run-1.jsonl
```

1. **Find the state first**, by tracing and by memory diffs ([Finding values in
   memory](memory.md)). A non-relocated EXE keeps its globals at the same addresses in every
   run. A spec file written once then serves every recording.
2. **Write a spec** with one value a line, in the form `NAME ADDRESS TYPE [COUNT [STRIDE]]`. An
   array with a stride reads one field of every actor: `car_x 0x61aad0 i32 8 0x558`.
3. **Keep raw words.** Store fixed point as integers. A recording rounded on the way can never
   be compared bit for bit.
4. **Add the game's own clocks**: its time counter and the sum of its frame times. They date
   each sample in game time. A port can use game time where it cannot use wall time.
5. **Start at a phase** instead of driving the menus. `--wait ADDR=VALUE` starts the recording
   when a phase variable takes a value (a countdown, a level number). It also finds the process
   again if the game restarts ([wait for the next race](../knowhow/wait-for-the-next-race.md)).
6. **Read the final state** a few seconds after the run ends. Games often finish their
   bookkeeping after the main loop.
7. **Check against the screen.** Take a capture at each event and at the end
   (`tools/live/shot.py`). Print the recorded values the way the game prints them, and compare
   each digit. A mismatch of one unit often comes from a rounding helper. Check the helper
   before you doubt the address ([x87 time format](../knowhow/x87-truncating-time-format.md)).

Before a reference recording, make sure the game runs as shipped. Check that the install folder
holds no mod (`tools/live/modfolder.py list`). Check that nobody else drives a shared emulator
(`tools/live/emulock.py`) ([mods in a reference recording](../knowhow/mods-loaded-in-a-reference-recording.md)).

## Read the recording

```bash
.venv/bin/python tools/live/memlog.py summary derived/race/run-1.jsonl
.venv/bin/python tools/live/memlog.py csv derived/race/run-1.jsonl --fields clock,car_x --fixed car_x
.venv/bin/python tools/live/loopcut.py derived/race/run-1.jsonl --x car_x --y car_y --index 1
```

The summary shows the rate, each value's range and how often each value changed. A value that
never changes comes from a wrong address. A value that changes every sample at the wrong rate is
a different field. `loopcut.py` cuts a recorded path at its first return to the start. Use it for
a racing line or a patrol that must be a closed loop
([closed lap](../knowhow/racing-line-must-be-a-closed-lap.md)).

## What a recording from outside cannot see

- **A sample is not atomic.** The recorder reads the values one after another while the game
  runs. One sample can then straddle frames. The clock may be a frame older or newer than the
  positions.
- **Most frames are missed.** At 10 samples a second, a game at 60 frames a second shows one
  frame in six. Per-frame inputs (each frame's time, the keys) need a hook inside the game loop.
- **A capture taken in the sampling loop stops sampling** while it runs.
- **The game's clock is not wall time.** A game converts its timer into its own unit, often
  with a rounded factor. POD's race clock runs 0.82 % slow. Measure the ratio over a long run
  ([game clock](../knowhow/game-clock-not-real-time.md)).
- **The FPU's control word is part of the state.** A game may change it after its start-up,
  POD races with 0x027f (53-bit), not the 0x037f its C library leaves. Record it in every
  in-game recording or step log, and measure with that word
  ([the control word](../knowhow/fpu-control-word-thread.md)).

## Replay into the port

A port that takes the actors' motion as its input can be checked against a recording before
the motion itself is ported. POD ported its race rules first and fed them the recorded
positions. It then checked laps, places, times and the results screen
([rules before physics](../knowhow/context-port-rules-before-physics.md)).

- **Cut the core where the original cuts.** If the original moves an actor and then tests the
  move against the rules, the core's step takes the move as input. The same entry point serves
  the motion's port later.
- **Rebuild frames from the recording.** Place frames at the game clocks the recording holds.
  Cut long moves into short pieces, so that the port skips no crossing.
- **Compare at every sample, with a window.** Keep every state the port passes through. Accept
  a sample when one of those states equals it. The window runs from the sample's clock to the
  end of the frame that runs at the next sample's clock. When the sample read some values as one
  block, compare them as one value
  ([samples that straddle frames](../knowhow/replay-sample-straddles-frames.md)).
- **Report agreement, and require it.** Where the input cannot be exact, report the share of
  samples that agree, field by field. Let the test require that share, so that a regression
  shows.
- **Stand-ins follow the original's rules.** When the replay must supply state from code not
  yet ported, build the stand-in from the original's own rules. Fill in from the recording only
  after that. A stand-in that picks freely can make the port agree less as more of it is ported
  ([stand-ins](../knowhow/replay-stand-in-for-unported-logic.md)).
- **Check the test can fail.** Change one unit of one recorded value and see the replay fail.

## Measure the arithmetic separately

A replay tests how the parts combine. Measure the parts themselves first, under the emulator,
on many inputs, against the original's own code ([re-emulate](../../skills/re-emulate/SKILL.md)).
For 32-bit games, `pe32_emu` lays out the game's structures (`array`, `put`, `get`). It loads
the FPU's control word (`fpu_cw=`). It lists the inputs whose result depends on the rounding
mode (`fpu_modes`). It can stop at a call to a library routine to read its arguments (`until=`)
([functions on game structures](../knowhow/emulate-function-on-game-structures.md),
[x87 rounding](../knowhow/x87-magic-add-rounding.md)).

A test of one step from a fresh state cannot show that state carries over from step to step.
Measure sequences too: call the routine N times without `reset()` between calls, so the
emulator's memory keeps what each step wrote, write each step's outputs, and let the port's
test keep its own state the same way. Compare the two sequences step by step.
