---
id: stale-text-buffer
title: 'The text routine''s line buffer outlives each text'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
compiler: []
tags: [buffer, stale, text]
tools: []
skills: [re-emulate, re-port]
docs: []
seen: ['Hardwar (1998), 2026-10-24']
---

# The text routine's line buffer outlives each text

## What to know

*2026-10-24, `text_draw` 0x4ad508, `text_add_char` 0x4ace54, `crates/draw2d`.*

`text_draw` gathers a line in a 160-byte buffer at `0x59a418` and draws it at a line end. The
buffer is not cleared between texts, only its length. When word wrap triggers on an empty
line (a right-aligned text whose x is already past the box), the length becomes -1, the
terminating 0 goes to the byte before the buffer, and the line drawn is whatever the buffer
still holds from earlier texts. 13 of 1,000 random texts differed from the original until the
port kept the buffer in its `Canvas` across texts.

Two more quirks of the same routine: a wrap in a line with no space drops the line's last
character, and justified text (alignment 1) with no space divides by zero. Emulator rigs must
clear `0x59a418` before each case, or a case depends on the one before.
