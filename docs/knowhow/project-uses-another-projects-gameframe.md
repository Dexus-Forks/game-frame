---
id: project-uses-another-projects-gameframe
title: A project's ./gameframe runs a second clone or another project's copy of GameFrame, and a tool is missing
kind: failure
status: solved
platform: [macos, linux]
genre: []
symptom: [tool-error, file-not-found]
tags: [gameframe-root, env, clone, symlink, worktree, parallel-sessions, venv, pe32_emu, setup]
signatures:
  - "ModuleNotFoundError: No module named 'pe32_emu'"
  - "ModuleNotFoundError: No module named 'pefile'"
tools: []
skills: [re-start, re-setup]
docs: [docs/working/environments.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# A project's ./gameframe runs a second clone or another project's copy of GameFrame, and a tool is missing

## Symptoms

A tool or emulator script that GameFrame has fails with a missing module (`pe32_emu`,
`pefile`), or behaves like an older version than the one on GitHub. `./gameframe root` prints
a folder that belongs to another game, a symlink, or a clone made for one game. In a git
worktree of the project it may print a different folder again.

## Cause

The project runs a GameFrame other than the machine's one checkout. Older launchers took the
first GameFrame they found, starting with `../game-frame` next to the project. Two projects
kept side by side in one folder then shared whatever sat there. In Hardwar's case it was a
symlink to a second clone made for another game, which had fallen behind and lacked the
32-bit emulator the project's scripts import. A git worktree of the project has no
`../game-frame` at all, so it fell through to the next match under `~/code` (*observed*,
Hardwar's parallel sessions). A clone made for one game drifts the same way: its tools go
stale, and its lessons reach GitHub only when someone pushes them.

## What to do

Keep one checkout of GameFrame on the machine, and pin every project to it in the project's
git-ignored `.env`:

```bash
GAMEFRAME_ROOT=~/code/game-frame
```

The launcher reads it from the project and from its main checkout, so worktrees use the same
one (`docs/working/environments.md`, "One GameFrame checkout"). Projects made by
`scripts/new-project.sh` get the line already; an older project needs the current launcher
(`scripts/update-project.sh`). Check with `./gameframe root` in the main checkout and in a
worktree. To change GameFrame from a game's session, use a git worktree of that one checkout,
not a second clone. Remove clones made for single games once their work is merged.
