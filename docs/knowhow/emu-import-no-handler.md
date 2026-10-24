---
id: emu-import-no-handler
title: An emulated function calls a Windows import that has no handler
kind: failure
status: solved
runtime: [unicorn]
api: [win32]
symptom: [tool-error]
tags: [pe_emu, pe32_emu, imports, stubs]
signatures:
  - 'which has no handler \(pass stubs='
tools: [pe32_emu, pe_emu]
skills: [re-emulate]
docs: [docs/ABI.md]
seen: ['GameFrame sample game, 2026-10-23']
---

# An emulated function calls a Windows import that has no handler

## Symptoms

`pe32_emu` stops with "the routine called KERNEL32.dll!GetTickCount, which has no handler
(pass stubs={...})".

## Cause

The emulator points every import at a stub that stops the run with the import's name, so a
routine never runs off into Windows code that is not there. The routine called one.

## What to do

Give the import a handler that returns what the measurement needs: `Emu32('GAME.EXE',
stubs={'GetTickCount': lambda emu: 1000})`. A handler reads the call's arguments with
`emu.arg(i)`. For a library routine whose effect you only need to see (`sprintf`, `printf`),
stop at the call with `until=` and read its arguments instead. Record each stub's fixed answer
next to the measurement, since it is an input.
