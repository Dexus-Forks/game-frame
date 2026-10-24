---
id: ghidra-dot-folder-path
title: Ghidra headless refuses a project path inside a folder whose name starts with a dot
kind: failure
status: solved
symptom: [tool-error]
tags: [ghidra, headless, worktree, project-path]
tools: [ghidra-export, ghidra-run]
skills: [re-ghidra]
docs: [docs/working/ghidra.md]
seen: ['GameFrame in a git worktree under .claude/worktrees/, 2026']
---

# Ghidra headless refuses a project path inside a folder whose name starts with a dot

## Symptoms

`analyzeHeadless` fails at once on a project in a git worktree such as
`.claude/worktrees/NAME/analysis/ghidra`, or in any path with a folder that starts with ".".

## Cause

Ghidra does not accept a project location with a folder name that starts with a dot
(*observed*).

## What to do

Use GameFrame's scripts, `scripts/ghidra-export.sh` and `scripts/ghidra-run.sh`: they put the
project behind a link in `$TMPDIR` (`ghidra_project_dir` in `scripts/lib.sh`). When calling
`analyzeHeadless` yourself, do the same: make a link outside the dot folder and give Ghidra the
link's path.
