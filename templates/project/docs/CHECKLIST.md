# Checklist: GAME_NAME

Updated: DATE

Each line: the item, its main original function, and its state: traced (read and documented),
measured (emulated, ground truth captured), reimplemented, verified (bit-exact against the
ground truth). Keep it short; details belong in the topic documents.

## Inventory

- [ ] Every binary identified (`./gameframe run peinfo`)
- [ ] Ghidra export of the game's own binaries (`./gameframe run ghidra-export`)
- [ ] Data containers identified and listed

## Foundations (usually first: everything else depends on them)

- [ ] Random number generator: traced / measured / reimplemented / verified
- [ ] Maths helpers the simulation uses (vectors, angles, fixed point)
- [ ] Game time and the update loop (fixed step? delta time?)
- [ ] Data container reader

## Systems

- [ ]
