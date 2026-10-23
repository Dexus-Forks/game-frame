---
id: insert-cd-file-open-failed
title: The game asks for its CD while the disc is there, because some file failed to open
kind: failure
status: solved
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [cd-rom]
api: [win32]
symptom: [file-not-found, wont-start]
tags: [cd-check, language, volume-label]
signatures:
  - 'insert\s(?:[^\n]{0,40}\s)?(?:the\s)?cd(?:-rom)?\b'
  - 'ins[eé]r(?:ez|er)\s(?:[^\n]{0,40}\s)?(?:le\s)?cd(?:-rom)?\b'
tools: []
skills: [re-observe, re-trace]
docs: [docs/observe/system-calls.md, docs/knowhow/context-windows-9x-directdraw.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# The game asks for its CD while the disc is there, because some file failed to open

## Symptoms

The game shows "Please insert the CD-ROM", or the same message in another language. The disc's
files are mounted.

## Cause

Many games of the CD era wrap every file open in one routine. On any failure, that routine
asks for the CD. The open can fail for many reasons, such as a language folder the disc does
not have, a path built from a bad ini value or a wrong drive label (*observed* in POD: the disc
had no English text folder).

## Evidence

`WINEDEBUG=+file` lists every open with its result. The last failed open before the message is
the cause.

## What to do

Trace the file opens and fix the first one that fails. Check the drive too. Many games take the
first drive where `GetDriveTypeA` is 5 (CD-ROM) and the volume label matches. In Wine, map the
disc's folder as a CD-ROM drive with that label (`winecfg`, Drives, Advanced).
