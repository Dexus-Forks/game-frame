# DOS and 16-bit games

Most of the harness was built on a 64-bit Windows game. DOS games (1985-2000) need different
tools at three steps: reading the executable, emulating code, and running the original. This
page is the whole DOS route; the skills link here. The tools are tested against
`examples/dos-sample/` (our own 16-bit code) by `tests/run.sh`.

## 1. Which kind of DOS program is it?

```bash
python3 tools/dos/dosinfo.py target/GAME.EXE              # format, segments, imports, fingerprints
python3 tools/dos/dosinfo.py target/GAME.EXE --pstrings 6 # Pascal strings, with their addresses
```

| dosinfo says | What it is | Ghidra | Emulate one function | Run it all |
|---|---|---|---|---|
| `MZ` | Real-mode DOS (Turbo Pascal, Turbo C, Microsoft C, QuickBASIC) | "Old-style DOS Executable", loaded at segment `1000` | `dos_emu.py` (real mode) | DOSBox-X |
| `NE` | 16-bit protected mode: Windows 3.x, OS/2 1.x, Borland Pascal 7 or Borland C++ with DPMI16 (`RTM.EXE`, `DPMI16BI.OVL`) | "New Executable", selectors `1000`, `1008`, ... | `dos_emu.py` (protected mode, same selectors) | DOSBox-X (or Windows 3.1 inside it) |
| `LE` / `LX` | 32-bit flat code behind a DOS extender (Watcom + DOS/4GW, PMODE/W) | needs an LE/LX loader extension | not yet in this harness | DOSBox-X |
| `PE` | A Windows program | use `tools/pe/peinfo.py` | `pe_emu.py` | Windows or Wine |

Packed executables (LZEXE, PKLITE, EXEPACK, DIET: dosinfo's fingerprints) must be unpacked
first, for example with UNP or by dumping memory from DOSBox-X after the unpacker has run.

Data appended after the MZ load image is an overlay (Borland `.OVR` data starts with `FBOV`),
a packed payload, or debug symbols.

## 2. Addresses: always Ghidra's numbering

Write every address as `seg:off` the way Ghidra prints it, in docs, labels and tests:

- **NE**: segment N of the file has selector `0x1000 + 8 * (N - 1)`. Segment 1 is `1000:`,
  segment 10 is `1048:`. Imports go after the last segment.
- **MZ**: the load image starts at segment `1000`, so `1000:0000` is the first byte after
  the header; a second segment in the image is `1000 + paragraphs`.

`dosinfo.py` prints Ghidra's selector for each segment. `readseg.py`, `dos_emu.py`,
`tdinfo.py` and `decomp_query.py` all accept and print these addresses. The *running* program
uses other numbers: real-mode segments where DOS loaded it, or the DPMI host's selectors.
Only the DOSBox-X debugger shows those (section 7).

## 3. Debug symbols: check before anything else

Borland compilers wrote Turbo Debugger symbols into the program (TPC/BPC `/V`, TLINK `/v`),
and many shipped games kept them. They give the original names of every procedure and global,
the parameters and locals of each procedure, record layouts, and code-to-source-line maps.

```bash
python3 tools/dos/tdinfo.py target/GAME.EXE summary          # units, counts
python3 tools/dos/tdinfo.py target/GAME.EXE procs makeenemy  # address, size, parameters, locals
python3 tools/dos/tdinfo.py target/GAME.EXE types enemy      # record fields with offsets
python3 tools/dos/tdinfo.py target/GAME.EXE labels > analysis/td-labels.tsv
scripts/ghidra-export.sh --labels analysis/td-labels.tsv target/GAME.EXE
```

With `--labels`, Ghidra names every function and global, and creates the functions it missed.

The record types become reimplementation code directly: `tdinfo.py BINARY rust TYPE ...` writes
Rust structs with readers in the original's field order, each field commented with its Pascal
type and offset, and `tdinfo.py BINARY rust-reader` the `PascalReader` they use (little-endian
integers, one-byte Boolean, 6-byte Real kept exact, `string[N]`). Generated names that clash
with Rust's prelude get `Rec` (`Optiontype` -> `OptionRec`). Records also get `write`, the
inverse of `read`.

`tdinfo.py BINARY rust-globals` writes one `Globals` struct holding every variable of the data
segment at its original offset, with `load` and `store` to and from a 64 KB image of the
segment (Booleans stay `u8`, so every byte survives). Ported code then works on `Globals`, and a
test can load the DS image from before an emulated run of the original, run the port, store,
and compare with the DS image from after: every variable checked at once.

The runtime library (`System`, `Crt`, `Dos`) has no symbols of its own. Borland Pascal 7's
random generator, for example, is `RandSeed := RandSeed * $08088405 + 1` (a near `NextRand`
built from 16-bit multiplies and shifts), `Random(n)` = (new seed * n) >> 32 using all 32
bits, and `Randomize` seeds from INT 21h AH=2Ch (CX low word, DX high word). Find it by the
constant `0x8405` and the global `RandSeed`, and measure it before trusting this.
Record field offsets are summed from field sizes: tdinfo says when a record's fields do not add
up to its size (a variant record), so check one offset in the code before relying on it.

## 4. Decompiling 16-bit code with Ghidra

`scripts/ghidra-export.sh` works for MZ and NE as it does for PE. Things that differ:

- **Library search crashes the import.** An NE program that imports a module (`RTM`) whose
  file sits next to it (`RTM.EXE`, a plain DOS program) made Ghidra's loader throw "Invalid
  index". The export script passes `-loader-libraryLoadDepth 0`, so Ghidra never searches.
- **No-return calls.** Ghidra does not know that `INT 21h` with `AH=4Ch` (exit) or Pascal's
  `Halt`/`RunError` never return: the next procedure's code is glued onto the end of the
  caller. Split it by creating the function at the right address (labels do this).
- **Missed functions.** Procedures reached only through relocations or tables are not found.
  Debug-symbol labels fix this; otherwise use `scripts/ghidra-run.sh BINARY decompile`.
- **Pascal calling convention.** Arguments are pushed left to right and the callee removes
  them (`retf N`). Ghidra shows these as `__stdcall16far` with `param_1` being the *last*
  Pascal argument. Read parameters from `[bp+6]` upwards: the first at the highest offset.
- **Strings.** Pascal strings (a length byte, then the text) are not found by Ghidra's string
  search. Use `dosinfo.py --pstrings` and `readseg.py BINARY pstr ADDR`.
- **Segment arithmetic.** `DAT_1048_12de` is offset `0x12de` in the data segment; a far
  pointer is offset then segment in memory (`readseg.py ... ptr`).
- **String constants sit just before their procedure.** Borland Pascal emits a procedure's
  string literals right before its first instruction, so the procedure that starts after a
  string usually uses it. Code refers to a Pascal string by its *length byte*: search for the
  address one before the text that `dosinfo.py --pstrings` prints.
- **Constants that look like globals.** A 16-bit constant equal to an offset in DS shows up in
  pseudocode as `&someGlobal` (a `BlockRead` count of 0x138a printed as `&tempdat3`). Check
  the instruction (`tools/dos/nedis.py`) before believing a pointer.
- **Floating point by interrupt.** Borland's FPU emulator replaces each x87 instruction with
  `INT 34h`-`3Dh` followed by the instruction's bytes; disassemblers show `int 0x3c` and
  nonsense after it. Read those runs as floating-point code (Real, Double), not as interrupts.
- **Calls through placeholders.** Disassembling the file directly shows every far call as
  `lcall 0:ffff`: the relocation fills it in. `tools/dos/nedis.py BINARY SEG:OFF LEN` applies
  the relocations and prints label names for call targets and DS globals.
- **Text files end at ^Z.** On a Turbo Pascal `Text` file, `Eof` is true when the next byte is
  0x1A, not only at the real end, and `Read` at the end returns 0x1A. A game that stores
  binary data in a Text file (demo recordings, scripts) stops early at a 0x1A byte in the
  right place: model it, and check which positions `Eof` can see.
- **Finding the game clock.** Look for writes to ports 43h/40h (the PIT: `mov al, 36h; out
  43h, al`, then the divisor's two bytes to 40h) and for `SetIntVec(8, ...)`: the new IRQ 0
  handler and the divisor give the tick rate (1,193,182 / divisor Hz), and the loop that
  waits for a counter the handler decrements gives ticks per frame. A handler that also
  calls the old vector when a running sum carries keeps the BIOS clock at 18.2 Hz.
- **Turbo Pascal types**: `Real` is 6 bytes and not IEEE (`readseg.py ... real48`), `Comp` is a
  64-bit integer kept in the FPU, `string[N]` takes N+1 bytes, sets are bit arrays of up to 32
  bytes, `Boolean` is one byte. Records are not padded.

## 5. Emulating one function: dos_emu.py

```python
import sys; sys.path.insert(0, 'tools/emu')
from dos_emu import DosEmu, Long, FarPtr
emu = DosEmu('target/GAME.EXE')               # NE: protected mode with Ghidra's selectors
emu.call('1000:873a', 5, Long(100000))         # Pascal order; returns AX (call_long: DX:AX)
emu.u16(FarPtr(0x1048, 0x12de))                # read a global after the call
emu.trace({'1040:0a12': 'Random'}, args=1)     # record calls (the random draws) with call sites
```

- Relocations are applied: selector, far-pointer and offset fixups to the program's own
  segments, chained or additive. Imports become stubs.
- DS and SS are the auto data segment, as in a Pascal program. Pass `regs={'ds': ...}` for
  code that expects something else.
- An import or interrupt without a handler stops the run with its name and the caller's
  address. Add `stubs={'KERNEL.3': fn}` or `interrupts={0x21: fn}`; a handler reads and sets
  registers (`emu.reg('ax')`, `emu.set(ax=...)`), and an import handler returns the number
  of argument bytes to remove.
- Not emulated: DOS itself, DPMI services, timers, VGA, sound, the x87 (Borland's FPU
  emulator fixups are left alone). Code that needs them runs in DOSBox-X instead.

## 6. Running the original: DOSBox-X

A store release usually ships its own DOSBox configuration (GOG: `__support/app/*.conf`). Start
from it so runs match what the publisher tuned: `scripts/dosbox-run.sh --conf THAT.conf ...`
(the script turns fullscreen off and keeps its own logging and autoexec).

`scripts/setup.sh --install` installs DOSBox-X (on macOS `brew install --formula dosbox-x`; the
cask fails Gatekeeper). Homebrew's build has the heavy debugger (CPU logs, memory breakpoints);
`brew install --HEAD --formula dosbox-x` adds the newest debugger commands (`STEPIN`,
`SAVESTATE`).

```bash
scripts/dosbox-run.sh target/game GAME.EXE                   # no window; prints the exit code
scripts/dosbox-run.sh --fileio --time 20 target/game GAME.EXE
python3 tools/dos/fileio_trace.py analysis/dosbox/dosbox.log --game target/game
```

- The game runs on a fresh copy of its folder (`analysis/dosbox/drive`), so saves and settings
  it writes never change your copy. `--in-place` mounts the folder itself.
- `--fileio` logs every open, read, seek and write. `fileio_trace.py` turns that into a map per
  file: offsets and sizes in reading order, with repeated patterns collapsed
  (`[2, 1, 4] x 120 (stride 7)` is a table of 120 seven-byte records). The read sizes of a
  file read field by field are its record layout, *measured*.
- A program that reads through a Pascal **text** file (Assign/Reset on a `Text` variable) is
  read in 128-byte blocks: the log shows the buffer, not the fields. Read the code instead.
- `--cycles N` fixes the CPU speed. Use the same value for every run you compare.
- Exit codes come back through `IF ERRORLEVEL` (DOSBox-X's shell does not expand
  `%ERRORLEVEL%`, and a redirect inside a false `IF` still creates the file).
- Leave windowless runs to `-silent` with SDL's dummy drivers: no window, no sound.

## 7. The debugger, from an agent or a script

DOSBox-X can connect its debugger to an external controller (setting `mcp_server=PORT`, the
`--mcp PORT` option of dosbox-run.sh). Two controllers:

- **For agents**: `dosbox-x-mcp-server` (https://github.com/caiiiycuk/dosbox-x-mcp-server),
  registered in `.mcp.json` as `dosbox`. Install it with
  `cargo install --git https://github.com/caiiiycuk/dosbox-x-mcp-server --tag v0.1.2 --locked`.
  It listens on 58991; ask it with its `dosbox_mcp_port` tool, then start DOSBox-X with
  `--mcp 58991`. Tools: `debug_break`, `debug_exec` (any debugger command), `debug_snapshot`,
  `debug_run`. It has no licence file: use it, do not copy its code.
- **For scripts and tests**: `tools/dos/dbx_control.py` speaks the same line protocol
  (`REQ id EXEC command` -> `BEGIN id OK`, output, `END id`). Use another port than the MCP
  server's.

Useful debugger commands (type `HELP` for the build's own list): `BP seg:off` (breakpoint),
`BPINT 21 3D` (break on an interrupt and AH), `BPM seg:off` (break when memory changes),
`D seg:off` (dump memory), `MEMDUMPBIN seg:off len` (to a file), `SR reg value` (set a
register), `LDT`/`GDT` (selectors), `LOGS`/`LOGL` (CPU trace to a file), `STEPIN`, `RUN`.

**Finding a selector at run time.** A protected-mode program's selectors are chosen by the DPMI
host. To map Ghidra's `1000` to the live one: break anywhere in the program, read `CS` (the
segment being executed), list `LDT`, and dump the first bytes at each code selector; compare
them with the file's segment bytes (`readseg.py BINARY hex 1000:0000 --count 2`). Bytes at
relocations differ; everything else matches.
