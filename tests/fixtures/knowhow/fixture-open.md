---
id: fixture-open
title: The test machine freezes after a while
kind: failure
status: open
runtime: [testvm]
symptom: [hang]
signatures:
  - 'gadget fault in fixturemod\+0x10[0-9a-f]'
commands:
  - '\bfixture-run\b'
seen: ['the sample game, 2026-10-23']
---

# The test machine freezes after a while

## Symptoms

It stops.

## Tried

- A bigger stack: no change.
- Fewer threads: no change.
