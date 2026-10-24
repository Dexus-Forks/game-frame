# Versions of GAME_NAME

Every edition of the game known to this project, how they differ, and which one is the
reference. List them before deep work: a newer edition found late costs the work again
(GameFrame's `docs/working/versions.md`). Game files stay out of git: keep each edition in its
own folder under `target/` (`target/first`, `target/gold`).

## Editions

| | First edition | Next edition |
|---|---|---|
| Source | where it came from (a disc, a store, an archive item) | |
| Files and checksums | the image or installer, its size and MD5 or SHA-256 | |
| Dates | mastered, executables linked (PE link dates), readme version | |
| Languages | text, voices, pictures | |
| Content | levels, tracks, characters, modes | |
| Builds | each executable and what it is for (software, MMX, a 3D card) | |
| Also on the disc | readmes, manuals, tools, extras | |

*Measured* DATE: checksums against the source's; dates from the PE headers
(`tools/triage/versiondiff.py`); the rest from the readme.

## Build identity

What each copy says about its own version, written by
`tools/triage/buildid.py target/EDITION --name EDITION --write docs/versions.md`: the disc's
mastering date, each executable's link date and VERSIONINFO, version strings in the code, and
version lines in the readme and setup files.

## Is this the latest official version?

Official releases found, with the source of each. Only the publisher's or developer's own
releases count: editions, regional and language releases, re-releases ("Gold", "GOTY"),
patches and updates, and digital re-releases (GOG, Steam). Fan patches, cracks and mods go in
the extras table, as context, never as the reference.

| Release | Date | Version | Source (URL) | Checked |
|---|---|---|---|---|
| | | | MobyGames, PCGamingWiki, the publisher's site on archive.org, GOG, Steam | DATE |

Answer: this project's copy is / is not the latest official build. If not, the latest is ...,
documented at ... .

## Updates, patches and extras

| File | What it is | Used |
|---|---|---|
| | an official patch, a fan patch, extra levels, an era tool | yes / no, and why |

## What changed in the data

Paste the output of `tools/triage/versiondiff.py OLD NEW` here, then say what each change is:
new languages, new levels, changed formats (a reader that needs a variant), files gone.

## What changed in the code

From the Ghidra exports of both editions, matched with `tools/triage/match_functions.py`
(`labels/NEW/map.tsv` maps old addresses to new):

| | Count |
|---|---|
| Functions matched and identical | |
| Matched and changed | |
| New in the newer edition | |
| Gone | |

| Function (old → new) | What changed | Matters to the port |
|---|---|---|
| | a constant, a condition, a new call | yes / no |

## The reference

**EDITION is the reference** (decided DATE: by default the last version released, which has
the most fixes). Say why when it is not the latest. Findings made on another edition cite
their edition, and each is checked again on the reference.
