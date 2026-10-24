---
id: game-folder-deleted-as-build-output
title: A disk cleanup deletes a project's target/, the copy of the game, as Cargo build output
kind: failure
status: solved
platform: [macos, linux]
genre: []
symptom: [file-not-found]
tags: [target, cargo, cargo-clean, build-output, cleanup, trash, git-clean, game-folder, backup, saves, agents]
signatures:
  - 'GAME-FILES-NOT-BUILD-OUTPUT'
  - 'is a GameFrame project.s game folder'
tools: [new-project, update-project, setup]
skills: [re-start, re-inventory]
docs: [docs/working/gates.md]
seen: ['POD (1997), Hardwar (1998), Tyrian 2000 (1999), 2026-10-24']
---

# A disk cleanup deletes a project's target/, the copy of the game, as Cargo build output

## Symptoms

A project's `target/` folder is gone, or empty. Tools that read the game fail with a missing
file. `./gameframe doctor` reports `game folder MISSING` or `EMPTY`. The files may be in the
Trash, if the cleanup moved them there.

## Cause

In a GameFrame project `target/` holds the user's own copy of the game: disc images, the
installed or extracted files and saves. Git ignores it, so no commit keeps a copy. Cargo's
build output has the same default name. A project with a Rust port has a `Cargo.toml` beside
`target/`, so the folder looks like build output to people, to cleaners and to agents. The
project's `.cargo/config.toml` sends Cargo to `build/`, but a cleaner does not read it.

On 2026-10-24 a disk cleanup moved every `target/` beside a `Cargo.toml` to the Trash
(*observed*). Three projects lost their game copies: POD, Hardwar and Tyrian. They were rebuilt
from the original disc images. Hardwar's original in-flight save could not be recovered.

The same loss follows from `rm -rf target`, `git clean -fdx` (it deletes ignored files),
`cargo clean --target-dir target`, a project without `.cargo/config.toml`, and build-folder
cleaners such as `kondo`, `cargo sweep` and `npkill`.

## What to do

Restore the game from the discs, an installer or a backup, then run `./gameframe doctor`. It
checks `target/` and writes its note again. GameFrame now guards the folder in four ways:

- `target/GAME-FILES-NOT-BUILD-OUTPUT.txt` says the folder is the game. `new-project.sh`
  writes it, and `update-project.sh` and `./gameframe doctor` add it to older projects. It is
  not a `CACHEDIR.TAG`, which tells backup tools to skip a folder.
- `.cargo/config.toml` sets `target-dir = "build"`. `update-project.sh` adds it when it is
  missing, and warns when an existing one still builds into `target/`.
- The delete gate (`scripts/hooks/gates/deletes.py`) blocks an agent's command that would
  delete or move a game folder. A command that starts with `GAMEFRAME_DELETE_GAME=1` passes,
  for when the user wants the folder gone.
- `./gameframe doctor` counts a missing `target/` as missing. It also counts an empty one
  when the project's documents name files in it.

Before any cleanup by hand or by script, tell the folders apart. A real Cargo `target/` holds a
`CACHEDIR.TAG` file, and a game folder holds none. Skip every folder that holds
`GAME-FILES-NOT-BUILD-OUTPUT.txt`.

Keep a backup of the original discs and saves outside the project. Git ignores them, so they
are the one copy. On macOS, `chflags -R uchg target` makes the files hard to delete or move to
the Trash, and `chflags -R nouchg target` undoes it. Tools that write into `target/` then fail,
so it suits a finished copy.

## Tried

- Rebuilding from the original discs: the game files came back in all three projects.
- Recovering Hardwar's original in-flight save: it could not be recovered.
