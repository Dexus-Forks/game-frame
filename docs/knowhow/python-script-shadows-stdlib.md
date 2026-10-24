---
id: python-script-shadows-stdlib
title: A helper script named like a standard module breaks imports, as dis.py broke capstone
kind: failure
status: solved
symptom: [tool-error]
tags: [python, import, shadowing, scripts]
signatures:
  - 'partially initialized module ''\w+'' has no attribute'
  - 'module ''(?:dis|inspect|types|token|tokenize|code|copy|random|string|struct|select|signal|queue|enum|ast|io|re)'' has no attribute'
tools: []
skills: [re-emulate]
docs: [docs/working/agents.md]
seen: ['POD: Planet of Death (1997), 2026-10-24']
---

# A helper script named like a standard module breaks imports, as dis.py broke capstone

## Symptoms

A script that imports capstone, or another library, fails with an AttributeError on a standard
module: `module 'dis' has no attribute ...`, or `partially initialized module`.

## Cause

Python puts the script's own folder first on the import path. A file in that folder named like
a standard module (`dis.py`, `code.py`, `struct.py`) is imported in place of that module. The
libraries the script uses import it too. In POD a disassembly helper named `dis.py` broke
capstone. Capstone imports `inspect`, and `inspect` imports `dis` (*observed*).

## What to do

Rename the script (`disasm_at.py`) and delete its `__pycache__` copy. Name helper scripts after
the game or the job (`pod_dis.py`, `measure_gates.py`). A single common word can match the name
of a standard module.
