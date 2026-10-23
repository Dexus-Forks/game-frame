---
name: re-site
description: Build and keep a project's website - a VitePress site in sites/NAME/ (or one site/) that logs the work in dated diary entries, one per milestone, and shows the project's own documents (findings, formats, status) through @include, so they have one copy. Readable for players, modders and other researchers, never a dump of commits or tool output, and never game material. Use when a project starts publishing, after a milestone or a release, and when the documents change shape.
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
| `.github/workflows/sites.yml` | Builds every site and publishes it on GitHub Pages (once per project) |

Commit `package-lock.json` after the first `npm install`. The `.gitignore` in the site keeps
`node_modules/` and the build out of git.

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

## 4. Write a diary entry

Write one entry per milestone, not per session or per commit. A milestone is something a reader
would notice:

- a format decoded on every file of its kind;
- a system ported and verified against the original;
- a release;
- a dead end that cost a day and taught something.

If nothing a reader would notice has changed, write no entry. Short findings wait for the next
one.

Name the file `diary/YYYY-MM-DD-slug.md`, dated the day the milestone landed:

```markdown
---
title: A disc, a bad block and the first key
summary: One or two sentences for the diary index - what was found, not what was done.
---

# A disc, a bad block and the first key

*2026-10-23. Tools: GameFrame, Ghidra 12.1, Python 3.13.*

What the goal was, in one paragraph. Then the story: what was tried, what the evidence showed,
how it was checked (with the numbers: "98 files decode, every checksum matches, every file
writes back byte for byte"), and what it means for a reader (a modder can now ...).

## Next

- Two or three open questions, the same ones as in docs/CHECKLIST.md.
```

The rules for an entry:

- **Tell it as findings.** Each paragraph says what is now known and how it is known: observed,
  measured, inferred or unknown, as in `re-land`.
- **Link, do not repeat.** The layout of a format lives in its document. The entry says what was
  found and links the page.
- **No commit lists.** Never paste `git log`, commit hashes or file lists. To show the work,
  link the compare range on the repository's host, once.
- **No tool dumps.** Describe output in a sentence with the numbers that matter. A short table
  of measured values is fine when it is the point.
- **Pictures that show the finding.** Screenshots of the original game, screenshots of the
  project's viewer with a decoded file, frames rendered by the port, diagrams, charts of
  measured values. Choose a few for each entry and caption each with what it shows and where it
  comes from (`The original, track 3`, `Our viewer, CAR1.BIN`). Put them in `public/` (WebP,
  under 1 MB) and link them as `/name.webp`. The game's files themselves stay out: no
  extracted textures, sounds or text in bulk, no archives, no downloads of its data.
- **Length.** Usually 300 to 1,500 words. Shorter is a note: fold it into the next entry.
  Longer usually means reference material that belongs in `docs/`.
- **Plain English.** One idea per sentence, no rhetorical patterns (`docs/working/writing.md`).
  Send long entries to `re-editor` and `re-checker` before they land.
- **Entries are history.** Do not rewrite old entries when a finding changes. Add a line at the
  top, "Later: ...", with a link to the entry or the document that has the new answer.

## 5. Keep it current

At each milestone, in this order:

1. Update the documents in `docs/` (`re-land`). The included pages follow by themselves.
2. Write the diary entry.
3. Update the front page's status line when the state has changed.
4. Add a page for any new document a reader would look for, and remove pages whose document is
   gone.
5. Check and build:

```bash
./gameframe run check sites/NAME        # or tools/site/check.py sites/NAME
cd sites/NAME && npm run build
```

`check.py` fails on game files, decompiled code (in a page or an included document), missing
includes, diary entries without a title or summary, and leftover `TODO(site)` lines. It warns
about entries that read like a commit log, entries that are too short or too long, and large
pictures. Fix every error. Read every warning and fix it, or say in the hand-off why it stays.

6. Land it with `re-land`: stage the site's files by name. The workflow publishes on the next
   push to `main`.

## Publishing

`.github/workflows/sites.yml` builds `site/` and every `sites/*/` that has a `package.json`.
One site is served at `https://OWNER.github.io/REPO/`. Several are served at `.../REPO/NAME/`,
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
