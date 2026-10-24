---
id: project-uses-another-projects-gameframe
title: A project's ./gameframe runs another project's GameFrame, or the source clone, and a tool is missing
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

# A project's ./gameframe runs another project's GameFrame, or the source clone, and a tool is missing

## Symptoms

A tool or emulator script that GameFrame has fails with a missing module (`pe32_emu`,
`pefile`), or behaves like an older or newer version than the one the project was made with.
`./gameframe root` prints a folder that belongs to another game, a symlink, or the clone you
change and push. In a git worktree of the project it prints a different folder again.

## Cause

Without `GAMEFRAME_ROOT`, the project's `./gameframe` takes the first GameFrame it finds:
`../game-frame` next to the project, then clones under `~/code`, then the plugin's copy. Two
projects kept side by side in one folder share whatever `../game-frame` is. In Hardwar's case
it was a symlink to another project's clone, which lacked the 32-bit emulator the project's
scripts import. A git worktree of the project sits deeper, so it has no `../game-frame` at
all and falls through to the next match under `~/code`, which was the source clone
(*observed*, Hardwar's parallel sessions).

## What to do

Give each game its own clone of GameFrame from GitHub, and name it in the project's
git-ignored `.env`:

```bash
GAMEFRAME_ROOT=~/code/GAME/game-frame
```

The launcher reads it from the project and from its main checkout, so worktrees use the same
clone (`docs/working/environments.md`, "One clone per game"). Projects made by
`scripts/new-project.sh` get the line already. An older project needs the newer launcher
(`scripts/update-project.sh`) or `export GAMEFRAME_ROOT=...` in each shell. Check with
`./gameframe root` in the main checkout and in a worktree. Keep the source clone for
changing GameFrame; don't run projects on it.
