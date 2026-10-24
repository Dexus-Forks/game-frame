---
id: worktree-ignored-folders-missing
title: A session in a git worktree finds no game, no Ghidra export and no Python environment
kind: failure
status: solved
platform: [macos, linux]
genre: []
symptom: [file-not-found, tool-error]
tags: [git, worktree, parallel-sessions, target, analysis, derived, venv, ghidra, lock, agents]
signatures:
  - 'Unable to lock project'
  - 'LockException'
tools: [ghidra-run, ghidra-export]
skills: [re-start, re-land, re-ghidra]
docs: [docs/working/agents.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# A session in a git worktree finds no game, no Ghidra export and no Python environment

## Symptoms

An agent session started in a git worktree (a parallel session, or a tool that makes one for
each task) cannot find `target/`, `analysis/export/`, `derived/`, `.venv` or `.env`. The
setup hook asks for `scripts/setup.sh`. A tool that worked in the main checkout fails with a
missing file. When several sessions run `ghidra-run.sh` at once, all but one stop with
Ghidra's `Unable to lock project` (a `LockException`).

## Cause

A worktree holds the tracked files only. The game, the Ghidra project and export, the
emulator's output and the Python environment are ignored by git on purpose, so a new worktree
has none of them (*observed*, Hardwar's four parallel sessions). Ghidra locks a project while a
headless job runs in it, and `ghidra-run.sh` uses one project, `analysis/ghidra-project`.

## What to do

Use the main checkout's ignored folders by their absolute paths, or link them into the
worktree (the links are ignored too):

```bash
MAIN=$(git worktree list | head -1 | awk '{print $1}')
for d in target analysis derived .venv; do [ -e "$d" ] || ln -s "$MAIN/$d" "$d"; done
[ -e .env ] || cp "$MAIN/.env" .env
```

Write new ground truth and notes into the main checkout's folders through the links, so other
sessions see them. Do not stage the links (`git add path`, never `git add -A`).

Run one Ghidra job at a time per project. Sessions that need Ghidra together take turns, or
each copies the project first and passes `--project DIR` to `ghidra-run.sh` (a copy does not
see labels applied to the original later). Read the export's text files freely: only the
headless job takes the lock.
