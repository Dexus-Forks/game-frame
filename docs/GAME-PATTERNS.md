# Patterns you will meet in game code

Most games are built from the same handful of parts. Recognising the part tells you what to
look for next and which details will matter for an exact reimplementation.

## The main loop

`WinMain` → engine set-up → a loop that, each frame, reads input, updates the game, draws, and
presents. Find it from the entry point, or from `PeekMessage`/`GetMessage` (the Windows message
pump sits inside it) and the present call (`IDirect3DDevice9::Present`, `IDXGISwapChain::Present`,
`SwapBuffers`).

- **Fixed step:** the update runs at a constant rate (often 30 or 60 per second) and an
  accumulator decides how many steps a frame runs. Game speed settings usually multiply the
  number of steps, not the step size. Reimplement the accumulator exactly: the number of steps
  per frame is behaviour.
- **Variable step:** the update takes the frame time (`dt`). Exact replay is then impossible
  without recording `dt`; tests feed a fixed `dt`.
- **Loop objects:** engines often have a stack or state machine of loops (menu, loading, game),
  each with load, start, update, draw, input and stop methods in a vtable.

The order of systems inside one update is observable (who sees whose changes this step): record
it early, in a table, from the update function's callees.

## Random number generators

Usually small, self-contained and the first thing to emulate:

| Kind | How to recognise it |
|---|---|
| LCG (MSVC `rand`) | `state = state * 214013 + 2531011; return (state >> 16) & 0x7fff` |
| LCG (glibc, Java, numerical recipes) | multipliers 1103515245, 0x5DEECE66D, 1664525 |
| xorshift | `x ^= x << 13; x ^= x >> 17; x ^= x << 5` and variants |
| Mersenne Twister | a 624-word state, the constant 0x9908B0DF, tempering masks 0x9D2C5680, 0xEFC60000 |
| WELL, PCG, SplitMix | constants such as 0x5851F42D4C957F2D (PCG), 0x9E3779B97F4A7C15 (SplitMix) |

Games often have several generators (one for the simulation, one for effects or sound) and
may seed them from the clock. Find every instance and who draws from each: a draw from the wrong
stream is as bad as a draw out of order. Helpers built on the generator (a float in [0,1), a
range, a "tolerance" around a value) each have their own rounding; emulate them too.

## Entities and components

- **Class hierarchies:** `Entity` → `Unit` → `Soldier`, with behaviour in virtual methods.
  Recover the hierarchy from vtables (shared early slots) and RTTI.
- **Entity-component systems:** an entity is an id or a small object holding a list of
  components; each component type has a type id (often a hash of its name) and a vtable with
  lifecycle slots (create, attach, update, serialize, destroy). Systems iterate components of
  one type in a fixed order: that order is behaviour.
- **Handles:** an index plus a generation counter, packed into 32 or 64 bits.

## Resources and data

- **Packages:** an archive (header, entry table, data), possibly compressed per entry. Entry
  names may be stored as hashes; the hash function is in the lookup code.
- **Type ids by hash:** a CRC32 or FNV-1a of the class name, sometimes of the name in UTF-16.
  Compute candidates from strings in the binary and match them against the constants in the
  registration table.
- **Serializers:** a `Serialize(Stream&)` or `Load(Reader&)` per class, base class first, with
  version checks. The same function often both reads and writes, depending on a flag in the
  stream.
- **Scripts and tables:** rules may live in data (XML, JSON, Lua, custom binary tables) rather
  than code. Read the data before reimplementing a rule by hand.

## Time

Games keep time as integer ticks (milliseconds or microseconds) or as float seconds, and the
calendar of a simulation (days, seasons, years) is derived from it. Float time accumulates
error: if the original does that, the reimplementation must too.

Use the game's own calendar and its own names. A game's year may have twelve "months" that are
seasons (Early Spring to Late Winter), or eight, or none; its tenth month is not October. Read
the names from the game's string tables, and write docs, tests and examples in them.

## Messages and text

Event logs, notifications and tooltips usually store a message type and its arguments (a
person, a building, a number), and format the text from a string table when it is shown. Saves
may keep only the formatted text. A reimplementation that keeps the arguments formats the text
the same way; one that loads a save shows the stored text as it is. Text in the player's
language comes from data, so never hard-code an English string where the original reads a
table.

## Saves

A save is the serialized world: often the same serializers as the data files, plus the random
generator states, the clock, and object references written as indices or ids. Pointers between
objects need a fix-up pass on load. Saves are the best test of a reimplementation: loading one
exercises every system's state.

## Rendering and UI

- Shaders ship compiled (DXBC for Direct3D 9-11, DXIL for 12, SPIR-V for Vulkan): disassemble
  them (fxc's `/dumpbin`, `dxc -dumpbin`, `spirv-dis`) to read the maths, and check constant
  buffer layouts against the CPU code that fills them.
- UI is often data-driven (layouts in data files, a widget tree, events by id), with screens as
  loop objects.
- Frame capture tools (RenderDoc, PIX, apitrace) show the draw calls, their states and the
  constant values of a real frame: the fastest way to understand a rendering pass.

## Networking (if any)

Messages are usually serialized with the same stream classes as saves. Lockstep games send only
inputs and depend on exact determinism; client-server games send state. Never test against a
live service without its owner's permission.

If you add lockstep multiplayer to a single-player game, look for state that belongs to one
player but is read by the simulation: settings, unlocked achievements, difficulty options, the
local clock, the window size. Two players with different values then compute different worlds.
Give every player the same values for a shared game (for example, count achievements from none
in multiplayer), and check with digests that two players stay identical.
