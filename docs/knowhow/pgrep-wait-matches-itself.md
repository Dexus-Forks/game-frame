---
id: pgrep-wait-matches-itself
title: A wait loop with `pgrep -f` never ends after the process has finished
kind: failure
status: solved
symptom: [hang]
tags: [pgrep, scripts, background, agents]
commands: [pgrep]
tools: []
skills: [re-observe]
docs: [docs/working/agents.md]
seen: ['Tyrian 2000 (1999), headless capture runs, 2026-10-24']
---

# A wait loop with `pgrep -f` never ends after the process has finished

## Symptoms

An agent starts a long capture in the background, then waits for it with a loop such as
`until ! pgrep -f "capture.py out 5" >/dev/null; do sleep 10; done`. The capture finishes and
writes its output, but the loop keeps running for as long as anyone lets it.

## Cause

`pgrep -f` matches the pattern against every process's whole command line. The loop runs inside
a shell whose own command line contains the same pattern text, so `pgrep` always finds the
waiting shell itself (*observed*: three such loops were still running 40 minutes after their
capture ended, each matching its own `zsh -c` command).

## What to do

Wait on the process ID, not on a pattern. Start the job with `&`, keep `$!`, and wait with
`wait $!` in the same shell, or poll with `kill -0 $PID`. If only a pattern is available, write
it so it cannot match itself, for example `pgrep -f "[c]apture.py out 5"`: the bracket keeps
the pattern's own text from matching. Better still, have the job write a file when it is done
(its log's last line) and wait for that.

## Tried

- `until ! pgrep -f "<the job's command>"`: never ends, because it matches the waiting shell.
