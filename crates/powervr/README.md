# gameframe-powervr

A software model of PowerVR's PCX2 as VideoLogic's SGL 3.1 drives it. SGL is a scene graph,
not a triangle API: a game builds display lists of transforms, materials, meshes, lights and
cameras once, edits a few of them each frame and calls `sgl_render`. A port that has traced
how a game's PowerVR build builds its scene can build the same graph here and render it at
any multiple of the device's size, with no SGL library and no GPU, natively and in
WebAssembly.

What it models:

- the scene graph: lists (kept state or not, ignored or not), transforms (`sgl_translate`,
  `sgl_rotate`, `sgl_scale`), meshes of three- and four-sided faces with normals and texture
  coordinates, the material calls (ambient, diffuse, specular, glow, opacity, texture map and
  the four texture effects), cameras, fog and background colours, ambient and point (spot)
  lights, `sgl_new_translucent` and `sgl_qual_fog`, light volumes;
- the lighting each vertex gets, as PowerVR's driver (`SGLMID4.dll`) works it: a spot light's
  `(1 − min((1 − cos) log2 c, 1))²`, no fall-off with distance, and the texture effects
  sending each term either into the colour the texel is multiplied by or into the colour added
  after it;
- the PCX2's per-pixel rules: the nearest opaque surface wins whatever the order (the chip
  sorted per pixel), the projection from the camera rectangle and zoom, the near and far
  planes, fog as `(1 − density)` kept per unit of depth, light volumes adding their light to
  what is inside them, translucent passes blended in order, and an RGB565 frame buffer;
- `Sgl`, a software SGL context: the calls by their names in `sgl.h`
  (`sgl_create_list`, `sgl_add_vertices`, `sgl_set_texture_effect`, `sgl_create_texture`,
  `sgl_set_point_light`, `sgl_render` and the rest).

The rules come from the declarations in `sgl.h` (SGL 3.1, March 1997) and from PowerVR's own
driver `SGLMID4.dll`, exported with Ghidra: the functions are named where they are used.
Where neither settles a detail (the texel layouts, the filtering, the output dither, how the
colours are iterated, how strongly a light volume lights), the code says so in a `Deviation:`
comment. Nothing has been compared with a real card yet.

What it does not model: SGL's immediate mode (`sgltri_*`; GameFrame's `wrappers/sgl` has it),
convex objects other than light volumes, shadow volumes in the scene graph, mipmaps, the
chip's tiles and regions, `sgl_position_light` (a point light is placed from the camera, which
POD's car lamps need: *inferred*), and the address callback and DirectDraw surface a game's
picture goes through.

The scene graph is shared with `wrappers/sgl`, the OpenGL stand-in for `sgl.dll` that runs a
game's own PowerVR build.

First used for POD Gold's PowerVR build, `WPODPVR.EXE` (POD's `apps/unpod/src/render_pvr.rs`).
