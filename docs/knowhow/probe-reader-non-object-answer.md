---
id: probe-reader-non-object-answer
title: 'A script driving a JSON-lines probe hangs when an answer is not an object'
kind: failure
status: solved
era: []
platform: []
runtime: []
hardware: []
api: []
genre: []
symptom: [hang]
tags: [probe, json-lines, driver-script]
signatures: []
tools: []
skills: [re-verify]
docs: []
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A script driving a JSON-lines probe hangs when an answer is not an object

## Symptoms

A driver script that talks to a game's probe (one command a line, one JSON answer a line)
stops; the game process and the script both sit at 0% CPU.

## Cause

The script skipped every line that did not start with `{` or `[`, to step over log lines. An
answer can be any JSON value (`null`, a number, a string), so the script skipped the answer
and waited for a line that never came, while the game waited for the next command
(*observed* with Un-POD's probe, 2026-10-24).

## What to do

Parse each line as JSON and treat any value as the answer; send log lines to stderr in the
program so they never share stdout with answers.
