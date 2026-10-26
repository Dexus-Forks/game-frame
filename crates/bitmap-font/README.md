# gameframe-bitmap-font

An old game's bitmap font as a TrueType font, built in memory. Many games of the 1990s keep
their fonts as fixed cells of palette indices, with an advance for each character. A launcher's
theme, a page or a tool that wants to write in the game's own letters gives this crate the cells
(`Bitmap`, one `Glyph` per character with its Unicode code point) and gets a `.ttf` back
(`build`). `font_face` wraps it in a CSS `@font-face` rule with a data URL, ready for a web view
such as the frame's (`FrameSetup::with_theme`).

Each set pixel becomes part of a square outline, 64 font units to the pixel. Runs of pixels in
a row, merged with the same run in the rows below, make one rectangle each. The tables are the
few a browser's font loader needs (`head`, `hhea`, `maxp`, `OS/2`, `hmtx`, `cmap` format 4,
`loca`, `glyf`, `name`, `post` 3.0), with no hinting. Draw the font at a whole multiple of the
cell's height in CSS pixels, and each pixel stays whole.

Nothing is written to disk, so a project can make the font from the player's own files each
time it starts and commit none of the game. Hardwar's launcher does this with the game's
`HARDS.FNT` and `HARDL.FNT`.

No dependencies and no `unsafe` code; builds for WebAssembly.

```rust
use gameframe_bitmap_font::{build, font_face, Bitmap, Glyph};

let i = [1u8, 1, 1, 1];
let font = Bitmap {
    family: "Tiny",
    width: 1,
    height: 4,
    baseline: 4,
    glyphs: vec![Glyph { code: 'i' as u16, advance: 2, pixels: &i }],
};
let css = font_face("Tiny", &build(&font));
assert!(css.starts_with("@font-face"));
```

Licence: Apache 2.0.
