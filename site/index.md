---
layout: home
title: GameFrame
hero:
  name: GameFrame
  text: An agentic framework for reverse engineering games
  tagline: Understand a game from the copy you own, and rebuild it exactly. A method, step-by-step skills and tested tools for AI agents and people, around Ghidra, Unicorn and REA.
  actions:
    - theme: brand
      text: Start here
      link: /overview
    - theme: alt
      text: What kind of game is it?
      link: /games/
    - theme: alt
      text: The first skill
      link: /skills/re-start
features:
  - title: Game preservation
    details: Many older games no longer run on current computers. Some are still on sale but have not been updated in years. This method works from the copy you bought, and records how the game works as it goes.
    link: /preservation/
  - title: A taxonomy of games
    details: Who drives, the game or the player? Ten families of games, each with its rule set, reference games and projects to learn from, and the systems they share.
    link: /games/
  - title: Observe the original
    details: Run the game where you can watch, drive and capture it. Every kind of input, every screen, and an object model with doors for tests and agents.
    link: /observe/
  - title: Skills for each step
    details: Inventory, observe, decompile, trace, decode formats, emulate, reimplement, verify, land. Each skill is a complete guide an agent can follow alone.
    link: /skills/re-start
  - title: Measured against the original
    details: Original functions run under emulation for ground truth, and the reimplementation must match their results bit for bit.
    link: /skills/re-emulate
  - title: Case studies
    details: Un-Banished, Tyrian 2000 and Frontier Elite II, and the tooling each built to capture its game.
    link: /case-studies/
  - title: Within the law
    details: You need your own copy of the game. The harness ships no game files. Each project writes new code from documented behaviour.
    link: /legal
---

## Set it up with your agent

Copy this prompt into your coding agent (Claude Code, Codex, Cursor or another). It clones the
harness, checks your tools, asks before it installs anything, and starts a project for your game.

```text
Set up the GameFrame harness on this computer for me.

1. Clone https://github.com/tanepiper/game-frame into ~/code/game-frame
   (or update it with git pull if it is already there).
2. Read AGENTS.md and .claude/skills/re-setup/SKILL.md.
3. Run scripts/setup.sh and show me the table of what is present and what is missing.
4. Before you install anything, tell me what scripts/setup.sh --install will install and how
   large each part is, and wait for my answer. Then run it.
5. Run tests/run.sh --ghidra and report how many checks passed. If any failed, show the
   failures and fix the setup before going on.
6. Run scripts/setup.sh --rea and help me register REA with this agent.
7. Ask me for the name of the game and the folder of my own copy of it. Then run
   scripts/new-project.sh ~/re/<short-name> "<Game Name>" "<path to the game>".
8. Open the new project, read its AGENTS.md, and start with the re-start skill.

Never copy the game's files, extracted assets or decompiled code into a git repository.
```
