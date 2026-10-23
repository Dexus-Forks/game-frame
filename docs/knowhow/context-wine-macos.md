---
id: context-wine-macos
title: 'Wine on macOS with Apple silicon: what to know before running a game there'
kind: context
status: context
platform: [macos, windows-9x]
runtime: [wine, rosetta, wow64, apple-silicon]
tags: [winedebug, winedbg, wineserver, crash-dialog, automation]
commands:
  - '(?:^|[;&|(]\s*|/|\bexec\s+|=\S*\s+)(?:wine|wine64|winedbg|wineserver|winecfg)(?=\s|$|")'
  - 'WINEDEBUG='
  - 'WINEPREFIX='
tools: []
skills: [re-observe, re-setup]
docs: [docs/observe/running-the-original.md, docs/observe/debuggers.md]
seen: ['POD: Planet of Death (1997), Wine 11.0 and 11.18, 2026-10-23']
---

# Wine on macOS with Apple silicon: what to know before running a game there

## What to know

On an Apple-silicon Mac, Wine runs as an x86-64 program under Rosetta. It runs a 32-bit game
through WOW64. Three layers translate the game. Some failures cannot be traced to any one layer
alone. Use Wine's trace channels in place of a debugger. Script every run. Check for leftover
processes before you read a result.

- **Trace channels.** `WINEDEBUG=+seh,+ddraw,+file,+dinput` logs exceptions with
  registers, surfaces, file opens and input. `+relay` with
  `HKCU\Software\Wine\Debug\RelayFromInclude` set to the game's module logs only the game's own
  calls. Attaching winedbg to a running game killed it ([winedbg](winedbg-attach-kills.md)).
- **No crash dialog.** Set `HKCU\Software\Wine\WineDbg\ShowCrashDialog` to 0, so a background
  thread's crash does not block the screen.
- **Run scripts.** Set `DYLD_FALLBACK_LIBRARY_PATH` inside the script. A value exported before
  the script starts is lost ([DYLD](macos-dyld-stripped.md)). Do not wait with `wineserver -w`
  ([wineserver](wineserver-wait-hangs.md)). Kill leftover game processes by name before each run
  ([leftover process](single-instance-leftover.md)).
- **Input.** macOS ignores synthetic input without the Accessibility permission. Wine windows
  belong to no app. Send input from inside Wine with `SendInput`
  ([input](wine-input-without-accessibility.md)). Bring the game's window to the front first
  ([DirectInput](directinput-not-foreground.md)).
- **Known open fault.** A 32-bit game's system calls can fault in `wow64cpu` now and then
  ([wow64cpu](wine-wow64-rosetta-fault.md)). If a game freezes after some seconds, read that
  entry before you look for a cause in the game.
- **Renderer.** For a 2D DirectDraw game, Wine's `renderer=gdi` (registry
  `HKCU\Software\Wine\Direct3D`) avoids an OpenGL start-up that crashed now and then under
  Rosetta (*measured* in POD).
