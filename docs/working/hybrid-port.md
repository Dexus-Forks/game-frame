# Replacing the original one routine at a time

Some ports run the original program inside themselves, on an emulated CPU, and replace its
routines with native code one at a time. The game is playable from the first day, because the
original code runs everything that is not ported yet. Every test can call the original as an
oracle. This page describes how to build that kind of port. It applies best to small programs
that an in-process emulator can run at full speed: DOS games, 8-bit and 16-bit computer games,
and console games. [The Frontier case study](../case-studies/frontier.md) is a complete example.
[Building the port](building-the-port.md) covers ports that do not keep the original.

## The parts

| Part | What it does |
|---|---|
| The runtime | An emulated CPU with the services the program calls (for DOS: DOS, BIOS, EMS, the mouse, the video and sound hardware). It loads the player's copy of the game and runs it from the start |
| The ports | Native functions, each a copy of one original routine |
| The hooks | A table from original addresses to ports. Before it runs an instruction, the CPU checks the address in a bitmap. On a hit it calls the port instead |
| The oracle | A way for tests to set up a state, call one original routine and stop when it returns |

## The port contract

Give every port the same signature: it takes the CPU and the memory, and returns a number. For
the CPU above, `fn(&mut Cpu, &mut dyn Bus) -> u64` serves.

- **The port leaves the machine as the original would.** That means every register, including
  scratch registers no caller reads, every byte the original writes, the flags that callers test,
  the stack pointer and the return address. A register the port leaves different is found by
  the next comparison, and it hides real differences after it.
- **The port returns the original's instruction count.** It counts the instructions the
  original would have run on the same path. The runtime charges the count to its clock. Timer
  interrupts, music and frame timing then stay where the original put them. A port that returns
  1 for a routine of 2,000 instructions makes the game run faster and moves every interrupt.
- **The port can decline.** A reserved value (for example `u64::MAX`) means "run the original
  instead", with the machine unchanged. A port declines the inputs it does not cover yet: an
  unported branch, a table the game has not set up yet, a recursion deeper than a fixed limit.
  Partial ports can then go into the game at once.
- **The port may call original code.** When a ported routine calls a routine that is not
  ported, the port can set up the call as the original would and run the original until it
  returns. A port that ends by jumping into original code (a tail call) runs the original until
  the CPU reaches the port's own return address and stack pointer.

Count instructions with the code open beside the port. Add the count for each straight run of
instructions, and add one for each branch taken. Comment each count with the instructions it
covers. The comparison tests find the miscounts.

## Interrupts

The original's clock is its instruction count. The runtime delivers a pending interrupt
between instructions. While a port runs, the runtime cannot deliver one, so it delivers it when
the port returns. Most game code tolerates that delay. Code that waits for an interrupt in a loop
(a busy wait on a tick counter) must stay original, or the port must run its loop through
the runtime one step at a time.

## Proving each port

| Check | How | What it finds |
|---|---|---|
| Fuzzed calls | A test sets random registers and memory within the routine's real input domain. It calls the port on one copy of the machine and the original on another. It compares registers, flags, every written byte and the instruction count | Wrong formulas, missed branches, miscounted instructions |
| Shadow verification | In the running game, the hook runs the port on a copy of the machine, then the original on the real one, and logs any difference. The game keeps the original's result | Inputs the fuzzer never makes |
| In-game tests | A test plays the game to a known point (a menu, the cockpit), then calls routines through the game's own entry points with real data | Routines whose inputs are hard to make up: object lists, loaded overlays |
| Long runs | The game runs with every port on and with every port off, for a fixed time with fixed input. The frames and the sound must be identical | Ports that are correct alone but wrong in combination |

Record which routines declined, and how often, during a real session. The declines with the
most calls show which branch or helper to port next.

## Choosing what to port

- Start at the bottom: the maths library, the random number generator, string formatting.
  Many routines call these, so each port is checked by every later test.
- Port generators early in games that make their worlds from seeds. A galaxy, a map or a
  name generator is pure: same input, same output. Fuzzing proves it completely. See
  [Game patterns](../GAME-PATTERNS.md), "Procedural generation".
- Use run-time coverage to find the routines a feature runs: record coverage with and without
  the feature, and compare ([Debuggers](../observe/debuggers.md)).
- `tools/triage/coverage.py --module NAME` shows how much of one overlay or module the port
  cites. It reads citations of the form `NAME:1cf5`, for programs where each module is numbered
  from the same base address.

## Pitfalls

- **Code that is paged in.** Overlays, banks and EMS pages move. A hook must check that the
  expected module is loaded at the address before it runs a port. A test that calls a routine
  in an overlay must first load that overlay through the program's own loader.
- **Host recursion and loops.** A port that follows a chain in game memory (a parent link, a
  string that includes another string) can loop forever or overflow the host stack on bad
  data. The original might only crash the emulated machine. Bound every such loop and decline
  past the bound.
- **State shared between tests.** Tests run in parallel in one process. Keep per-call state
  (such as a recursion depth) in thread-local storage, not in a global.
- **Saving the machine around a test call.** A test that calls game code in the middle of
  play must save the registers and restore them before the game continues.
