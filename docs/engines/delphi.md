# Delphi programs: forms, class tables and the runtime

Borland's Delphi (1995) compiled Object Pascal with the VCL, a library of window classes. Small
studios wrote some games of the late 1990s with it, such as Holiday Island. Delphi 1 made 16-bit
Windows 3.x programs (NE). Delphi 2 and later made 32-bit Windows programs (PE). A Delphi game
has no engine. It is a set of forms (windows) whose event handlers hold the game's code, over
the VCL and the Pascal runtime.

This page is about Delphi 1 first. The facts marked *observed* and *measured* come from one
Delphi 1 program, Holiday Island (1997), where `tools/dos/delphi1.py` and `tools/formats/dfm.py`
were first written. Check them in your own program before you rely on them. The general 16-bit
tools and addresses are in [DOS and 16-bit Windows](../DOS.md); the Borland Pascal runtime
shared with Turbo Pascal is there too.

## Recognising Delphi

| Sign | Where | Tool |
|---|---|---|
| `RCDATA` resources that start with `TPF0`, named after a form's class in capitals (`TFORM1`) | the resource table | `tools/formats/dfm.py GAME.EXE --list` |
| Class names as Pascal strings in the code segments: `TForm`, `TApplication`, `TImage`, `TTimer`, `Exception` | the VCL's class tables | `dosinfo.py --pstrings` |
| The Borland Pascal runtime, with imports from `KERNEL`, `USER`, `GDI`, often `TOOLHELP` and `WIN87EM` | the NE header | `dosinfo.py` (fingerprint and imports) |
| The VCL's error messages, such as the stream and list errors | the string table resources | a resource viewer |

A 32-bit Delphi program has `TPF0` forms in `RCDATA` too, and the Borland linker of the 1990s
names its sections `CODE`, `DATA` and `BSS`. `dfm.py` reads its forms. The class tables on this
page are Delphi 1's. For 32-bit Delphi, IDR (Interactive Delphi Reconstructor) names the VCL and the
event handlers; it does not read 16-bit programs.

The order of the code segments follows the order of the units. In the program studied the
game's own units come first, the VCL's units next, SysUtils second to last and System last
(*observed* for SysUtils and System from their routines; *inferred* for the VCL from the class
names in each segment).

## Forms and their resources

A form resource is the binary form stream (DFM) that the VCL reads when it creates the form.
It holds the form and every control on it: their class, name, position and size, and every
property set in the designer. `tools/formats/dfm.py GAME.EXE derived/forms/` writes each form as
text. The format and the value types are in the tool's help.

Read the forms before the code. They give:

- each window's controls, with their places in pixels;
- each timer's `Interval` and `Enabled`: a game clock is often a `TTimer` (in the program
  studied, a 100 ms timer counts the game's ticks, *observed*);
- each event's handler, by name: `OnClick = StartButtonClick`. The handler is a published
  method of the form's class.

The project file (`.DPR`) creates the forms with `Application.CreateForm` in its main block, in
order. Each form's `OnCreate` runs at that point. Find those calls in the program's entry code
to know the order.

## Classes in a Delphi 1 program

An object is a block on the heap. Its first 4 bytes are a far pointer to its class's VMT, so
`TObject`'s instance size is 4 (*observed*). The fields follow. `TObject.NewInstance` allocates
the block, and `InitInstance` fills it with zeros before it stores the VMT pointer, as
Borland's documentation of `InitInstance` says. Every field of a new object starts at 0, `nil`
or `False`. A port that creates a form must do the same, or a counter the original never sets
before its first use reads garbage.

Each class's tables lie in the code segment of its unit, as data between the procedures. The
class reference (`TClass`) points into the VMT. The slots before that point are near pointers
into the same segment and far pointers whose selectors the loader patches (*observed*):

| Slot | Size | Holds |
|---|---|---|
| VMT − `0x22` | word | unknown (small numbers) |
| VMT − `0x20` | near | type information (RTTI): kind 7 (a class), the name, the unit's name, the published properties |
| VMT − `0x1e` | near | published field table |
| VMT − `0x1c` | near | published method table |
| VMT − `0x1a` | near | dynamic method table |
| VMT − `0x18` | near | the class name, a Pascal string |
| VMT − `0x16` | word | instance size in bytes |
| VMT − `0x14` | far | the parent class (0 for `TObject`) |
| VMT − `0x10`, `0x0c`, `0x08`, `0x04` | far | `DefaultHandler`, `NewInstance`, `FreeInstance`, `Destroy` |
| VMT + 0 | far | the virtual methods, 4 bytes each |

The method table is a word count, then a far pointer and a Pascal-string name for each
published method. The field table is a word count, a near pointer to a class table (a word
count and far pointers to VMTs), then for each published field a word offset in the object, a
word index into the class table and the field's name. A form's controls are its published
fields, so the field table gives the offset of each control in the form object.

32-bit Delphi uses the same tables with 4-byte pointers, more slots before the VMT, and a size
word at the start of each method entry.

`tools/dos/delphi1.py GAME.EXE classes` lists every class with its VMT, instance size and
parent. `methods` and `fields` list the published methods and fields. A table counts only when
its `NewInstance` and `FreeInstance` slots are relocated far pointers, its name is an
identifier and its parent is another class it found. In the program studied it found 129
classes and 154 published methods (*measured*).

### Old-style objects

Delphi 1 still compiles Borland Pascal's `object` types, and a game's own controls may use them.
Their VMT is in the data segment: a word with the instance size, a word with its negative, a
word for the dynamic method table, a reserved word, then a far pointer for each virtual method
from `+8` on (Borland Pascal 7's layout; the methods from `+8` *observed*). The object holds the
VMT's offset in a word among its fields. In the program studied that word is at `+9`, after a
rectangle and an enabled flag (*observed*). Calls go through `[vmt + 8]`, `[vmt + 0xc]` and so
on.

## Event handlers Ghidra misses

The VCL calls an event handler through a pointer. When it reads a form, it looks up each
event's method name in the class's method table and stores the far pointer in the control's
event property, an 8-byte method pointer (code, then `Self`). The control's code calls through
that pointer later. A handler assigned in code is stored the same way, and a method pointer in
initialised data is a far pointer in the data segment's relocation chains. Ghidra's analysis
does not follow these far pointers reliably. In the program studied its first export had no
function at 12 or more of the 154 published methods, the game clock's timer handler among them
(*observed*: a report of Ghidra's functions at those addresses).

Recover them in three steps:

1. Name every published method and make the missing functions:
   `tools/dos/delphi1.py GAME.EXE labels > analysis/delphi-labels.tsv`, then
   `scripts/ghidra-export.sh --labels analysis/delphi-labels.tsv GAME.EXE`.
2. Map each form's events to code: `tools/dos/delphi1.py GAME.EXE handlers` prints the form,
   the control, the event, the method and its address. Delphi finds a method by name without
   regard to case, and so does the tool. In the program studied all 137 events of its 23
   forms resolve, to the same addresses traced by hand (*measured*).
3. Measure the rest with `exportcov.py` (NE). It walks the relocation chains, those through an
   iterated data segment included ([Iterated segments](../DOS.md#iterated-segments)). In the
   program studied the far pointers of the method tables gave the missing published methods,
   and the chains through the data segment gave nine more handlers, reached only through
   method pointers there.

## Naming the runtime

The System unit is Borland Pascal's runtime with Delphi's additions. Name it first and do not
trace it. In the program studied it is the last code segment (*observed*):

| Routine | How to recognise it |
|---|---|
| `Random(range)`, `RandSeed`, `Randomize` | the multiplier `0x08088405` in a near helper built from 16-bit multiplies; `Random` returns the high word of the new seed times the range; `Randomize` reads the time with `INT 21h AH=2Ch`. The same as Borland Pascal 7 ([DOS](../DOS.md); [gameframe-turbo-pascal](../../crates/turbo-pascal/README.md) `TurboRandom`) |
| `LongMul`, `LongDivMod` | `DX:AX` times or divided by `BX:CX`; the quotient in `DX:AX`, the remainder in `BX:CX`, truncated toward zero |
| String helpers | a temporary string on the stack: load, append, store into a `string[N]` cut to N, `Copy`, `Delete`, compare with the result in the flags |
| Text files | `Assign`, `Reset`, `ReadLn`, `Eof`, `Val` work on a `TextRec` with its own buffer ([DOS](../DOS.md)) |
| `FillChar`, `Move`, `GetMem`, `FreeMem`, `Halt` | `rep stosb`, `rep movsb`, a size with a far pointer, `ExitCode` and the exit chain |
| Set membership | an element, the set's size and its base give a byte offset and a bit mask |
| Floating point | `Round` and `Trunc` with `frndint` and `fistp` (`Trunc` loads another control word first); `/` as a helper around `fdivp` with a check for the Pentium's FDIV flaw; a helper that converts a 6-byte `Real` to `Extended` |
| `NewInstance`, `FreeInstance`, `InitInstance` | the VMT slots above point at them |

SysUtils sits before System. `StrToInt` calls `Val` and raises `EConvertError` on a bad number.
`FileExists` tests `FileAge(name) <> -1`. Name the VCL's methods from the class tables and the
Windows calls they make: `TTimer`'s setters call `KillTimer` and `SetTimer`;
`TApplication.ProcessMessages` loops on `PeekMessage`; `TMediaPlayer` sends MCI commands.

## Floating point and WIN87EM

Delphi 1 has `Integer` (16 bits), `LongInt` (32 bits), the 6-byte `Real`, and, with `{$N+}`,
`Single`, `Double`, `Extended` and `Comp` on the x87.

- **The instructions are real x87 instructions.** Each one starts with `fwait` (`9B`) and has an
  NE OS fixup (a relocation whose flag bits 0 and 1 are both set). Its number says what follows
  the `fwait`: 2 an `SS` override, 3 a `CS` override (constants kept in the code segment), 4 an
  `ES` override, 5 an x87 opcode with no override, 6 nothing (a lone `fwait`). Type 1 (`FIARQQ`)
  does not occur in the program studied. Windows patches them into emulator calls through
  `WIN87EM.DLL` only when there is no coprocessor (*inferred* from the import and the fixup
  names in Microsoft's NE documentation). Ghidra and `dos_emu` see the x87 instructions
  themselves. In the program studied there are 334 such fixups in 35 functions (*observed*).
- **`Real` goes through the x87 too.** In the program studied a runtime helper converts a 6-byte
  `Real` to `Extended`, and the arithmetic runs on the x87 (*observed*). Without `{$N+}`, Turbo
  and Borland Pascal do `Real` arithmetic in software instead ([DOS](../DOS.md)).
- **Each step rounds to 64 bits; a `Double` store rounds to 53.** A port that computes in `f64`
  differs in the last bit now and then. [gameframe-x87](../../crates/x87/README.md) does the
  x87's arithmetic in integers.
- **`Round` rounds a half to the even neighbour.** It is `frndint` (or `fistp`) under the
  default control word. The software `Round` of the 6-byte `Real` rounds halves away from zero.
  The two give different results for 2.5.
- **The emulator's control word.** `dos_emu` does not set one. Unicorn then reports a control
  word of 0 but computes at 64 bits until an `fldcw` loads a word. A `Trunc` that stores the
  control word, sets the chop bits and loads it back would then switch to 24 bits (*measured*
  under Unicorn). Set the control word the program's start-up sets before you measure such code.

## Strings and data

- Strings are `ShortString`: a length byte and up to 255 characters. `string[N]` takes N + 1
  bytes. Delphi 1 has no long strings. Text is in the code page of Windows (1252 in Western
  Europe).
- A procedure's string constants are Pascal strings just before its code, so `strings.txt`
  misses many. Use `dosinfo.py --pstrings`.
- Typed constants and initial values are in the data segment, which Borland's linker often
  stores as iterated records ([Iterated segments](../DOS.md#iterated-segments)).
- `Boolean` is one byte, sets are bit arrays of up to 32 bytes. Check record offsets against a
  field table or the code before you rely on them.

## Pitfalls

- **Messages, not real time.** A `TTimer` posts `WM_TIMER`, and Windows merges timer messages
  that wait. The game's clock runs slower than the interval when a step takes long. Measure the
  rate in the running original.
- **The VCL's mouse rules.** A control captures the mouse when a button goes down on it.
  `OnClick` comes on release over the same control, before `OnMouseUp` (*observed*). Port the
  order, or the game's state differs after a click.
- **Handlers inside handlers.** Code that calls `Application.ProcessMessages` in a loop lets
  other handlers run in the middle of it, timers included. Look for those calls before you
  decide the order of a step.
- **The class tables are not code.** Gaps between functions in the code segments are mostly
  class tables and case jump tables. Do not make functions in them.

## Tools

| Tool | What it does here |
|---|---|
| `tools/dos/dosinfo.py` | Recognise the program: NE, the Borland runtime, imports, Pascal strings |
| `tools/formats/dfm.py` | Every form as text, the forms' list, and every event with its method's name |
| `tools/dos/delphi1.py` | Classes, published methods and fields, labels for Ghidra, events mapped to code |
| `tools/triage/exportcov.py` | How much of the code the export covers, and the starts it missed |
| `tools/dos/nedis.py`, `readseg.py` | Disassembly with relocations; typed constants from the expanded data segment |
| `tools/emu/dos_emu` | Run one handler or one runtime routine with chosen inputs |
| [gameframe-x87](../../crates/x87/README.md), [gameframe-turbo-pascal](../../crates/turbo-pascal/README.md) | The x87's arithmetic, `Random`, the 6-byte `Real` |

## Open questions

- The first slot of a Delphi 1 VMT (VMT − `0x22`) holds small numbers. What is it?
- Delphi 1's start-up control word and the precision of `WIN87EM`'s software emulator are not
  measured.
- Do Delphi 1's VCL units keep the same segment order in every program?
