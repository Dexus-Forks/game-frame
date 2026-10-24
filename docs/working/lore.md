# Reading the story for design decisions

A game's story, its manual and the text inside its program are evidence about its design. They
often say what a system is for before the code does. When you know what a system is for, you
know where to look for it, which of its values matter, and how exact the port must be. The
worked example is Hardwar (1998).

## Why the lore is evidence

Designers build systems to serve the game they describe to players. The manual tells players
what to care about: what runs out, what costs money, what kills them, what they are trying to
reach. Those things are the systems the designers cared about. A reverse engineer who reads the
code first sees thousands of functions of equal weight. A reverse engineer who reads the story
first knows which few of them the game is about.

The lore helps in three ways:

- **It names a system before you find it.** A manual that says "craft recharge under the
  lights" tells you that light is game logic. You then look for a test of position against a
  light, and you find it sooner.
- **It explains a value you have already found.** A constant that scales the lights with time
  is a puzzle until the story says the craft run on daylight.
- **It says what must be exact.** A rule the player can see and plan around (a price, a
  deadline, a ceiling) must match the original. A visual detail the story makes part of the
  setting (fog on a foggy moon) must match too. A choice the story does not care about may be
  a place where a port can differ.

Most of this evidence is *observed* in a document or a string, and the link to the code is
*inferred* until it is traced. Keep the two labels apart. The lore tells you where to look. The
code and the emulator still decide what the game does.

## Hardwar as the example

Hardwar is a flight and trading game set in a mining colony on Titan, cut off from space for
two centuries. Two gangs that were once corporations hold most of the city. The player flies a
small solar-powered craft called a Moth. The Hardwar project read the manual on the player's
disc and the long strings in the program before it traced most systems. The story explained
several things the code had shown without a reason, and it named several systems before they
were found.

| In the story | In the code | Evidence |
|---|---|---|
| Moths run on solar power cells. Ordinary daylight cannot fill them. Pilots recharge under lightwells, towers that beam light down | The city's eight crater lights are scaled by a factor of a time input that never reaches zero. The world clock keeps a time of day. AI pilots look for a lightwell when their power is low | Clock and lights *observed* in code; the strings *observed*; that the time input is the time of day is *inferred* |
| Titan is a freezing, foggy moon | The fog tables in the game's remap file, and a fog distance setting | Tables *observed*; that the fog is part of the look is the setting's claim |
| Two factions control the water purifiers and the food domes | The descriptions of the goods state a production chain: ore to sheet metal, water into power cells, chemicals from the polluted air | *Observed* in the program's strings; the economy's code is not yet traced |
| Leaving Titan is every character's goal. Solar craft cannot climb high enough | The altitude ceiling depends on the fitted power cell. A fusion cell raises it | *Observed* in an equipment description; the flight model is not yet traced |
| A crime report clears after a day. A character plans to meet at a set hour the next night. Loans gather interest | The world clock has days and hours, and rules run on it | Clock *observed*; the timed rules *observed* as text |
| The plot seals a crater and a tunnel, renames a building and changes food production | The world has tunnel blocks and per-crater zones the program can close | *Observed* as strings; the mechanism is *inferred* |
| The aliens take only a pilot who carries no guns | The ending checks the player's fitted weapons | *Observed* in a developer's note left in the program |

Each row changed the plan:

- **Light is a resource.** Lighting is part of the rules as well as the drawing. The time input
  of the crater lights, the lightwells and the recharge test belong to the exact core. The
  renderer only reads them.
- **The fog is part of the look.** It is the setting, so the fog tables and the fog distance
  must match recorded frames. A port that pushes the fog away as an improvement does it as a
  marked addition behind an option.
- **The economy is the core.** The story is a stalemate over water and food, and the economy
  is where that stalemate lives. Production, needs and prices must match the original bit for
  bit. The goods descriptions are a specification to check the traced code against.
- **The ceiling is a rule.** The height limit is tied to the power cell. It is part of the
  flight model's exact rules.
- **The clock is world state.** Rules and story events are scheduled on it, and the save must
  hold it. The order of timed events matters.
- **The save holds the plot.** The plot edits the world: sealed tunnels, a renamed building,
  changed production. Each change is state the save must hold and a replay must reach.
- **The ending is a rule.** It tests the player's fittings, so the fitting data and that test
  are part of the exact core.

## The program's own text

The manual is written for players. The program's long strings were written for the game. In
Hardwar about 200 strings are whole sentences: radio bulletins, faction statements, police
notices, e-mails from characters and descriptions of goods. Read in address order, they tell
the whole plot. The goods descriptions state the production chain. The equipment descriptions
state the ceiling. A note left by a developer states the last step of the ending.

These strings are rules in plain language. When the economy is traced, its production table
should match what the descriptions say. Where it differs, one of the two is wrong, and both
are worth a second look. List the long strings early with
`./gameframe run sentences GAME.EXE --min-words 6 > analysis/notes/sentences.txt`, and read
them before the code.

Other sources help the same way: the README, the setup program's options, cheat sheets, the
box, a strategy guide, interviews with the developers and a wiki written by players. Record
each source, and label what came from players' accounts as unverified.

## The step in the method

Read the story early, during the inventory (`re-inventory`, "Read the story"):

1. Find the manual on the disc or in the store's extras (often a PDF), the README and any
   other text the publisher shipped.
2. List the program's long strings with `sentences` and keep the list in `analysis/` (it is
   the game's text).
3. Write `docs/lore.md` from the template: a short summary of the setting in your own words,
   the rules the text states, and the table "in the story → in the code → evidence".
4. Mark each row's code side as *not yet traced* until it is. Fill it in as the systems are
   traced.
5. When a system's purpose is unclear, or a constant has no reason, read `docs/lore.md` again
   before guessing.
6. Copy what the lore says must be exact into the project's `AGENTS.md`, under "Rules for this
   kind of game".

Write the summary in your own words. Never copy the manual's text or the game's strings into a
tracked file. Name a string by its address and say what it states.

## Pitfalls

- **The manual describes the design as planned.** Features are cut late and manuals are
  printed early. A rule in the manual is a lead until the code shows it.
- **Sources disagree.** Hardwar's manual and its strings give two different lengths for the
  colony's isolation. Record both, with their sources.
- **Later releases change the rules.** Patches and re-releases change values and sometimes
  rules. Note which release a player's account describes.
- **Names can be jokes.** Hardwar names places after buildings near the developer's office and
  wanted pilots after characters in a film. A name in the data is not always a clue to its
  role.
