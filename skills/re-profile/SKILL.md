---
name: re-profile
description: Build an evidence-backed game profile and project quick start after the first inventory. Classify the game, engine, presentation, release, hardware and input; identify project-specific traps and the next useful steps. Ask the human only about unknowns that could change the route or goal. Use after re-inventory and when new evidence changes the profile.
---

# Profile a game

Use this after the first inventory, when the project has enough evidence to make a useful
starting profile. The result is `docs/GAME_PROFILE.md`: a short guide to what this game is, how
it is built, what it supports, what may cause trouble, and where to start.

The profile summarizes the findings. It does not replace `docs/FINDINGS.md`, `docs/versions.md`,
`docs/lore.md` or the evidence in topic documents.

## Gather what is already known

1. Read the project's `AGENTS.md`, `docs/FINDINGS.md`, `docs/CHECKLIST.md` and `docs/versions.md`.
   Read `docs/lore.md` when it exists. Search the project docs and labels for earlier findings.
2. Read the game's manual, readme, setup program text and configuration files in `target/`.
   Use the evidence already collected by `re-inventory`; inspect a binary or data file only when
   it can answer a question the profile needs.
3. Classify the game with `docs/games/README.md`, its family page and
   `docs/games/interaction.md`. Check the relevant engine, platform and hardware pages under
   `docs/engines/`, `docs/computers/`, `docs/consoles/` and `docs/hardware/`.
4. Read `docs/working/knowhow.md`. Use `./gameframe advise TOPIC` for likely era, runtime or
   hardware pitfalls. A missing know-how entry is not evidence that the issue does not apply.

Do not repeat the whole inventory. Follow a lead only far enough to classify it or mark it as
unknown for the next stage.

## Build the profile from evidence

Record the game's identity and the edition being studied, including release dates when known.
Use `docs/versions.md` as the source for the reference build. Record regional release dates
separately when they differ.

Describe the game on the axes that help choose a route:

- genre and family;
- who drives the game and its time model;
- presentation: text, 2D, isometric, 2.5D or 3D;
- engine, middleware, programming language or scripting runtime, when known;
- target system and the places where the rules live: code, scripts, data or a known engine;
- graphics APIs, display modes, graphics cards, sound systems and input devices.

For graphics, sound and input, keep these kinds of evidence separate:

- **Officially documented:** listed in the manual, readme, setup program, vendor notes or other
  official source. Cite the exact source.
- **Found in the files or code:** imports, driver files, configuration, device probes or code
  paths. Mark the claim *observed* and say what the evidence does and does not prove.
- **Measured in a run:** state the build, emulator or hardware, configuration and result. Mark
  it *measured*.
- **Community-reported:** cite the report and call the compatibility unverified until tested.
- **Inferred or unknown:** label it *inferred* or *unknown* and say what would settle it.

Use primary sources for official support. Use community reports only for unofficial support.
If sources conflict, show the disagreement. A missing device name in a manual does not prove the
game cannot use that device. Do not describe a code path or a report as a successful runtime
test.

When local files do not establish a release date or hardware claim, use the agent's web research
tools if available. Prefer the publisher, developer, archived official manual, patch notes or
vendor documentation. For community compatibility, cite the specific report and date. If research
tools or sources are unavailable, leave the claim *unknown*.

Every factual claim follows the evidence labels in the project's `AGENTS.md`: *observed*,
*measured*, *inferred* or *unknown*. Include a local document and section, a binary address or
string, or a direct source link. A community post is *observed* as a report; it does not prove
that the game works with that hardware. Human answers are marked as user-provided choices, not
as observations about the game.

## Decide whether to ask the human

Use the evidence first. Do not ask for a fact that can be read from the game, its files or a
reliable source. Ask only when an answer could change the recommended route, the project goal or
what must be exact. Useful questions include:

- Is the goal a reimplementation, a mod or editor, a format reader, or documentation?
- Which platforms or hardware must the result support?
- How closely must the interface and non-gameplay behavior match?
- Which edition or behavior should be the reference when the sources disagree?

Ask only the questions that matter for this project. If you can ask the user in the current
session, ask directly and keep the questions concise. If you are running as a subagent or cannot
wait for an answer, put the few material questions under Open in the hand-off. Keep unanswered
items *unknown* and continue with work that does not depend on them.

## Turn the profile into a quick start

Use the game's family, engine and platform to choose the next steps from existing GameFrame
guides. Give each step a reason tied to the evidence. Include likely traps only when this game's
profile supports them, and give a cheap check for each one. Examples include a DOS game whose
speed depends on CPU timing, a 3D game with per-card builds, a managed game whose rules live in
assemblies, or a game that reads most of its rules from data.

Keep the first actions short and concrete. Name the command or skill, the file or binary it
applies to, and what result to record. Do not turn the profile into a generic checklist of every
GameFrame tool.

## Write and update the profile

Fill the project's `docs/GAME_PROFILE.md` template. If the project does not have one, create it
with the sections Summary, At a glance, Graphics and display, Sound, Input, Traps to check early,
Quick start, Questions for the user, and Evidence and sources. Keep it concise and link to the
detailed findings instead of repeating them. Include:

1. a one-paragraph summary and the reference edition;
2. the game and technology classifications above;
3. separate official, file/code, measured and community-reported hardware support;
4. the user's goal and constraints, when known;
5. project-specific traps, the first check for each, and a short ordered quick start;
6. unresolved questions and what evidence would answer them.

Update `docs/FINDINGS.md` with a link to the profile and `docs/CHECKLIST.md` with its state.
Revise the profile when a new binary, edition, runtime observation or hardware test changes a
claim or the recommended route. Preserve the earlier evidence in its detailed document.

## Done

The project has a concise profile with evidence for each claim, official and unofficial support
clearly distinguished, a useful first route, and only material questions left open. The profile
does not claim that an untested behavior has been measured.
