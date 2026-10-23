# The sample game

A tiny Windows x64 game, written in C for GameFrame, to practise on and to test the tools.
It has the parts a real game has: an engine DLL with a random number generator and an
allocator, a fixed-step simulation, a constant table, an object with a table of functions (a
vtable), and a data file loader.

```bash
./build.sh                                   # build/game.exe, build/engine.dll, build/mixer32.dll, build/sample.dat (needs zig)
../../.venv/bin/python ground_truth.py       # emulate the original: build/truth/seed-N.f32 and .trace
../../.venv/bin/python port.py               # the reimplementation: build/port/
../../.venv/bin/python ../../tools/verify/bitcompare.py f32 build/truth/seed-1.f32 build/port/seed-1.f32
../../.venv/bin/python port.py --naive build/naive   # doubles instead of floats: watch the checks fail
```

![bitcompare.py and seqdiff.py on this sample. The port matches the original in 600 of 600 values, and its 746 random draws are identical. The naive port matches only 7 values.](../../docs/assets/captures/bitcompare.svg)

Try the whole method on it as an exercise: run `scripts/ghidra-export.sh` on the two binaries,
find `simulate` without looking at `game.c`, label what you find, read the growth table with
`tools/pe/readdata.py`, and write your own port. The compiler inlined `tree_new` and the vtable
calls into `simulate`, and the data loader compares its magic as one integer: both are lessons
from real games.

| File | What |
|---|---|
| `engine.c`, `game.c` | The original's source (in a real project you never have this) |
| `build.sh`, `make_data.py` | Build for Windows x64 from any host with `zig cc` |
| `ground_truth.py` | Runs `simulate` under the emulator for 20 seeds, with every random draw traced |
| `port.py` | A reimplementation that matches bit for bit; `--naive` shows a common mistake |
| `decode_sample.py` | A decoder for the data file, for `tools/formats/validate.py` |
| `mixer32.c` | A 32-bit (x86) DLL: a stdcall mixer step and a function that calls Windows, for `tools/emu/pe32_emu.py` |
