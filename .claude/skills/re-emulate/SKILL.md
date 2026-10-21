---
name: re-emulate
description: Run individual functions from a game's Windows x64 binaries under the Unicorn emulator (tools/emu/pe_emu.py) to capture exact ground truth for chosen inputs, and record the calls they make (random draws) with call_trace.py. Use for anything a reimplementation must match bit for bit - random number generators, maths, procedural generation, simulation steps, pathfinding, parsers - instead of trusting the pseudocode.
---

# Emulate original code for ground truth

Reading code tells you the shape of a computation; only running it tells you the exact result.
In a simulation, one bit of difference in a float or one random draw out of order grows until
the reimplementation's world no longer matches. The emulator runs a single original function,
many times, with inputs you choose, without starting the game.

## How the harness works

```python
import sys; sys.path.insert(0, 'tools/emu')
from pe_emu import Emu, F32
emu = Emu('target/Game.exe', ['target/Engine.dll'])  # loads, relocates, links imports by name
```

- Every module loads at its preferred base (relocated if two overlap). Imports between loaded
  modules are linked; any other import becomes a stub. Common C runtime and kernel32 calls have
  working stand-ins (`CRT_STUBS`: malloc, free, memset, memcpy, strlen, critical sections...).
  An import with no stand-in stops the run with its name: add one (`stubs={'Name': handler}`).
- `emu.call(addr, *args)` uses the Windows x64 convention: integers and pointers in rcx, rdx,
  r8, r9, floats wrapped in `F32(...)` or `F64(...)` in xmm0-3, the rest on the stack. It returns
  rax; `call_f32` returns xmm0, `call_i32` a signed int.
- Memory: `malloc(size)` (zeroed), `write(addr, bytes)`, `read(addr, n)`, `u8/u32/i32/u64`,
  `f32s(addr, n)`, `cstr(addr)`, `bytes_in(data)` (copy bytes in, get the address).
- Addresses: `export(module, name)`, or `va(module, 0x1400be770)` for a Ghidra address (it
  follows a relocated module).
- `hook_function(addr, name, handler)` replaces any function, original or imported, with Python:
  use it for callees that touch files, threads, the GPU, or that you want to observe.

## Steps

1. **Trace first** (`re-trace`): know the function's arguments, the layout of the objects it
   reads (offsets and types), the globals it needs (a singleton pointer at `DAT_...`), and its
   callees that must be stubbed.
2. **Build the inputs** in emulated memory: `malloc` each object, `struct.pack` the fields at
   their offsets, link pointers. Point globals at your objects with `write(va(...), pack('<Q', ptr))`.
   Build the smallest object graph that works; the emulator reports every invalid access with
   the address and the code position, which tells you the next field to fill.
3. **Write a generator script** (`tools/emu/<topic>.py` in the project, modelled on
   `examples/sample-game/ground_truth.py`): a docstring with usage and outputs; a fresh `Emu`
   per independent case; many inputs (every cell of a small map, seeds 1 to 50, edge cases such
   as zero, negatives and borders); raw little-endian outputs into `derived/emu/<topic>/`
   (ignored by git, because ground truth is derived from the game).
4. **Record the draws** when the function consumes the game's random number generator:
   ```python
   from call_trace import CallTrace
   trace = CallTrace(emu, {emu.va('engine.dll', 0x180012340): 'Rand',
                           emu.va('engine.dll', 0x180012380): 'RandFloat'}, args=1)
   emu.call(target, ...)
   trace.write('derived/emu/ai/seed-1.trace')   # "Rand(10)<TAB>game.exe+0x9be55" per draw
   ```
   Only outermost calls are logged, so a float draw that calls the integer draw counts once.
5. **Compare** the reimplementation's output with `re-verify` (`bitcompare.py`, `seqdiff.py`).
   Not exact? Emulate intermediate functions separately and compare stage by stage, until the
   first stage that differs is small enough to read instruction by instruction.

## Pitfalls

- **Invalid memory access**: a pointer field you did not fill. Find which offset of which object
  from the reported code position and the disassembly there.
- **Thread-local storage**: the harness maps a TEB at GS; code that uses `__declspec(thread)`
  variables also needs the TLS slot array (`TEB+0x58`) pointing at a block for the module.
- **Exceptions**: SEH and C++ exceptions do not unwind in the emulator; inputs that make the
  original throw will stop the run.
- **Long functions** (a whole map generator): pass `timeout_insns` and emulate them whole rather
  than porting blind; dump intermediate state with hooks to compare in stages.
- **DllMain is not run**: if a function depends on globals a DLL sets up at load, initialise
  them yourself, or call the module's initialiser explicitly.
- **32-bit games** (x86) need a variant of the harness: `UC_MODE_32`, arguments on the stack,
  `this` in ecx, a TEB at FS. `docs/ABI.md` lists the conventions. Linux ELF and macOS Mach-O
  binaries need a different loader; the calling convention is System V (rdi, rsi, rdx, rcx, r8,
  r9; floats in xmm0-7).
- **Floating point state**: the emulator starts with the default MXCSR (round to nearest). If the
  game changes the rounding mode or enables flush-to-zero, set it the same way.
