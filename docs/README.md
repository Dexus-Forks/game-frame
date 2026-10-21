# Documentation

GameFrame is an agentic framework for reverse engineering games. Its playbook of skills and agents runs inside your coding agent. Its toolkit of tested tools checks every result against the original.

These pages hold everything GameFrame knows about reverse engineering games, apart from the
step-by-step skills (in `.claude/skills/`, listed in the [README](../README.md#skills)). Start with the kind
of game you have, then how to observe it, then the reference pages as you need them.

The website is built from these files where they are (see [site/README.md](../site/README.md)):
the order of this page and of each folder's `README.md` is the order of the site's menu.

## Game preservation

- [Game preservation](preservation/README.md): why older games disappear, what GameFrame
  does to keep them playable, the law, AI, and the work it builds on.

## Kinds of games

- [A taxonomy of games](games/README.md): who drives, the axes, the families, the
  connections between them.

## Platforms

- [Home computers and the PC](computers/README.md): the CPU families, the 8-bit and 16-bit
  home computers, the PC eras and the Mac, with the tools and timing for each.
- [Console games](consoles/README.md): the home consoles by generation, the tools, the routes
  to a port, and matching decompilation.
- [Handhelds](handhelds/README.md): from the Game & Watch and the Game Boy family to the
  Switch, the Steam Deck and phone games, with saves, clocks, links and LCD behaviour.
- [Arcade](arcade/README.md): the arcade boards, with MAME as the record of each board's
  clocks, encryption and refresh rate.

## Technology of the era

- [Hardware of the era](hardware/README.md): CPUs and floating point, 2D graphics, 3D
  accelerators, sound, memory, storage, input and networks.
- [Software engines and middleware](engines/README.md): recognising an engine, the id and
  Build engines, Unreal, other licensed engines, studio engines, middleware.

## Crates

- [Crates](crates/README.md): clean Rust implementations of chips and runtimes old games
  need, such as the OPL2 FM chip, a WMA decoder, the MSVC maths functions and Turbo Pascal's
  `Real`.

## Observing the original

- [Observing the original](observe/README.md): running the original, inputs, the interface,
  an object model of the game.

## Working on a project

- [Working on a project](working/README.md): how agents work well on this, and how to write
  findings.

## Reference

- [Game patterns](GAME-PATTERNS.md): the parts most games are built from, as they appear in
  machine code.
- [DOS games](DOS.md): 16-bit programs, Borland debug symbols, the DOS emulators.
- [Calling conventions and floating point](ABI.md): what decides exactness.
- [What a project can ship](OUTPUTS.md): documents, a website, programs, releases.
- [Setup](SETUP.md): the toolchain.
- [Legal](LEGAL.md): working within the law and with respect for the original.

## Case studies

- [Case studies](case-studies/README.md): three reimplementations and the tooling they built.
