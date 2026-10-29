# The harness: modules around a game

A game needs things around it to run: a CPU with the right features, an operating system, its
graphics and sound APIs, its libraries, and its DRM or store client. Each project used to find
these one failure at a time. The harness writes each answer down once, as a module, so the
next game that needs the same thing starts with it.

There are three parts:

1. **The analyser**, `tools/triage/needs.py`, reads an executable, a game's folder or a disc
   image. It lists the game's needs as `kind:value` keys with their evidence:
   `api:glide2` because `GAME.EXE imports glide2x.dll`, `drm:steam` because of
   `steam_api64.dll`, `os:windows-3x` from an NE header.
2. **The modules**, in `docs/harness/modules/` here and in a project's own
   `docs/harness/modules/`. A module says which needs it provides, which runtime it works in,
   which modules it needs first, how to set it up, and how to check it works.
3. **The loader**, `tools/live/harness.py`. It runs the analyser, picks a runtime module that
   covers the most needs, adds a layer module for each need left, and writes the plan. With
   `--script` it writes the set-up steps as one shell script for you to read and run.

```bash
./gameframe run needs game/                         # what the game needs
./gameframe run harness plan game/                  # which modules cover it, and what is missing
./gameframe run harness plan game/ --runtime emupc-wine --with silent-audio --script derived/harness.sh
./gameframe run harness modules                       # every module
```

## Needs

A need is a kind and a value. The kinds are `cpu`, `os`, `extender`, `api`, `lib`, `drm`,
`store` and `media`. `needs_signs.py` beside the analyser holds every sign it reads and the need
each sign shows. A need is a lead and never a proof. An import shows the program can use an
API, and a build may never call it. Confirm a need by running the game.

## Modules

A module is one Markdown file. Its front matter uses the same small YAML subset as the
know-how entries:

| Field | What |
|---|---|
| `id` | the file name without `.md` |
| `title` | one line: what the module gives the game |
| `kind` | `runtime` (what the game runs in: Wine, an emulated PC, DOSBox-X) or `layer` (something added inside a runtime) |
| `runtime` | for a layer, the runtimes it works in; for a runtime, its own id |
| `provides` | the needs it covers |
| `requires` | the modules it needs first |
| `vars` | the variables its steps read, such as `STEAM_APP_ID` |
| `seen` | the games it was proved on, with the date |

The body has these sections:

- **What it does**: one paragraph.
- **Set up**: the steps, as `bash` blocks. The loader copies these blocks into the script, in
  order.
- **Check**: how to see that it works.
- **Pitfalls**: what went wrong, and what each symptom means.

The steps must run without a person, except where a person must act, such as signing in to a
store. Mark those steps, and never type the owner's credentials.

## Learning modules from games

When a game starts to run because of a change around it, write that change as a module. Copy
`modules/_template.md`. Put a module that only this game needs in the project's
`docs/harness/modules/`. Put one that any game with the same need could use in GameFrame's
(AGENTS.md rule 8). Add the game to `seen` each time a module works for another game. When a
module fails for a game, add the symptom to its Pitfalls, or write a know-how entry
(`gameframe advise`).

## Vision

When a step needs to find something on the screen, such as a button to click in a VM,
`tools/live/align.py` lines up a reference picture with a capture of the screen and maps points
between them. When the screen has changed too much to line up, `tools/llm/locate.py` asks a
model to find the element, on API credits ([API credits](../working/api-credits.md)).
