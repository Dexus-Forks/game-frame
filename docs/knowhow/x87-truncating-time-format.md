---
id: x87-truncating-time-format
title: Times printed by the game differ by a hundredth from the same values printed by a tool
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
compiler: [watcom]
hardware: [x86-cpu]
genre: [racing-sports]
symptom: [tool-error]
tags: [x87, fpu, rounding-mode, frndint, fldcw, control-word, time-format, fixed-point]
tools: [pe32_emu, memrec]
skills: [re-trace, re-port, re-verify, re-emulate]
docs: [docs/COMPILED-CODE.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# Times printed by the game differ by a hundredth from the same values printed by a tool

## Symptoms

A time read from memory and printed by our reader is a hundredth away from the game's
screen. In POD the gap behind the winner read +23"21 in our reader and +23"20 on the
results screen; the other times matched.

## Cause

POD's `format_time` (`FUN_0047e4d0`) converts a 16.16 time in x87 floating point: minutes =
`v × 2.5431e-7`, seconds = `rem / 65536`, each through `FUN_004d18ac`, then hundredths =
`frac × 0.0015259 + 0.5`, with a carry into the seconds at 100 and into the minutes at 60
(0x47e543, 0x47e560). `FUN_004d18ac` is not a bare `frndint`: it saves the control word,
loads one with the rounding set to chop (`mov byte [esp+1], 0x1f; fldcw`), runs `frndint`
and puts the old word back (*observed*). So these conversions truncate whatever mode the game
runs in. The game itself runs with round to nearest (*measured*: its gate test, an `fadd`
of 1.5 × 2^68 that does depend on the mode, matches under the control word 0x037f on 20,000
cases and not under truncation; [x87-magic-add-rounding](x87-magic-add-rounding.md)).

An earlier version of this entry inferred from the results screens that the whole game ran
truncated. The screens fit, because the rounding happens inside the helper; the inference
about the mode did not.

## Evidence

The three instructions of `FUN_004d18ac` (0x4d18ae..0x4d18bd); `format_time` run under
Unicorn on 20,000 values, stopped at its `sprintf` call to read the minutes, seconds and
hundredths (`measure/race_rules.py`).

## What to do

Print times the game's way: the whole of `format_time` is exact in integers (`minutes =
v / 3,932,160`, truncated; seconds `rem / 65536`, truncated; hundredths `(frac × 100 + 32768)
/ 65536`, truncated; then both carries). `crates/pod-race` `time::parts` and
`run/vm/racelog.py` `clock()` do this. Compute a gap as the difference of the raw values,
then format it. When a game's float conversions disagree with a tool by one unit, read the
rounding helper's own instructions before blaming the FPU's mode, and measure any
mode-dependent code under both control words.
