# Replacement libraries for a 1990s game

A game of the late 1990s often needs a library that no modern system has: a 3D card's own API
(Glide, ATI3DCIF, the S3D Toolkit), an old DirectDraw that grants any display mode, or a CPU
that allows port I/O. A replacement library with the same file name and the same exports runs
the unchanged game. It can also draw at a higher resolution, log every call, or serve as the
first piece of a port. This page covers how to write one, from the POD: Planet of Death
project, whose unchanged 3dfx, ATI and S3 builds run through replacements written that way
([case study](../case-studies/pod.md)).

## Start with a probe

Before drawing anything, write a probe: a library that exports every name the game imports,
logs every call with its arguments, and dumps the first records it is given (vertices as floats
and hex, surface structures as words). One run of the game then gives the layouts from the
game's own data, with no SDK header to hand. POD settled two APIs' vertex and surface layouts
this way.

1. **List the imports exactly.** `./gameframe run peinfo GAME.EXE --imports` gives the library's
   file name and each imported name. Early Glide 2 games import `glide.dll`, not
   `glide2x.dll`, with decorated `__stdcall` names such as `_grGlideInit@0`. Games that load
   the library with `LoadLibrary` name it only in their strings.
2. **List every call site with its constant arguments.** A scan of the call sites, reading the
   values pushed before each call (`./gameframe run refs GAME.EXE ADDRESS`, or `ghidra-run.sh
   callargs`), gives the game's whole use of the API in one table. It shows which states,
   formats and modes must be right, and which never occur.
3. **Read each call's success value at its call site.** SDKs of the time disagree with
   themselves: the S3D Toolkit's `InitLib` returns 0 on success, and `CreateRenderer` and every
   table function return 1 ([S3D Toolkit](../knowhow/s3dtk-error-setstate.md)).

## Build it

Write the library in Rust or C for 32-bit Windows. GameFrame's `scripts/zcc.sh` links a Rust
`cdylib` with `zig cc`, without MinGW, and exports exactly the names in the crate's `.def` file:

```toml
# .cargo/config.toml
[target.i686-pc-windows-gnu]
linker = "/path/to/gameframe/scripts/zcc.sh"
```

```text
; glide.def: decorated names as the game imports them
LIBRARY glide.dll
EXPORTS
_grGlideInit@0=grGlideInit@0
_grSstOpen@24=grSstOpen@24
```

```bash
ZCC_DEF_DIR=wrapper/glide cargo build --release --target i686-pc-windows-gnu
```

Use `panic = "abort"` in the profiles. Check the export table of the result (`peinfo --exports`)
against the game's imports, then call a simple export under `pe32_emu` before the first run in
the game.

## What the game assumes

- **A window it never made.** Early APIs drew full screen: Glide 2's `grSstOpen` takes no
  window handle. Find the game's own window by its class, even while hidden, since a game may
  open the API before it shows the window.
- **More than one thread.** A game may call the API from a loading thread too. An OpenGL or
  Direct3D context belongs to one thread, so record every call as a command and replay the
  commands on one render thread. Answer only queries (sizes, memory needs, read pointers) at
  once.
- **Direct access to the frame.** A linear frame buffer pointer has a fixed stride, and a game
  may set the write mode after it takes the pointer. Hand out a buffer filled with a marker, and
  read what changed when the game releases it, in the mode then current.
- **The card's depth and colour rules.** Map the card's depth to the new API's so the compare
  functions stay the same, clamp vertex colours where the card clamped them, and put gamma in
  the final copy to the window, where the card's output table was.
- **Pixel formats.** A ViRGE draws 15-bit pixels, never 5-6-5. Make the DirectDraw surfaces
  report the format the game names ([ViRGE surfaces](../knowhow/s3-virge-15-bit-surfaces.md)).
- **Port I/O.** A Windows 95 game may probe the CPU through ports. A vectored exception handler,
  installed from the library's `DllMain`, can step over `cli`, `sti` and `out` and make `in`
  read 0xff, with no change to the EXE ([privileged instruction](../knowhow/wine-privileged-instruction.md)).
- **Any display mode.** A modern display cannot change to 640 × 480 at 16 bits. A DirectDraw
  that keeps every surface in memory, grants any mode and shows the frame scaled in a window
  runs a full-screen DirectDraw game unchanged.

## Measure what the card is given

A build named after a card does not always give the card its scene. POD's ATI and S3 builds
drew the textured track and cars in software and gave the chips only untextured shadows and
light beams. A counter in the probe, by kind of call and texture use, over one real scene, says
whether redrawing the card's calls at a higher resolution can sharpen the picture, or whether
only the finished frame can be scaled ([3D-card builds](../knowhow/context-3d-card-builds.md)).

## Sibling builds check each other

When a game ships one build per card, the builds draw the same scene through different
libraries. Compare the same scene in two builds: an error in one replacement shows as a
difference, as an unclamped colour did in POD's ATI build against its S3 build.

## When it runs slowly

Log frames per second, the share of time spent drawing and the time per kind of command from
the start. Software OpenGL in an emulated PC (Mesa's llvmpipe) drew POD's menus at 2 to 5 frames
a second; `LP_NATIVE_VECTOR_WIDTH=128` roughly doubled that
([a Glide 2 replacement](../knowhow/context-glide2-wrapper.md)).
