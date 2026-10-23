---
id: fixture-solved
title: The sample tool stops with a widget overflow
kind: failure
status: solved
runtime: [testvm]
symptom: [crash]
tags: [widgets]
signatures:
  - 'widget overflow at 0x[0-9a-f]+'
tools: [peinfo]
skills: [re-observe]
docs: [docs/working/knowhow.md]
seen: ['the sample game, 2026-10-23']
---

# The sample tool stops with a widget overflow

## Symptoms

"widget overflow at 0x1234".

## Cause

Too many widgets.

## What to do

Count the widgets first.
