# Your own copy and the law

This chapter summarises the law in 35 countries as it stands in October 2026. It is not legal
advice. The law that applies is the law where you live, wherever the code is hosted. If a
project matters, ask a lawyer who knows software law in your country. For the rules every
project follows, read [Legal](../../LEGAL.md).

## The rule that holds everywhere

Most countries let a person who owns a game study how it works. No country lets that person
share the game. Every project made with GameFrame starts from a copy that the person owns.
Where the game is still sold, buy it from the store. GameFrame never links to download sites
for games.

## Three groups of countries

Most countries follow one of three patterns.

| Group | What it allows | Examples |
|---|---|---|
| The EU model | Observing, studying and testing a program you own. Decompiling only to make an independent program work with it | The EU, the UK, South Korea, Singapore, New Zealand, Ukraine, Norway, and partly India |
| The US trade-agreement model | The same study rights, with a strict ban on breaking locks and a closed list of exceptions that includes interoperability | The US, Canada, Mexico, Colombia, Australia |
| Back-up and research only | A back-up copy, sometimes a research copy. No express permission to decompile | Brazil, Argentina, Egypt, Indonesia, Thailand, Vietnam, Malaysia, the UAE, South Africa under its current Act |

Observing a program while it runs is the widest permission. It covers running the original,
capturing frames and state, and recording calls. These are the playbook's observe and emulate
steps ([re-observe](../../../.claude/skills/re-observe/SKILL.md),
[re-emulate](../../../.claude/skills/re-emulate/SKILL.md)). Decompiling is the narrowest
permission. Many countries allow it only for interoperability, and about a third of the
countries covered give it no express cover. Treat decompiling as the step that needs the most
care.

## What this means in practice

GameFrame's defaults follow from these points. The user decides what their own project does
([The user decides](../../working/agents.md#the-user-decides)).

- **Sharing.** No country covered allows sharing game files, assets or decompiled code. Every
  decompilation exception also forbids passing the information on.
- **New code.** Singapore, New Zealand, South Korea, Turkey and the EU forbid using decompiled
  information to make a "substantially similar" program. A reimplementation that behaves like
  the original and is written in its own words stays on the safe side
  ([Writing findings](../../working/writing.md)).
- **Locks.** Laws differ most here. Japan has no known interoperability exception to its lock
  rules. Thailand removed its exceptions in 2022. Mexico makes circumvention for profit a crime.
  Australia's Federal Court held in 2024 that a game's online checks count as locks. Where a
  game's files are encrypted or protected, the regional pages say what is known for each
  country.
- **Services.** South Korea fines people who run private servers, even for a single-player game.
  Japan has arrested people who sold edited save files.

## There is no such thing as "abandonware"

A game stays under copyright when its maker stops selling it. Copyright lasts at least the
author's life plus 50 years in every country covered. No commercial game from the 1970s onward
is out of copyright anywhere. ScummVM's FAQ says the word "is not a proper legal concept in any
country" ([ScummVM FAQ](https://docs.scummvm.org/en/latest/help/faq.html)). Sharing an
out-of-print game is still unlawful.

## Hosting

GitHub applies the US takedown process (the DMCA) to every repository, wherever its owner lives
([GitHub DMCA policy](https://docs.github.com/en/site-policy/content-removal-policies/dmca-takedown-policy)).
Your own work is still judged by the law of your country. A project that holds only its own new
code and documents gives a rights holder nothing to take down.

## What takedowns have shown

- Nintendo removed builds of the Super Mario 64 PC port that included the game's assets.
- Take-Two removed the re3 and reVC projects for GTA III and Vice City in February 2021
  ([GitHub DMCA notice](https://github.com/github/dmca/blob/master/2021/02/2021-02-19-take-two.md)),
  then sued their authors. Those projects needed the player's own data. Their code came from a
  decompilation of the original programs.

Requiring the player's own files is necessary. It is not enough on its own. The code a project
publishes must be new, written from documented behaviour. The process must be written down so
that anyone can check it.

## The pages of this chapter

1. [Treaties](treaties.md): what every country shares, and what the treaties leave to each.
2. [United States, Europe and the United Kingdom](us-europe-uk.md): the US, the EU, the UK,
   Norway and Switzerland.
3. [East Asia](east-asia.md): Japan, China, South Korea, Taiwan, Hong Kong.
4. [South and Southeast Asia and Oceania](asia-pacific.md): India, Singapore, Malaysia,
   Indonesia, the Philippines, Thailand, Vietnam, Australia, New Zealand.
5. [The Americas](americas.md): Canada, Mexico, Brazil, Argentina, Chile, Colombia.
6. [Africa, the Middle East, Other Nations](africa-middle-east.md): South Africa, Nigeria,
   Kenya, Egypt, Israel, Turkey, the UAE, Saudi Arabia, Russia, Ukraine.
7. [35 countries compared](compared.md): one table.

Each country uses the same headings: studying a program, locks, libraries and archives, how long
copyright lasts, owners who cannot be found, cases, and games that shut down. "(not confirmed)"
marks a point that has not been checked against the official text.
