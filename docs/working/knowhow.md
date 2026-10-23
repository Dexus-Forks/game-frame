# Know-how: failure modes and context

An old game can fail to run in many small ways. A tool that reads its files can fail too. The
first time, such a failure can cost hours. When someone wrote it down and the agent finds the
note, it costs minutes. GameFrame's know-how is the record of these failures. It is a set of
entries in [docs/knowhow/](../knowhow/README.md), one per failure mode or piece of context.
Agents reach it in three ways:

- they ask, with `./gameframe advise "<error text or topic>"`;
- a hook names the matching entries after a shell command whose output matches a known error;
- the same hook names a context entry when the work turns to its runtime, such as the first
  `wine` command of a session.

Some entries give a fix. Others record a dead end, with what was tried. A dead end saves time
too. The next agent does not try the same thing again, unless it has a new idea.

## The taxonomy

Facets place each entry. Each facet has a fixed list of values, kept with their aliases in
[docs/knowhow/README.md](../knowhow/README.md#taxonomy). The aliases are the other words people
and logs use: `ddraw` for `directdraw`, `win95` for `windows-9x`.

| Facet | What it places | Examples |
|---|---|---|
| `era` | When the game was made | `early-1990s`, `late-1990s` |
| `platform` | The system the game was made for, or the machine that runs the tools | `dos`, `windows-9x`, `macos` |
| `runtime` | What runs the game now | `wine`, `rosetta`, `wow64`, `dosbox`, `unicorn` |
| `hardware` | The part of the machine involved | `display`, `cd-rom`, `ports`, `cyrix`, `3d-card` |
| `api` | The programming interface involved | `win32`, `directdraw`, `directinput`, `glide` |
| `compiler` | The compiler, linker or runtime library that made the binary | `watcom`, `msvc`, `borland` |
| `genre` | The kind of game, as in [docs/games/](../games/README.md) | `racing-sports`, `first-person` |
| `symptom` | What the person sees | `crash`, `hang`, `wrong-picture`, `no-input`, `file-not-found`, `tool-error` |

Free `tags` add anything else: `cpu-probe`, `mutex`, `ini`.

A failure mode is also keyed by its **error signatures**. A signature is a regular expression
that matches text the failure prints. It can match a message (`privileged instruction`), an
exception code (`c0000096`), or a faulting module and offset (`wow64cpu\+0x113[59]`). A
signature should match this failure and little else. A failure that prints nothing, such as a
hang, has no signature. `advise` finds it by its facets and its words.

## The entry format

An entry is one Markdown file. Its name is its id: `wine-privileged-instruction.md`. The front
matter is YAML, kept to three forms so the standard library can read it: `key: value`,
`key: [a, b]`, and a list of `- 'item'` lines. Write each regular expression in single quotes.
Inside single quotes a backslash stays a backslash.

```yaml
---
id: wine-privileged-instruction
title: A privileged instruction under Wine, from a Windows 95 port probe
kind: failure                 # failure or context
status: solved                # solved, workaround, dead-end, open; context for a context entry
era: [late-1990s]
platform: [windows-9x]
runtime: [wine]
hardware: [ports, cyrix, x86-cpu]
api: []
compiler: []
genre: []
symptom: [crash]
tags: [cpu-probe, port-io]
signatures:                   # matched against output, case-insensitively
  - 'privileged instruction'
  - '\bc0000096\b'
commands: []                  # context entries: matched against a shell command
tools: [insns]                # GameFrame tools, by the names `gameframe tools` shows
skills: [re-observe]
docs: [docs/observe/running-the-original.md]
seen: ['POD: Planet of Death (1997), 2026-10-23']
---
```

The body has sections with fixed titles. `advise` prints the first paragraph of one of them.

| Section | In | Holds |
|---|---|---|
| `## Symptoms` | every failure | What is seen: the message, the exception, the picture, with the exact text |
| `## Cause` | `solved`, `workaround` | Why it happens, with its evidence label (*measured*, *observed*, *inferred*) |
| `## Evidence` | any | How the cause was found: the trace channel, the breakpoint, the tool |
| `## What to do` | `solved`, `workaround` | The fix, as steps and commands. Its first paragraph is the short answer |
| `## Tried` | `dead-end`, `open` (and any) | One line per attempt and its result. The list is the point of a dead end |
| `## What to know` | `context` | The facts for that era, runtime or hardware. Its first paragraph is the short answer |

`./gameframe advise --check` checks every entry. The id must match the file name. The kind and
status must be known. Every facet value must be in the taxonomy. The regular expressions must
compile. The tools, skills and documents it names must exist. Each status needs its sections.
A failure must name at least one symptom. `tests/run.sh` runs the check.

The facts in an entry follow [the evidence rules](evidence.md). An entry says which game it was
seen in (`seen`). It holds nothing from that game beyond addresses and short message texts.

## Asking: `gameframe advise`

```
./gameframe advise "A privileged instruction was executed at 004a88cd"
wine GAME.EXE 2>&1 | tail -100 | ./gameframe advise -
./gameframe advise directdraw wine --why
./gameframe advise --entry wine-wow64-rosetta-fault
./gameframe advise --list
./gameframe advise "..." --json
```

For each entry `advise` prints its status, its title, the short answer (what to do, or for an
open entry how many things were tried and what), its tools, skills and documents, and the command
that prints it in full. When nothing matches, `advise` exits 1. It then names the skill section
that says how to add an entry.

The search uses plain text matching. It weighs matches in this order
(`tools/gameframe/advise.py`):

1. An error signature that matches the text (100 points). This finds a failure by its log.
2. A facet value or tag that the text names, by its name or an alias (12 points each). This
   finds a topic, such as `directdraw wine`.
3. The text's words in the entry's title (4 points each).
4. The text's words in the entry's body (1 point each, at most 6). A body-only match needs two
   words.

`--signatures` keeps only the first kind. The hook uses only signatures and command patterns.
On ordinary output it says nothing.

### Why there are no embeddings yet

Keywords, aliases and regular expressions are enough for the first hundred entries. Error text
is exact. A signature finds `wow64cpu+0x1135` better than any similarity score would. The
taxonomy's aliases cover most of the different words people use for one topic. The search
needs no model, index or network. It runs inside a hook in milliseconds. It gives the same
answer on every machine.

Embeddings become worth their cost when one of these happens in practice:

- people ask in their own words and `advise` misses entries that the words do not share, even
  after adding aliases;
- there are so many entries that word scores no longer rank the right one first;
- entries come in other languages.

Then the plan is a small local index built from the entries (`gameframe advise --index`). The
index would only rank entries. The signatures would still decide what the hook shows. Until
then, record each missed query in [GAPS.md](../../GAPS.md), so the decision rests on evidence.

## Being told: the hook

`scripts/hooks/advise.py` runs after every shell command in Claude Code (`PostToolUse` and
`PostToolUseFailure`, matcher `Bash`, in `hooks/hooks.json` and `.claude/settings.json`). It
reads the command and its output and looks for:

- entries whose error signature matches the last 40,000 characters of the output, whether the
  command failed or not (Wine prints its errors and still exits 0);
- context entries whose `commands` pattern matches the command line, such as `\bwine\b`.

It adds at most three entries to the agent's context, each as `advise` prints it. It names an
entry once per session, so a repeated error does not repeat the note. It never blocks a command.
When anything goes wrong inside the hook, it exits 0 and says nothing.

Other agents can run the same check by hand (`... 2>&1 | ./gameframe advise - --signatures`).
Their post-tool hooks (Codex, Gemini CLI, Cursor, Copilot) take a different event format. The
next step is to add them to `scripts/hooks/advise.py`. That work follows the pattern of
`scripts/hooks/gates/adapters.py`.

## The MCP tool (planned)

An MCP server `gameframe-knowhow` would give clients without shell hooks the same two calls:
`advise(text, signatures_only)` and `entry(id)`, each returning the JSON of `advise --json`. It
would wrap `tools/gameframe/advise.py`. It would need no index. It is not built yet.
`./gameframe advise --json` serves every client that can run a command.

## Adding entries

Every slice ends with [re-land](../../skills/re-land/SKILL.md). Its "Know-how" step asks
whether this slice hit a failure, a dead end or a fact about the era that the next game would
hit too. If it did, follow these steps.

1. Run `./gameframe advise "<the error text>"`. If an entry exists, add to it: a new attempt
   under `## Tried`, a new signature, a fix that turns `open` into `solved`.
2. Otherwise copy [docs/knowhow/_template.md](../knowhow/_template.md) to a new file named by
   its id. Fill in the facets, at least one signature when the failure prints anything, and the
   sections its status needs.
3. Keep game-specific entries in the project's own `docs/knowhow/`. Move an entry to
   GameFrame's when it holds for other games, with a test when it changes a tool.
4. Run `./gameframe advise --check`. Then run `advise` with the error text again. The entry
   should come first.

Write an open entry too. Its list of what was tried saves the next session time.

## What it does not do

It does not decide anything. An entry says what happened and what worked. The agent and the
user make the choice. It does not learn by itself. People and agents write the entries and
review them like any other document. It does not search the skills or the guides. `gameframe
find` searches those. Each entry links to the guides it rests on.
