# Documentation

GameFrame is an agentic framework for reverse engineering games. Its playbook of skills and agents runs inside your coding agent. Its toolkit of tested tools checks every result against the original.

These pages cover reverse engineering games, apart from the
step-by-step skills (in `skills/`, listed in the [README](../README.md#skills)). Start with the kind
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
- [Arcade](arcade/README.md): the arcade boards, with MAME's source as the reference for each board's
  clocks, encryption and refresh rate.

## Technology of the era

- [Hardware of the era](hardware/README.md): CPUs and floating point, 2D graphics, 3D
  accelerators, sound, memory, storage, input and networks.
- [Replacement 3D-card libraries](hardware/wrappers.md): stand-in DLLs that run a 1990s game's
  build for one 3D card on a modern machine, unchanged and at a larger size.
- [Software engines and middleware](engines/README.md): recognising an engine, the id and
  Build engines, Unreal, other licensed engines, studio engines, middleware.
- [Numbers, text and libraries](semantics/README.md): integer and floating-point behaviour on
  each platform, C runtime functions whose results differ, and text encodings.

## Media and files

- [Media, files and protection](media/README.md): CDs and floppies, identifying unknown files,
  packed and protected code.

## Crates

- [Crates](crates/README.md): clean Rust implementations of chips and runtimes old games
  need, such as the OPL2 FM chip, a WMA decoder, the MSVC maths functions and Turbo Pascal's
  `Real`.

## Observing the original

- [Observing the original](observe/README.md): running the original, inputs, the interface,
  an object model of the game.
- [The harness](harness/README.md): what a game needs to run, and the modules that provide it:
  runtimes, layers, and the steps to set each up.

## Agent teams

- [Agent teams](teams/README.md): how the agents work together, the hand-off, where the data
  lives, and eight teams with diagrams.

## Working on a project

- [Working on a project](working/README.md): how agents work well on this, how to write findings,
  and how to separate source analysis from implementation.

## Known failures and context

- [Know-how](knowhow/README.md): failure modes seen before, with their fixes, dead ends and
  what was tried, and what to know for an era, runtime or kind of hardware.
  `./gameframe advise` searches them.

## Reference

- [Game patterns](GAME-PATTERNS.md): the parts most games are built from, as they appear in
  machine code.
- [DOS games](DOS.md): 16-bit programs, Borland debug symbols, the DOS emulators.
- [Calling conventions and floating point](ABI.md): the details a bit-exact port must copy.
- [Reading compiled code](COMPILED-CODE.md): what compilers make of common source, and how to
  find the game's own code past the runtime.
- [Recovering types](TYPES.md): structures, classes, lists, trees, enumerations and the C++
  standard containers, from the code that uses them.
- [Taxonomies](TAXONOMIES.md): shared names for kinds of code, data, values, evidence and
  differences between the port and the original.
- [The data dictionary](DATA-DICTIONARY.md): one record per value the rules use, with its
  place, type, unit, range, meaning and evidence.
- [What a project can ship](OUTPUTS.md): documents, a website, programs, releases.
- [Setup](SETUP.md): the toolchain.
- [Legal](LEGAL.md): working within the law and with respect for the original.

## Case studies

- [Case studies](case-studies/README.md): five projects and the tooling they built.
