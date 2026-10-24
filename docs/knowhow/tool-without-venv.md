---
id: tool-without-venv
title: 'A GameFrame tool fails with "No module named unicorn", pefile or capstone'
kind: failure
status: solved
symptom: [tool-error]
tags: [python, venv, setup, imports]
signatures:
  - 'No module named ''(?:unicorn|pefile|capstone|frida)'''
tools: [setup]
skills: [re-setup]
docs: [docs/SETUP.md]
seen: ['GameFrame sessions, 2026']
---

# A GameFrame tool fails with "No module named unicorn", pefile or capstone

## Symptoms

`ModuleNotFoundError: No module named 'unicorn'` (or `pefile`, `capstone`, `frida`) when a
tool starts.

## Cause

The tool ran with the system's Python, which lacks GameFrame's packages. They live in a
virtual environment, `.venv`, which `scripts/setup.sh --install` makes.

## What to do

Run tools through the launcher, which uses the `.venv`: `./gameframe run NAME ARGS`. Or call
`.venv/bin/python tools/pe/peinfo.py GAME.EXE`. When there is no `.venv` yet, run `./gameframe doctor
--install` (`scripts/setup.sh --install`). Frida is optional: it comes from
`requirements-live.txt`.
