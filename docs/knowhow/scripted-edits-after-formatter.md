---
id: scripted-edits-after-formatter
title: A scripted edit to source code fails to match after a formatter has run
kind: failure
status: solved
platform: []
genre: []
symptom: [tool-error]
tags: [rustfmt, cargo-fmt, black, prettier, edit, replace, agents, reflow]
signatures:
  - 'String to replace not found'
  - 'old_string not found'
tools: []
skills: [re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# A scripted edit to source code fails to match after a formatter has run

## Symptoms

An agent's exact-text replacement (an edit tool, `sed`, a Python `replace`) reports that the
text to replace is not found, in a file it wrote or read a few steps earlier. The code looks
the same, except for where lines break.

## Cause

A formatter (`cargo fmt`, `rustfmt` on save, `black`, `prettier`) reflowed the file between
the read and the edit: a long call split over several lines, a match arm joined, a trailing
comma added. The text the agent remembers no longer exists byte for byte. In the Hardwar
project it happened several times after `cargo fmt` and `cargo clippy --fix`, while the
crates were being written (*observed*).

## What to do

Read the file again after any formatter run, and match on the formatted text. Keep an edit's
match short and inside one statement, so a reflow elsewhere does not break it. Run the
formatter once at the end of a set of edits rather than between them, then read the result.
When a replacement fails, read the lines around it; do not retry the same text.
