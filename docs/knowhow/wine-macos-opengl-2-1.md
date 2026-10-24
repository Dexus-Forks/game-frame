---
id: wine-macos-opengl-2-1
title: A wrapper DLL draws nothing under Wine on macOS, and its log says "version '130' is not supported"
kind: failure
status: solved
era: []
platform: [macos]
runtime: [wine, apple-silicon]
hardware: []
api: [glide]
compiler: []
genre: []
symptom: [wrong-picture]
tags: [opengl, glsl, core-profile, wgl]
signatures:
  - "version '1[3-9]0' is not supported"
  - "OpenGL 2\\.1 Metal"
tools: []
skills: [re-observe]
docs: [docs/hardware/wrappers.md]
seen: ['glidetest.exe through glide2x.dll, Wine 11 on an Apple M3 Max, 2026-10-25']
---

# A wrapper DLL draws nothing under Wine on macOS, and its log says "version '130' is not supported"

## Symptoms

A wrapper that draws with OpenGL opens its window, but every pixel stays black. Its log shows
`OpenGL 2.1 Metal - 90.5`, then a shader error: `version '130' is not supported`. Calls that do
not draw still work.

## Cause

Wine's macOS driver gives `wglCreateContext` an OpenGL 2.1 context, which has GLSL up to 1.20
(*observed*). macOS gives OpenGL 3.2 and later only as a forward-compatible core profile
(*observed*). A request for a core profile without the forward-compatible flag returns no
context (*observed*).

## Evidence

`glidetest.exe` under Wine on an Apple M3 Max. With `GLIDE_LOG` set, the log showed
`OpenGL 2.1 Metal`. Then `wglCreateContextAttribsARB` with version 3.2 and the core profile
returned null. With `WGL_CONTEXT_FLAGS_ARB` set to `WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB` it
returned a context that reports `OpenGL 4.1 Metal`, and all 27 checks of the test passed.

## What to do

Create the plain context. If `GL_VERSION` is below 3, create a core context with
`wglCreateContextAttribsARB`, with the forward-compatible flag (0x2094 = 2) and the core profile
(0x9126 = 1), and make it current. Compile the shaders as GLSL 1.50 there. Bind a vertex array
object before drawing, as a core context draws nothing without one. Keep line widths at 1, as a
forward-compatible context refuses wider lines. `wrappers/glide/src/window.rs` (`core_context`)
does this. The other choice is shaders in GLSL 1.20, as `wrappers/sgl` has.

## Tried

- A 3.2 core profile without the forward-compatible flag: no context.
- Loading `glGenVertexArrays` in the function table of a 2.1 context: missing, so the table
  fails to load. Load it apart, only in a core context.
