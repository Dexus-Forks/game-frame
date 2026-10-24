---
id: context-keep-every-dialog
title: 'Keep a capture of every error dialog and crash, as it happens'
kind: context
status: context
era: []
platform: []
runtime: [wine, virtual-machine, dosbox]
hardware: [display]
symptom: [crash, wont-start]
tags: [evidence, screenshots, diary]
tools: [shot]
skills: [re-observe, re-land]
docs: [docs/working/evidence.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# Keep a capture of every error dialog and crash, as it happens

## What to know

The dialogs a game shows on the way to running are evidence: the exact text, the address,
the state of the screen behind it. They are also the story of the work. They are easy to lose.
A scratch capture is overwritten by the next run, and once the cause is fixed the dialog never
comes back. In POD's wrapper work the Cyrix "privileged instruction" fault and both S3D
Toolkit errors were seen, fixed and lost. The diary could only describe them, and one had to be
re-created afterwards by undoing a fix (*observed*, 2026-10-24).

- **Capture the moment a dialog appears**, before clicking it away, and copy the capture out of
  the scratch area at once: `derived/captures/YYYY-MM-DD/` (game material stays out of git),
  named for what it shows (`s3-setstate-error.png`, not `a1.png`).
- **Keep the text as well as the picture**: the message, the address, the log lines around it.
  Paste them into the know-how entry's Symptoms, so the entry's signature can match them.
- **Never reuse a capture name** across runs. A fixed name (`/tmp/a1.png`) loses every earlier
  stage. `./gameframe run shot NAME --text "THE MESSAGE"` keeps the screen (or `--from FILE`)
  in `derived/captures/DATE/`, with the text beside it, and never overwrites.
- **At the end of the session**, the diary shows each stage: the failure, the fix, the result.
  The posts draw on the kept captures, and reference pages can link them. A failure re-created
  later says so in its caption.
