---
layout: home
title: GameFrame
hero:
  name: GameFrame
  text: An agentic framework for reverse engineering games
  tagline: Understand a game from the copy you own, and rebuild it exactly. Its playbook of skills and agents runs inside your coding agent. Its toolkit of tested tools, around Ghidra, Unicorn and REA, checks every result against the original.
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
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="5" rx="1"/><path d="M5 9v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9"/><path d="M10 13h4"/></svg>'
    title: Game preservation
    details: Many older games no longer run on current computers. Some are still on sale but have not been updated in years. This method works from the copy you bought, and records how the game works as it goes.
    link: /preservation/
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><circle cx="17.5" cy="17.5" r="3.5"/></svg>'
    title: A taxonomy of games
    details: Who drives, the game or the player? Ten families of games, each with its rule set, reference games and projects to learn from, and the systems they share.
    link: /games/
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/></svg>'
    title: Observe the original
    details: Run the game where you can watch, drive and capture it. Every kind of input, every screen, and an object model with entry points for tests and agents.
    link: /observe/
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6l1.5 1.5L8 5"/><path d="M4 12l1.5 1.5L8 11"/><path d="M4 18l1.5 1.5L8 17"/><path d="M11 6h9M11 12h9M11 18h9"/></svg>'
    title: Skills for each step
    details: Inventory, observe, decompile, trace, decode formats, emulate, reimplement, verify, land. Each skill is a complete guide an agent can follow alone.
    link: /skills/re-start
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/></svg>'
    title: Measured against the original
    details: Original functions run under emulation for ground truth, and the reimplementation must match their results bit for bit.
    link: /skills/re-emulate
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M2 5h6a4 4 0 0 1 4 4v11a3 3 0 0 0-3-3H2z"/><path d="M22 5h-6a4 4 0 0 0-4 4v11a3 3 0 0 1 3-3h7z"/></svg>'
    title: Case studies
    details: Un-Banished, Tyrian 2000 and Frontier Elite II, and the tooling each built to capture its game.
    link: /case-studies/
  - icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z"/><path d="M9 12l2 2 4-4"/></svg>'
    title: Within the law
    details: You need your own copy of the game. GameFrame ships no game files. Each project writes new code from documented behaviour.
    link: /legal
---

## The method

Each step has a skill. The steps run in order. When a check fails, you go back to the trace step.

![The method as a loop. Inventory, observe, decompile, trace, format, emulate, reimplement, verify and land, each with its skill. When a check fails, the work goes back to the trace.](../docs/assets/method-loop.svg)

## Checked against the original

Every result is checked against the original game. The test suite runs the original's own
functions under emulation and compares the port with them, value by value and random draw by
random draw. A port that is only close fails.

![tests/run.sh --ghidra: the port matches the original bit for bit, draws random numbers in the original's order, and the naive port is caught. 104 passed, 0 failed.](../docs/assets/captures/hero.svg)

Agents can also read this site as plain text. [llms.txt](/llms.txt) lists every
page with a short description, and [llms-full.txt](/llms-full.txt) holds all
of them in one file. Each page has a Markdown copy at the same address with `.md` at the end,
such as [/games/interaction.md](/games/interaction.md).

## The 3D-card builds, on today's machines

Many Windows games of the late 1990s shipped one build for each 3D card: 3dfx, PowerVR, ATI
Rage, S3 ViRGE. No modern system has those cards or their driver libraries. Those builds no
longer start. GameFrame has stand-in DLLs for Glide 2, PowerVR SGL, ATI3DCIF, the S3D Toolkit
and DirectDraw. Copy them beside the game. Its EXE then runs unchanged. The Glide and PowerVR
builds are drawn again with OpenGL at any size. The ATI and S3 builds show their frame scaled.

![POD Gold's PowerVR build in a race at 1280 × 960, drawn by GameFrame's sgl.dll](../docs/assets/case-studies/pod-sgl-race.webp)

They are tested with one game so far, POD: Planet of Death, under Wine. Each library handles
the calls POD makes. It logs the modes and states it does not draw. The first run of a new game
shows much of what is missing. [Replacement 3D-card libraries](../docs/hardware/wrappers.md) says what each one does,
how to build it and how to use it.
