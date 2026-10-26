---
id: context-read-the-story
title: The manual and the program's own text say what a system is for before the code does
kind: context
status: context
genre: [vehicle-sims, management, rpg, adventure, strategy]
tags: [lore, story, manual, strings, sentences, design, setting, rules, what-must-be-exact]
tools: [sentences]
skills: [re-inventory, re-start, re-trace]
docs: [docs/working/lore.md]
seen: ['Hardwar (1998), 2026-10-24']
---

# The manual and the program's own text say what a system is for before the code does

## What to know

Read the game's story early. The manual says what the player plans around, and the program's
long strings often state rules in plain words. Together they say which systems the designers
cared about, and so which the port must match exactly. In Hardwar the craft run on solar cells
and recharge under lights, so light is game logic; two factions hold water and food, so the
economy is the core; the altitude ceiling depends on the power cell; the plot seals tunnels,
so the save holds them (`docs/working/lore.md`).

- List the program's sentences: `./gameframe run sentences game/GAME.EXE > analysis/notes/sentences.txt`.
- Write `docs/lore.md` from the project template, with the table "in the story → in the code →
  evidence" (`re-inventory`, "Read the story").
- When a constant or a system has no reason, read the lore again before guessing.
- Keep the game's text out of tracked files. Name a string by its address and say what it states.
