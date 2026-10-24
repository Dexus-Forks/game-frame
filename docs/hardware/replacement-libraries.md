# Replacement libraries for a 1990s game

A game of the late 1990s often needs a library that no modern system has. It may need a 3D
card's own API (Glide, ATI3DCIF, the S3D Toolkit), an old DirectDraw that grants any display
mode, or a CPU that allows port I/O. A replacement library with the original's file name and
exports runs the unchanged game. It can also draw at a higher resolution, log every call, or
serve as the first piece of a port. This page covers how to write one. The method comes from the
POD: Planet of Death project. Its unchanged 3dfx, ATI and S3 builds run through replacements
written this way ([case study](../case-studies/pod.md)).

## Start with a probe

Before you draw anything, write a probe. A probe is a library that exports every name the game
imports and logs every call with its arguments. It also dumps the first records it is given
(vertices as floats and hex, surface structures as words). One run of the game then gives the
layouts from the game's own data, with no SDK header to hand. POD settled the vertex and
surface layouts of two APIs this way.

1. **List the imports exactly.** `./gameframe run peinfo GAME.EXE --imports` gives the library's
   file name and each imported name. Early Glide 2 games import `glide.dll`, not
   `glide2x.dll`. They use decorated `__stdcall` names such as `_grGlideInit@0`. Games that load
   the library with `LoadLibrary` name it only in their strings.
2. **List every call site with its constant arguments.** Scan the call sites and read the
   values pushed before each call (`./gameframe run refs GAME.EXE ADDRESS`, or `ghidra-run.sh
   callargs`). The result is one table of the game's whole use of the API. It shows which
   states, formats and modes must be right. It also shows which ones never occur.
3. **Read each call's success value at its call site.** SDKs of the time are not consistent
   with themselves. The S3D Toolkit's `InitLib` returns 0 on success. Its `CreateRenderer` and
   every table function return 1 ([S3D Toolkit](../knowhow/s3dtk-error-setstate.md)).

## Build it

Write the library in Rust or C for 32-bit Windows. GameFrame's `scripts/zcc.sh` links a Rust
`cdylib` with `zig cc` and needs no MinGW. The DLL exports exactly the names in the crate's
`.def` file:

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
against the game's imports. Then call a simple export under `pe32_emu` before the first run in
the game.

## What the game assumes

- **A window it never made.** Early APIs drew full screen. Glide 2's `grSstOpen` takes no
  window handle. Find the game's own window by its class, even while the window is hidden. A
  game may open the API before it shows the window.
- **More than one thread.** A game may call the API from a loading thread too. An OpenGL or
  Direct3D context belongs to one thread. Record every call as a command. Replay the commands
  on one render thread. Answer only queries (sizes, memory needs, read pointers) at once.
- **Direct access to the frame.** A linear frame buffer pointer has a fixed stride. A game may
  set the write mode after it takes the pointer. Hand out a buffer filled with a marker. When
  the game releases the buffer, read what changed, in the write mode current at that time.
- **The card's depth and colour rules.** Map the card's depth to the new API's depth, so that
  the compare functions stay the same. Clamp vertex colours where the card clamped them. Put
  gamma in the final copy to the window. The card applied gamma in its output table.
- **Pixel formats.** A ViRGE draws 15-bit pixels. It never draws 5-6-5. Make the DirectDraw
  surfaces report the format the game names ([ViRGE surfaces](../knowhow/s3-virge-15-bit-surfaces.md)).
- **Port I/O.** A Windows 95 game may probe the CPU through ports. The library's `DllMain` can
  install a vectored exception handler. The handler can step over `cli`, `sti` and `out`, and
  make `in` read 0xff. The EXE needs no change ([privileged instruction](../knowhow/wine-privileged-instruction.md)).
- **Any display mode.** A modern display cannot change to 640 × 480 at 16 bits. A replacement
  DirectDraw can keep every surface in memory, grant any mode and show the frame scaled in a
  window. Such a DirectDraw runs a full-screen DirectDraw game unchanged.

## Measure what the card is given

A build named after a card does not always give the card its scene to draw. POD's ATI and S3
builds drew the textured track and cars in software. They gave the chips only untextured
shadows and light beams. Add a counter to the probe, by kind of call and by texture use, and run
it over one real scene. The counts say whether a redraw of the card's calls at a higher
resolution can sharpen the picture. If it cannot, only the finished frame can be scaled
([3D-card builds](../knowhow/context-3d-card-builds.md)).

## Sibling builds check each other

When a game ships one build per card, the builds draw the same scene through different
libraries. Compare the same scene in two builds. An error in one replacement shows as a
difference between them. In POD, an unclamped colour in the ATI build showed this way against
the S3 build.

## When it runs slowly

From the start, log frames per second, the share of time spent drawing and the time per kind of
command. Software OpenGL in an emulated PC (Mesa's llvmpipe) drew POD's menus at 2 to 5 frames
a second. `LP_NATIVE_VECTOR_WIDTH=128` roughly doubled that
([a Glide 2 replacement](../knowhow/context-glide2-wrapper.md)).
