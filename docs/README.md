# Documentation

Everything the harness knows about reverse engineering games, apart from the step-by-step
skills (in `.claude/skills/`, listed in the [README](../README.md#skills)). Start with the kind
of game you have, then how to observe it, then the reference pages as you need them.

The website is built from these files where they are (see [site/README.md](../site/README.md)):
the order of this page and of each folder's `README.md` is the order of the site's menu.

## Game preservation

- [Game preservation](preservation/README.md): why older games disappear, what the harness
  does to keep them playable, the law, AI, and the work it builds on.

## Kinds of games

- [A taxonomy of games](games/README.md): who drives, the axes, the families, the
  connections between them.

## Engines, hardware and consoles

- [Software engines and middleware](engines/README.md): recognising an engine, the id and
  Build engines, Unreal, other licensed engines, studio engines, middleware.
- [Hardware of the era](hardware/README.md): CPUs and floating point, 2D graphics, 3D
  accelerators, sound, memory, storage, input and networks.
- [Console games](consoles/README.md): the platforms, the tools, the routes to a port, and
  matching decompilation.

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
