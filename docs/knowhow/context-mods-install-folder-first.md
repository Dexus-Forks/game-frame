---
id: context-mods-install-folder-first
title: 'CD-era games often look in their install folder before the CD: a mod can be a file, with no patch'
kind: context
status: context
era: [late-1990s]
platform: [windows-9x]
runtime: [wine, virtual-machine]
hardware: [cd-rom]
api: [win32]
genre: []
tags: [modding, file-open, install-folder, ini]
commands: []
tools: [modfolder]
skills: [re-trace, re-observe]
docs: [docs/knowhow/insert-cd-file-open-failed.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---

# CD-era games often look in their install folder before the CD: a mod can be a file, with no patch

## What to know

Games of the late 1990s kept their large data on the CD and a few files on the hard disk. Many
open every data file through one wrapper that builds two names: one under the install folder
(read from an ini file or the registry), one under the CD's root. The wrapper tries the
install folder first and falls back to the CD. A changed file placed at the same relative
path under the install folder then replaces the disc's copy, with no patch to the game and no
new disc image. Removing the file undoes the mod.

The same wrapper is often the one that asks for the CD when both opens fail (see
[insert-cd-file-open-failed](insert-cd-file-open-failed.md)).
Some folders are install-folder only: the game's own saves, settings and records.

## Evidence

In POD: Planet of Death (1997), `FUN_00479954` reads `Directory` and `Language` from
`Ubi.ini` [POD2_0]. `open_data_file` (`FUN_0047ba70`) calls `folder_path` (`FUN_0047aeb4`),
a `switch` on a folder number that builds `Directory\Data\Binary\<Folder>\name` and the same
under the CD root, then `CreateFileA` on the first and, on failure, the second (*observed*).
Folders 8 to 12 (records, championship, setups, modem) have no CD path. A changed text table,
menu image, track and car placed under `C:\POD\Data\Binary\...` all loaded in the unchanged
game (*measured*, 2026-10-23).

## What to do

Before building a mod loader, find the game's file-open wrapper and read how it builds a
name:

1. Find the wrapper: the function that calls `CreateFileA` (or `_open`, `fopen`) for data
   files. The string table usually holds the folder names (`Data\Binary\Voitures\`) and the
   ini key (`Directory`); their only reference is the path builder.
2. List the folder numbers and the roots each one tries, and in what order. Note the folders
   with no CD fallback and the language folder.
3. Test with the smallest visible change: one string in a text table. Copy the changed file
   under the install folder at the same path and start the game.
4. Keep mods as a folder tree laid out like the disc (`Data/Binary/<Folder>/FILE`), so
   installing is a copy and removing is a delete: `./gameframe run modfolder install MOD_DIR
   INSTALL_DIR`, `list` and `clear`. Under Wine the folder names match in any
   case.

If the game reads only from the CD, the mod needs a changed disc image or a patched path
builder instead.
