# Reading compiled code

A compiler turns a game's source into machine code. It reorders, merges and rewrites the
source to make it faster or smaller. A decompiler such as Ghidra reverses part of this, and
leaves the rest for you. This page lists the forms compilers produce from common source, so
that you can see the source behind them. It covers x86 from 16-bit DOS to x64. The details of
calling conventions, C++ layout and floating point are on
[Calling conventions and floating point](ABI.md). The game-level structures (the main loop,
random generators, entities) are on [Game patterns](GAME-PATTERNS.md).

## Finding the game's own code

An executable starts in the C runtime's start-up code, which then calls the program's `main`,
`WinMain` or Pascal main block. Skip past the runtime.

| Runtime | How to find the game's entry |
|---|---|
| MSVC (Windows) | The CRT start-up function calls `GetCommandLine` and `GetStartupInfo`, then `WinMain` or `main` with the module handle and the arguments. From Visual C++ 2005 on, the entry first calls `__security_init_cookie` |
| MinGW and GCC | `__main` or `__libc_start_main` receives `main`'s address as its first argument |
| Borland C and Turbo C (DOS) | The start-up code sets up the data segment and the stack, calls the `#pragma startup` list, then `main` |
| Turbo Pascal and Borland Pascal | The main block starts with calls to the System unit's initialisation, then each used unit's initialisation in order |
| Watcom (DOS/4GW) | The entry of the LE program calls `__CMain`, which calls `main` |

Ghidra's Function ID and library signatures name many runtime functions. Strings help too:
a runtime error message ("R6002", "Runtime error") sits in the runtime. The game's own strings lead to the game's own code. `tools/triage/similar.py` against a build of the same
runtime shows how much of an executable is library code.

## Optimised arithmetic

| In the machine code | In the source |
|---|---|
| `imul` by a large constant, then `sar` or `shr` and an add of the sign bit | Division by a constant. The divisor is about 2^(32+s) / constant, where s is the extra shift |
| `lea eax, [eax+eax*4]`, `shl`, add chains | Multiplication by a small constant (here 5) |
| `and eax, 7` on an unsigned value | `x % 8` |
| `cdq`, then `xor`, `sub` with `edx` | `abs(x)` |
| `cdq`, `and edx, 7`, `add`, `sar eax, 3` | Signed division by 8, rounded toward zero |
| `neg eax`, `sbb eax, eax` | `x != 0 ? -1 : 0`, often then `and` with a value to select it |
| `setcc al`, `movzx eax, al` | A comparison stored as 0 or 1 |
| `cmovcc` | A `?:` or a short `if` without a jump |
| `test eax, eax` then `jz` | `if (x == 0)` |

When the source divides by a constant, the port should divide by that constant, with the same
signedness. The multiply-and-shift form comes from the compiler. It gives the same result.
Confirm a recovered divisor with `re-emulate` on a few inputs, including negative ones.

On 16-bit code, operations on 32-bit `long` values become calls to helper routines in the
runtime, such as Borland's `LXMUL@`, `LDIV@` and `LUMOD@`, or Microsoft's `__aNlmul` and `__aNldiv`.
Name them once. Then each call reads as one operation.

## Branches and loops

| In the machine code | In the source |
|---|---|
| A compare, a conditional jump over a block | `if` |
| A jump at the end of the `if` block over an `else` block | `if` ... `else` |
| The test at the bottom of the loop, a jump to it at the top | `while` or `for`, compiled to test once at the bottom |
| A test before the loop, then the body, then the test again | A loop the compiler rotated |
| A bounds check, then `jmp [table + reg*4]` | `switch` with a jump table. The table holds one address per case |
| A tree of compares on the same value | `switch` with cases far apart, compiled as a binary search |
| A chain of conditional jumps to the same false target | `&&` |
| A chain of conditional jumps to the same true target | `\|\|` |

Ghidra recovers most jump tables. When a `switch` shows as a computed jump without cases, find
the table's address and size from the bounds check, and read its entries with
`tools/pe/readdata.py`. Each entry is one case.

## Functions

- **Inlined functions.** A small function, or one called once, can be copied into its caller.
  The same few instructions appearing in many places, with the same constants, are usually one
  inlined function. Port it as one function.
- **Inlined library calls.** `rep movsd`, `rep stosd` and unrolled moves are `memcpy` and
  `memset`. Short loops comparing bytes are `strlen`, `strcmp` and similar functions.
- **Tail calls.** A function that ends with `jmp` to another function returns that function's
  result.
- **No frame pointer.** Optimised code addresses locals from `esp` or `rsp`. The offsets change after each `push`. Ghidra tracks this. When a local looks wrong, check the stack depth
  at that instruction.
- **Shared code.** The linker can merge identical functions. One body may then serve two names. Its callers may expect different meanings.

## The flags

Each arithmetic instruction sets the flags: zero, sign, carry and overflow. The conditional
jump after it reads them. The jump names give the signedness of the comparison.

| Signed | Unsigned | Meaning |
|---|---|---|
| `jl`, `jle`, `jg`, `jge` | `jb`, `jbe`, `ja`, `jae` | Less, less or equal, greater, greater or equal |

The port must use the same signedness, whatever type Ghidra declares. On older code, an x87
comparison stores its result with `fnstsw ax`, then tests bits of `ah` (`test ah, 41h`,
`sahf`). [ABI](ABI.md) explains how to read these.

## Debug builds, release builds and symbols

A game may ship with more than one build, such as a debug build on a demo disc or a version for another platform. A debug build keeps local variables on the stack and does not reorder code,
so it is easier to read. A build with symbols (a PDB, a MAP file, Borland debug data) names
functions and types. Look for these before reading an optimised build alone. The same
function in two builds, matched with `similar.py` and its strings, gives the readable version
of the optimised one.

## What to record

- The compiler and runtime, with the evidence (`peinfo.py`, `dosinfo.py`, runtime strings).
- The address of the game's entry past the runtime.
- Each runtime helper you named, in the labels file, so the export reads cleanly.
- Each recovered constant (a divisor, a jump table) with how you confirmed it.
