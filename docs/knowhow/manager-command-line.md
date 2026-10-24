---
id: manager-command-line
title: 'The game wants the Manager''s command line as well as its registry key'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [command, line, manager]
tools: []
skills: [re-emulate, re-observe, re-format]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# The game wants the Manager's command line as well as its registry key

## What to know

*2026-10-24, Wine 10 in the emulated PC (`docs/ground-truth.md`).*

Symptom: with every value of `HKCU\Software\Refinery\Hardman\Config` written (a `+reg` trace
shows `config_read` reading all of them, up to `PhilsEnding`), `HARDWARW.EXE` still stops with
"Please run Hardwar from the Hardwar Manager only."

Cause: the message has one user, at `0x4105c5` in the command-line parser `FUN_004103b4`
(*observed*). The parser walks the arguments. When none of them is a `/t:` option, it shows the
message and quits. The Manager starts the game with one of `/t:new FILE`, `/t:load FILE`,
`/t:connect`, `/t:newserver FILE`, `/t:loadserver FILE` or `/t:video NAME` (the keywords are
the strings at `0x53021c`-`0x530348`); `/t:lobby` is refused ("Lobby launching is not
supported in this version"). `config_read` (`0x412928`) fails silently instead: it returns
early and leaves `DAT_00580234` at 0, and `win_main` then quits without a word.

Fix: start the game as the Manager does, `HARDWARW.EXE /t:new C:\HARDWAR\SAVES\NAME.SAV` for
a new world named NAME, or `/t:load C:\HARDWAR\SAVES\NAME.SAV` to load one
(Hardwar's `vm_run.sh`).

For GameFrame: a launcher-only game can check its command line as well as its configuration.
Search the "run from the launcher" string's users before writing more registry values.
