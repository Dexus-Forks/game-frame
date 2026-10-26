---
name: re-site
description: Build and keep a project's website - a VitePress site in sites/NAME/ (or one site/) that logs the work in live diary entries during the day, one per milestone, consolidated into one update-notes post per past day, and shows the project's own documents (findings, formats, status) through @include, so they have one copy. Readable for players, modders and other researchers, never a dump of commits or tool output, and never game material. Use when a project starts publishing, after a milestone or a release, and when the documents change shape.
---

# Build the project's website

A project's website tells people what the project found and how it is known. Its readers were
not there for the work. Write every page for them: a player who wants to know what works, a
modder who wants a format, a researcher who wants the evidence, and you in six months.

The site is a view of the repository. The reference pages show the documents in `docs/` with
VitePress's `<!--@include: ...-->`, so each fact has one copy and the site never drifts from the
repository. The only text written for the site alone is the front page and the diary.

## 0. Has it been done?

```bash
ls site sites 2>/dev/null             # an existing site: extend it, do not start another
git log --oneline -- site sites | head
```

## 1. Decide the sites

- **One project, one site** is the usual case: `sites/<name>/`, for example `sites/pod/`.
- **Several sites** suit a project with distinct readers: the port's site for players, and a
  format reference for modders. Each gets its own folder in `sites/`. Do not make a site per
  topic. A topic is a page.
- A project that will only ever have one site may use `site/` instead (`--single`). Either
  layout builds with the same workflow.

Ask the user when it is not clear which readers the site is for. Write the answer in one
sentence at the top of the front page.

## 2. Make it

```bash
./gameframe run new-site NAME "Title" "One sentence: what it is and for whom"     # sites/NAME/
./gameframe run new-site NAME "Title" "..." --single                              # site/
./gameframe run new-site NAME "Title" "..." --manual                              # publish by hand only
cd sites/NAME && npm install && npm run dev                                       # http://localhost:5173
```

(In GameFrame or a copied project: `scripts/new-site.sh` with the same arguments.)
`npm install` downloads VitePress: it is on the project's ask list, so the user approves it.
The script writes:

| File | What it is |
|---|---|
| `index.md` | The front page. Replace every `TODO(site)` line. Keep YAML values in double quotes |
| `findings.md`, `status.md` | `docs/FINDINGS.md` and `docs/CHECKLIST.md` through `@include` |
| `diary/index.md`, `diary/posts.data.mts` | The list of entries, newest first, with each summary |
| `.vitepress/config.mts` | Title, navigation, the diary in the sidebar, local search, the footer |
| `.vitepress/theme/` | The default theme with four colours to set from the game (`custom.css`), and the style of figures and their captions |
| `public/` | The diary's pictures (WebP) and diagrams (SVG) |
| `.github/workflows/sites.yml` | Builds every site and publishes it on GitHub Pages (once per project) |

Commit `package-lock.json` after the first `npm install`. The `.gitignore` in the site keeps
`node_modules/` and the build out of git.

- **Publishing by hand.** Without `--manual` the workflow builds and publishes on every push
  to `main` that changes the site or `docs/`. A project that pushes often spends many Actions
  minutes that way. With `--manual` it runs only when started from the Actions tab or with
  `gh workflow run sites.yml`. Ask the user which they want. To change an existing workflow,
  delete its `push:` block and keep `workflow_dispatch:`.
- **A theme in the game's style.** Set the colours in `.vitepress/theme/custom.css` from the
  game: its palette or its interface, read in the project's viewer. Keep VitePress's dark and
  light modes, and choose a dark mode default in `config.mts` (`appearance: 'dark'`) when the
  game is dark. Use system fonts. Never load the game's fonts, pictures or logo files from the
  game's data; a logo drawn by the project's viewer, as a small WebP, is a capture like any
  other.

## 3. What goes on it

| Page | It answers | Source |
|---|---|---|
| Front page | What is this, what state is it in, where do I start? | Written for the site: three or four lines and links |
| Diary | What happened, what was found, how is it known? | `diary/YYYY-MM-DD-slug.md`, written for the site |
| Findings | What is known about the game, with evidence? | `@include` of `docs/FINDINGS.md` |
| A page per format or system | The layout, the rules, the constants | `@include` of its document in `docs/` |
| Status | What is done, verified and open? | `@include` of `docs/CHECKLIST.md`, or a short summary of it |
| Changelog | What changed in each release? | `@include` of `CHANGELOG.md`, when the project keeps one |
| How to play or build | The steps, for the port's users | `@include` of the README section or `docs/` guide |

Add a page for a document when a reader would look for it by name: a format, a system, the
save file. Leave out working files: hand-offs, notes in `analysis/`, labels, raw tool output.
A page that only lists other pages is a sidebar group, not a page.

To add a page:

```markdown
<!-- sites/NAME/formats/container.md -->
<!--@include: ../../../docs/formats/container.md-->
```

The path is relative to the page. Then add the page to `nav` or `sidebar` in
`.vitepress/config.mts`. Run `npm run build`: it fails on a dead link, including a link inside
an included document. Fix the link in the document in `docs/`, never in a copy.

## 4. Write the diary: live entries, then one post per day

The diary has two kinds of entry.

- **Live entries** are written during the day, one per milestone, so a reader can follow the
  work as it happens. Name each `diary/YYYY-MM-DD-HHMM-slug.md`, with the time it was written on
  a 24-hour clock (`2026-10-26-1430-fog-tables.md`). The diary shows the time beside the title.
- **Day posts** gather a day's live entries once the day is over. Each past day ends up with
  one post, named `diary/YYYY-MM-DD-slug.md`. Live entries on one day often repeat each other,
  and the day post removes the repeats. The live entries stay on the site. The day post links
  each of them, and the diary's list shows only the day post for that day.

A milestone is something a reader would notice:

- a format decoded on every file of its kind;
- a system ported and verified against the original;
- a release;
- a dead end that cost a day and taught something.

If nothing a reader would notice has changed, write no entry. Short findings wait for the next
one.

A live entry:

```markdown
---
title: A disc, a bad block and the first key
summary: One or two sentences for the diary index - what was found, not what was done.
---

# A disc, a bad block and the first key

*2026-10-23 14:30. Tools: GameFrame, Ghidra 12.1, Python 3.13.*

What the goal was, in one paragraph. Then the story: what was tried, what the evidence showed,
how it was checked (with the numbers: "98 files decode, every checksum matches, every file
writes back byte for byte"), and what it means for a reader (a modder can now ...).

## Next

- Two or three open questions, the same ones as in docs/CHECKLIST.md.
```

### Consolidate each past day

Before you write anything else, look for live entries from a day before today:

```bash
./gameframe run check sites/NAME --days     # or tools/site/check.py sites/NAME --days
```

For each day it lists, read that day's live entries and write one day post. Write it like a
game's update notes: what is new for a reader first, then a little detail, then a short change
list.

```markdown
---
title: Fog, shadows and the first full race
summary: The renderer now draws fog and shadow volumes as the 3D card did, and a whole race runs from the menu.
---

# Fog, shadows and the first full race

*2026-10-25. Tools: GameFrame, Ghidra 12.1.*

## What is new

- **Fog as the card drew it.** One or two sentences each: what a reader can now see or do, and
  how it is known (measured against the original, 500 frames match).
- **Shadow volumes.** ...

## How it works

A few paragraphs on the one or two findings that matter most, with their figures. Link the
documents for the detail.

## Changes

- The texture loader reads the 16-bit formats.
- The menu starts a race.

## Next

- The open questions left at the end of the day.

## The day's entries

- [14:30 · Fog tables measured](./2026-10-25-1430-fog-tables)
- [17:05 · Shadow volumes drawn](./2026-10-25-1705-shadow-volumes)
```

Then:

1. Keep every figure the live entries used, with its caption. Drop repeats.
2. Keep each finding once, in its newest and most exact form.
3. The change list says what changed for a reader in plain words. It is not a commit list.
4. Keep the day's live entries. Never delete them. End the day post with "The day's entries",
   a list that links each one by its file name, with its time and title, in time order.
5. Run the check again. It fails while a past day has live entries and no day post, and while a
   day post misses a link to one of its live entries.

The diary's list and sidebar hide a past day's live entries once its day post exists, so a
reader finds them through that post. A project that writes no day posts runs
`check.py --keep-live`, and says so in its own `AGENTS.md`.

The rules for an entry:

- **Tell it as findings.** Each paragraph says what is now known and how it is known: observed,
  measured, inferred or unknown, as in `re-land`.
- **Link, do not repeat.** The layout of a format lives in its document. The entry says what was
  found and links the page.
- **No commit lists.** Never paste `git log`, commit hashes or file lists. To show the work,
  link the compare range on the repository's host, once.
- **No tool dumps.** Describe output in a sentence with the numbers that matter. A short table
  of measured values is fine when it is the point.
- **Pictures that show the finding, from the first entry.** Give every entry two to five
  figures. Use screenshots of the original game, screenshots of the project's viewer with a
  decoded file (its `--export` mode writes them, `re-browse`), frames rendered by the port and
  charts of measured values. Draw a diagram for each format and system an entry explains: a
  file's layout with its offsets, a record's fields, a key applied to a name, a table's
  levels, the order of a calculation. Write the diagrams as SVG by hand or with a small script
  from the numbers they show, in the theme's colours. Caption each figure in italics on the
  line after it with what it shows and where it comes from (`*Our viewer, CAR1.BIN: ...*`,
  `*Diagram: the layout of the pack, from FUN_00512198.*`). Put them in `public/` (WebP under
  1 MB, SVG) and link them as `/name.webp`. Adding figures to old entries later costs more
  than drawing them as each entry is written. The game's files themselves stay out: no
  extracted textures, sounds or text in bulk, no archives, no downloads of its data.
- **Length.** A day post is usually 300 to 1,500 words. Longer usually means reference material
  that belongs in `docs/`. A live entry can be short, since the day post gathers them.
- **Plain English.** One idea per sentence, no rhetorical patterns (`docs/working/writing.md`).
  Send long entries to `re-editor` and `re-checker` before they land.
- **Day posts are history.** Consolidating a past day is the one rewrite. After that, do not
  rewrite a day post when a finding changes. Add a line at the top, "Later: ...", with a link to
  the post or the document that has the new answer.

## 5. Keep it current

At each milestone, in this order:

1. Write the day post for each past day that has live entries and no post yet (section 4).
2. Update the documents in `docs/` (`re-land`). The included pages follow by themselves.
3. Write the live entry.
4. Update the front page's status line when the state has changed.
5. Add a page for any new document a reader would look for, and remove pages whose document is
   gone.
6. Check and build:

```bash
./gameframe run check sites/NAME        # or tools/site/check.py sites/NAME
cd sites/NAME && npm run build
```

`check.py` fails on game files, decompiled code (in a page or an included document), missing
includes, diary entries without a title or summary, a past day's live entries with no day post,
a day post that does not link each of its live entries, and leftover
`TODO(site)` lines. It warns about entries that read like a commit log, day posts that are too
short or too long, two posts for one day, and large pictures. Fix every error. Read every warning and fix it, or say in the hand-off why it stays.

7. Land it with `re-land`: stage the site's files by name. The workflow publishes on the next
   push to `main`, or when the user starts it, if it is manual.

## Publishing

`.github/workflows/sites.yml` builds `site/` and every `sites/*/` that has a `package.json`.
A workflow made with `--manual` runs only when started by hand. One site is served at
`https://OWNER.github.io/REPO/`. Several are served at `.../REPO/NAME/`,
with a short index page at `.../REPO/` that links each one. The user turns on Pages in the
repository's settings, with "GitHub Actions" as the source. Publishing makes the site public,
so the user decides when. A private repository can still build the site and keep it private.

## Pitfalls

- **Copies drift.** Copying a document into the site gives two versions within a week. Always
  `@include`.
- **Unquoted YAML.** A colon in an unquoted front-matter value (`tagline: Ubi Soft: 1997`) stops
  the build with a YAML error. Quote every value that has a colon.
- **The base path.** GitHub Pages serves the site under `/REPO/`. Links that start with `/` work
  because the workflow sets `SITE_BASE`. A link written as `https://OWNER.github.io/...` inside
  the site breaks on a fork. Use paths.
- **Dead links from includes.** A document's relative link (`formats/container.md`) works on
  GitHub but resolves against the page on the site. Give the target a page at the same relative
  place, or link the document on the repository's host.
- **The internal checklist as a status page.** `docs/CHECKLIST.md` written in shorthand reads
  badly to a stranger. When it does, write a short `status.md` that summarises it, and link
  the checklist.
