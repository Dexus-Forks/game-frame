# Checklist: GAME_NAME

Updated: DATE

Each line: the item, its main original function, and its state: traced (read and documented),
measured (emulated, ground truth captured), reimplemented, verified (bit-exact against the
ground truth). Keep it short; details belong in the topic documents.

## Inventory

- [ ] Every binary identified (`./gameframe run peinfo`)
- [ ] Ghidra export of the game's own binaries (`./gameframe run ghidra-export`)
- [ ] Data containers identified and listed
- [ ] The story read: the manual and the program's long strings, `docs/lore.md` written (`re-inventory`, "Read the story")

## Viewer (before the game: every format becomes a reader, a document and a panel)

- [ ] Catalogue of every file (`./gameframe run catalog target/ derived/catalog`)
- [ ] The project's viewer app, on the port's own readers
- [ ] Pictures and palettes
- [ ] Sounds and music
- [ ] Models and levels
- [ ] Text, fonts and saves

## Site

- [ ] The site and its first diary entry (`re-site`)

## Foundations (everything in the game depends on them)

- [ ] Random number generator: traced / measured / reimplemented / verified
- [ ] Maths helpers the simulation uses (vectors, angles, fixed point)
- [ ] Game time and the update loop (fixed step? delta time?)
- [ ] Data container reader

## Systems

- [ ]
