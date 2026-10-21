# Your own copy and the law

This chapter summarises the law as it stands in October 2026. It is not legal advice. Laws
differ between countries. If a project matters, ask a lawyer who knows software law where you
live. For the rules every project follows, read [Legal](../LEGAL.md).

## Start from a copy you bought

Every project made with the harness starts from a copy of the game that the person owns. Where
the game is still sold, buy it from the store. Where it is not, use the copy you already have.
The harness never links to download sites for games.

## United States

- **Interoperability.** Section 1201(f) of the US Copyright Act allows a lawful user to analyse
  a program to make an independently created program work with it
  ([17 U.S.C. 1201](https://www.law.cornell.edu/uscode/text/17/1201)).
- **Case law.** In *Sega v. Accolade* (1992), the court held that disassembling a game to learn
  how it worked with the console was fair use
  ([Copyright Office summary](https://www.copyright.gov/fair-use/summaries/segaenters-accolade-9thcir1992.pdf)).
  In *Sony v. Connectix* (2000), it held the same for copies made while building an emulator
  ([Copyright Office summary](https://www.copyright.gov/fair-use/summaries/sony-connectix-9thcir2000.pdf)).
- **Preservation exemptions.** The current exemptions run from October 2024 to October 2027.
  A person may restore a game whose servers have been shut for at least six months, for
  "personal, local gameplay". Libraries, archives and museums may keep games playable on their
  own premises ([37 CFR 201.40](https://www.ecfr.gov/current/title-37/chapter-II/subchapter-A/part-201/section-201.40)).
- **Remote access was refused.** In October 2024 the Copyright Office refused to let libraries
  offer out-of-print games remotely for research
  ([VGHF statement](https://gamehistory.org/dmca-2024-statement/)). It said the proposed
  safeguards were too weak. The next rulemaking opened in June 2026
  ([Copyright Office](https://www.copyright.gov/1201/2027/)).

## European Union

- **Study and test.** Article 5(3) of the Software Directive lets a lawful user observe, study
  and test a program while running it, to find the ideas behind it
  ([Directive 2009/24/EC](https://eur-lex.europa.eu/eli/dir/2009/24/oj)).
- **Decompilation.** Article 6 allows decompilation where it is indispensable for
  interoperability with an independently created program. Article 6(2)(c) forbids using what
  you learn to make a program "substantially similar in its expression". That rule is one
  reason every project here writes new code.
- **Formats and functions.** In *SAS Institute v World Programming* (2012), the Court of Justice
  held that a program's functionality, its programming language and its data file formats are
  not protected as expression.
- **Games that end.** The Stop Destroying Videogames citizens' initiative gathered 1,294,188
  verified signatures. In June 2026 the European Commission chose a voluntary industry code of
  conduct over new law
  ([Lewis Silkin](https://www.lewissilkin.com/insights/2026/06/24/european-commission-stops-short-of-imposing-mandatory-requirements-following-citi-102n4yy)).

## United Kingdom

- Section 50B of the Copyright, Designs and Patents Act 1988 allows a lawful user to decompile a
  program to create an independent program that works with it
  ([legislation.gov.uk](https://www.legislation.gov.uk/ukpga/1988/48/section/50B)).
- Section 50BA allows a lawful user to observe, study and test a program while running it
  ([legislation.gov.uk](https://www.legislation.gov.uk/ukpga/1988/48/section/50BA)).
- A petition to protect games from being shut down reached 189,887 signatures. The government
  replied that it has "no plans to amend UK consumer law"
  ([UK Parliament](https://petition.parliament.uk/petitions/702074)). Parliament debated it on
  3 November 2025.

## There is no such thing as "abandonware"

A game stays under copyright when its maker stops selling it. Copyright does not end because a
company closed or a game went out of print. ScummVM's FAQ says the word "is not a proper legal
concept in any country" ([ScummVM FAQ](https://docs.scummvm.org/en/latest/help/faq.html)).
Sharing an out-of-print game is still unlawful.

## What takedowns have shown

- Nintendo removed builds of the Super Mario 64 PC port that included the game's assets.
- Take-Two removed the re3 and reVC projects for GTA III and Vice City in February 2021
  ([GitHub DMCA notice](https://github.com/github/dmca/blob/master/2021/02/2021-02-19-take-two.md)),
  then sued their authors. Those projects needed the player's own data. Their code came from a
  decompilation of the original programs.

Requiring the player's own files is necessary. It is not enough on its own. The code a project
publishes must be new, written from documented behaviour. The process must be written down so
that anyone can check it. This harness is built around both.
