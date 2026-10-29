//! An old game's bitmap font as a TrueType font, built in memory.
//!
//! Many old games keep their fonts as cells of palette indices: a width, a height and an
//! advance per character. A launcher or a page that wants to write in the game's own letters
//! can turn such a font into TrueType with [`build`] and give it to a web view as CSS
//! ([`font_face`]). Each set pixel becomes part of a square outline: runs of pixels in a row,
//! merged with the same run in the rows below, make one rectangle each. Nothing is written to
//! disk, so a project can make the font from the player's own files at run time and commit
//! none of the game.
//!
//! The tables are the minimum a browser's font loader takes (OpenType's `head`, `hhea`, `maxp`,
//! `OS/2`, `hmtx`, `cmap` format 4, `loca`, `glyf`, `name` and `post` 3.0), with no hinting.
//! Draw it at a whole multiple of the cell's height in CSS pixels to keep each pixel whole.
//! Hardwar's launcher uses it for the game's menu fonts.
//!
//! ```
//! use gameframe_bitmap_font::{build, font_face, Bitmap, Glyph};
//! let i = [1u8, 1, 1, 1];
//! let font = Bitmap { family: "Tiny", width: 1, height: 4, baseline: 4,
//!     glyphs: vec![Glyph { code: 'i' as u16, advance: 2, pixels: &i }] };
//! let css = font_face("Tiny", &build(&font));
//! assert!(css.starts_with("@font-face"));
//! ```

/// Font units per pixel of the bitmap.
const UNIT: i32 = 64;

/// One character of a bitmap font.
pub struct Glyph<'a> {
    /// The Unicode code point it stands for.
    pub code: u16,
    /// How far the pen moves, in pixels.
    pub advance: u32,
    /// `width` × `height` cells, top row first; 0 is blank, anything else ink.
    pub pixels: &'a [u8],
}

/// A whole bitmap font: fixed cells and the row its baseline is under.
pub struct Bitmap<'a> {
    pub family: &'a str,
    pub width: u32,
    pub height: u32,
    /// Rows above the baseline: the cell's rows from 0 to `baseline - 1` stand on it.
    pub baseline: u32,
    pub glyphs: Vec<Glyph<'a>>,
}

/// A rectangle of pixels: columns `x0..x1`, rows `y0..y1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
}

/// The glyph's ink as rectangles: each row's runs, grown down while the next row has the
/// same run.
fn rects(pixels: &[u8], width: u32, height: u32) -> Vec<Rect> {
    let ink = |x: u32, y: u32| {
        pixels
            .get((y * width + x) as usize)
            .is_some_and(|&p| p != 0)
    };
    let mut open: Vec<Rect> = Vec::new();
    let mut done = Vec::new();
    for y in 0..height {
        let mut runs = Vec::new();
        let mut x = 0;
        while x < width {
            if ink(x, y) {
                let start = x;
                while x < width && ink(x, y) {
                    x += 1;
                }
                runs.push((start, x));
            } else {
                x += 1;
            }
        }
        let mut next = Vec::new();
        for r in open.drain(..) {
            if let Some(i) = runs.iter().position(|&(a, b)| (a, b) == (r.x0, r.x1)) {
                runs.remove(i);
                next.push(Rect { y1: y + 1, ..r });
            } else {
                done.push(r);
            }
        }
        next.extend(runs.into_iter().map(|(x0, x1)| Rect {
            x0,
            x1,
            y0: y,
            y1: y + 1,
        }));
        open = next;
    }
    done.extend(open);
    done.sort_by_key(|r| (r.y0, r.x0));
    done
}

fn u16be(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn i16be(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn u32be(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn clamp16(v: i32) -> i16 {
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// One glyph's `glyf` record and its bounds (x min, y min, x max, y max), or an empty record.
fn outline(g: &Glyph<'_>, font: &Bitmap<'_>) -> (Vec<u8>, Option<[i16; 4]>, usize) {
    let rs = rects(g.pixels, font.width, font.height);
    if rs.is_empty() {
        return (Vec::new(), None, 0);
    }
    let base = font.baseline as i32;
    // Clockwise in y-up units: top left, top right, bottom right, bottom left.
    let points: Vec<[i32; 2]> = rs
        .iter()
        .flat_map(|r| {
            let (x0, x1) = (r.x0 as i32 * UNIT, r.x1 as i32 * UNIT);
            let (top, bottom) = ((base - r.y0 as i32) * UNIT, (base - r.y1 as i32) * UNIT);
            [[x0, top], [x1, top], [x1, bottom], [x0, bottom]]
        })
        .collect();
    let bounds = [
        points.iter().map(|p| p[0]).min().unwrap_or(0),
        points.iter().map(|p| p[1]).min().unwrap_or(0),
        points.iter().map(|p| p[0]).max().unwrap_or(0),
        points.iter().map(|p| p[1]).max().unwrap_or(0),
    ]
    .map(clamp16);
    let mut out = Vec::new();
    i16be(&mut out, rs.len() as i16);
    for b in bounds {
        i16be(&mut out, b);
    }
    for i in 0..rs.len() {
        u16be(&mut out, (i * 4 + 3) as u16);
    }
    u16be(&mut out, 0); // no instructions
    out.resize(out.len() + points.len(), 1u8); // on the curve, 16-bit deltas
    let mut last = [0i32, 0];
    let mut ys = Vec::new();
    for p in &points {
        i16be(&mut out, clamp16(p[0] - last[0]));
        i16be(&mut ys, clamp16(p[1] - last[1]));
        last = *p;
    }
    out.extend(ys);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    (out, Some(bounds), points.len())
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, c| {
        let mut w = [0u8; 4];
        w[..c.len()].copy_from_slice(c);
        sum.wrapping_add(u32::from_be_bytes(w))
    })
}

/// The `name` table: family, style, full name and PostScript name, in UTF-16 for Windows.
fn name_table(family: &str) -> Vec<u8> {
    let postscript: String = family
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let names = [
        (1u16, family.to_string()),
        (2, "Regular".to_string()),
        (4, family.to_string()),
        (6, postscript),
    ];
    let mut strings = Vec::new();
    let mut out = Vec::new();
    u16be(&mut out, 0);
    u16be(&mut out, names.len() as u16);
    u16be(&mut out, (6 + 12 * names.len()) as u16);
    for (id, text) in names {
        let utf16: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        for v in [3, 1, 0x409, id, utf16.len() as u16, strings.len() as u16] {
            u16be(&mut out, v);
        }
        strings.extend(utf16);
    }
    out.extend(strings);
    out
}

/// `cmap` format 4: one segment for each run of consecutive codes with consecutive glyphs.
fn cmap_table(codes: &[u16]) -> Vec<u8> {
    // (first code, last code, first glyph id); glyph 0 is `.notdef`.
    let mut segs: Vec<(u16, u16, u16)> = Vec::new();
    for (i, &c) in codes.iter().enumerate() {
        let gid = i as u16 + 1;
        match segs.last_mut() {
            Some(s) if s.1 + 1 == c && s.2 + (s.1 - s.0) + 1 == gid => s.1 = c,
            _ => segs.push((c, c, gid)),
        }
    }
    segs.push((0xffff, 0xffff, 0));
    let n = segs.len() as u16;
    let mut power = 1u16;
    let mut log = 0u16;
    while power * 2 <= n {
        power *= 2;
        log += 1;
    }
    let mut sub = Vec::new();
    u16be(&mut sub, 4);
    u16be(&mut sub, 0); // length, below
    u16be(&mut sub, 0);
    u16be(&mut sub, n * 2);
    u16be(&mut sub, power * 2);
    u16be(&mut sub, log);
    u16be(&mut sub, n * 2 - power * 2);
    for s in &segs {
        u16be(&mut sub, s.1);
    }
    u16be(&mut sub, 0);
    for s in &segs {
        u16be(&mut sub, s.0);
    }
    for s in &segs {
        let delta = if s.0 == 0xffff {
            1
        } else {
            s.2.wrapping_sub(s.0)
        };
        u16be(&mut sub, delta);
    }
    for _ in &segs {
        u16be(&mut sub, 0);
    }
    let len = sub.len() as u16;
    sub[2..4].copy_from_slice(&len.to_be_bytes());
    let mut out = Vec::new();
    u16be(&mut out, 0);
    u16be(&mut out, 1);
    u16be(&mut out, 3);
    u16be(&mut out, 1);
    u32be(&mut out, 12);
    out.extend(sub);
    out
}

/// The font file.
pub fn build(font: &Bitmap<'_>) -> Vec<u8> {
    let mut glyphs: Vec<&Glyph<'_>> = font.glyphs.iter().filter(|g| g.advance > 0).collect();
    glyphs.sort_by_key(|g| g.code);
    glyphs.dedup_by_key(|g| g.code);
    let em = (font.height as i32 * UNIT) as u16;
    let ascent = (font.baseline as i32 * UNIT) as i16;
    let descent = -((font.height as i32 - font.baseline as i32) * UNIT) as i16;

    let mut glyf = Vec::new();
    // `.notdef` starts and ends at 0: it has no outline.
    let mut loca = vec![0u32, 0];
    let mut hmtx = Vec::new();
    let (mut max_points, mut max_contours, mut max_advance) = (0usize, 0usize, 0u16);
    let mut all = [i16::MAX, i16::MAX, i16::MIN, i16::MIN];
    // .notdef: empty, a cell wide.
    u16be(&mut hmtx, (font.width as i32 * UNIT) as u16);
    i16be(&mut hmtx, 0);
    for g in &glyphs {
        let (record, bounds, points) = outline(g, font);
        glyf.extend(record);
        loca.push(glyf.len() as u32);
        let advance = (g.advance as i32 * UNIT) as u16;
        max_advance = max_advance.max(advance);
        u16be(&mut hmtx, advance);
        i16be(&mut hmtx, bounds.map_or(0, |b| b[0]));
        if let Some(b) = bounds {
            max_points = max_points.max(points);
            max_contours = max_contours.max(points / 4);
            all = [
                all[0].min(b[0]),
                all[1].min(b[1]),
                all[2].max(b[2]),
                all[3].max(b[3]),
            ];
        }
    }
    if all[0] > all[2] {
        all = [0; 4];
    }
    let count = glyphs.len() as u16 + 1;

    let mut head = Vec::new();
    u32be(&mut head, 0x0001_0000);
    u32be(&mut head, 0x0001_0000);
    u32be(&mut head, 0); // checksum adjustment, below
    u32be(&mut head, 0x5f0f_3cf5);
    u16be(&mut head, 0x000b);
    u16be(&mut head, em);
    head.extend([0u8; 16]); // created, modified
    for b in all {
        i16be(&mut head, b);
    }
    u16be(&mut head, 0); // style
    u16be(&mut head, 8); // smallest readable size
    i16be(&mut head, 2);
    i16be(&mut head, 1); // long loca
    i16be(&mut head, 0);

    let mut hhea = Vec::new();
    u32be(&mut hhea, 0x0001_0000);
    i16be(&mut hhea, ascent);
    i16be(&mut hhea, descent);
    i16be(&mut hhea, 0);
    u16be(&mut hhea, max_advance);
    i16be(&mut hhea, all[0]);
    i16be(&mut hhea, 0);
    i16be(&mut hhea, all[2]);
    i16be(&mut hhea, 1);
    i16be(&mut hhea, 0);
    i16be(&mut hhea, 0);
    hhea.extend([0u8; 8]);
    i16be(&mut hhea, 0);
    u16be(&mut hhea, count);

    let mut maxp = Vec::new();
    u32be(&mut maxp, 0x0001_0000);
    u16be(&mut maxp, count);
    u16be(&mut maxp, max_points as u16);
    u16be(&mut maxp, max_contours as u16);
    u16be(&mut maxp, 0);
    u16be(&mut maxp, 0);
    u16be(&mut maxp, 2);
    maxp.extend([0u8; 16]);

    let codes: Vec<u16> = glyphs.iter().map(|g| g.code).collect();
    let mut os2 = Vec::new();
    u16be(&mut os2, 4);
    i16be(&mut os2, (font.width as i32 * UNIT) as i16);
    u16be(&mut os2, 400);
    u16be(&mut os2, 5);
    u16be(&mut os2, 0); // installable
    for v in [em / 2, em / 2, 0, em / 4, em / 2, em / 2, 0, em / 2] {
        i16be(&mut os2, v as i16);
    }
    i16be(&mut os2, UNIT as i16);
    i16be(&mut os2, ascent / 3);
    i16be(&mut os2, 0);
    os2.extend([0u8; 10]); // panose
    u32be(&mut os2, 0b11); // Basic Latin, Latin-1 Supplement
    os2.extend([0u8; 12]);
    os2.extend(*b"GFHW");
    u16be(&mut os2, 0x40); // regular
    u16be(&mut os2, codes.first().copied().unwrap_or(0x20));
    u16be(&mut os2, codes.last().copied().unwrap_or(0x20));
    i16be(&mut os2, ascent);
    i16be(&mut os2, descent);
    i16be(&mut os2, 0);
    u16be(&mut os2, ascent as u16);
    u16be(&mut os2, (-descent) as u16);
    u32be(&mut os2, 1); // Latin 1
    u32be(&mut os2, 0);
    i16be(&mut os2, ascent / 2);
    i16be(&mut os2, ascent);
    u16be(&mut os2, 0);
    u16be(&mut os2, 0x20);
    u16be(&mut os2, 1);

    let mut post = Vec::new();
    u32be(&mut post, 0x0003_0000);
    u32be(&mut post, 0);
    i16be(&mut post, descent / 2);
    i16be(&mut post, UNIT as i16);
    post.extend([0u8; 20]);

    let mut loca_bytes = Vec::new();
    for o in loca {
        u32be(&mut loca_bytes, o);
    }

    let tables: [(&[u8; 4], Vec<u8>); 10] = [
        (b"OS/2", os2),
        (b"cmap", cmap_table(&codes)),
        (b"glyf", glyf),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"loca", loca_bytes),
        (b"maxp", maxp),
        (b"name", name_table(font.family)),
        (b"post", post),
    ];
    let n = tables.len() as u16;
    let mut out = Vec::new();
    u32be(&mut out, 0x0001_0000);
    u16be(&mut out, n);
    u16be(&mut out, 128); // search range: 8 tables × 16
    u16be(&mut out, 3);
    u16be(&mut out, n * 16 - 128);
    let mut offset = 12 + 16 * tables.len();
    let mut body = Vec::new();
    let mut head_at = 0;
    for (tag, data) in &tables {
        if *tag == b"head" {
            head_at = offset;
        }
        out.extend(*tag);
        u32be(&mut out, checksum(data));
        u32be(&mut out, offset as u32);
        u32be(&mut out, data.len() as u32);
        body.extend(data);
        while body.len() % 4 != 0 {
            body.push(0);
        }
        offset = 12 + 16 * tables.len() + body.len();
    }
    out.extend(body);
    let adjust = 0xb1b0_afbau32.wrapping_sub(checksum(&out));
    out[head_at + 8..head_at + 12].copy_from_slice(&adjust.to_be_bytes());
    out
}

/// A `@font-face` rule for a font file from [`build`], as a data URL.
pub fn font_face(family: &str, ttf: &[u8]) -> String {
    format!(
        "@font-face {{ font-family: \"{family}\"; src: url(\"{}\") format(\"truetype\"); }}\n",
        data_url("font/ttf", ttf)
    )
}

/// `data:MIME;base64,...` for any bytes, such as a font or a picture.
pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = format!("data:{mime};base64,");
    for c in bytes.chunks(3) {
        let n = c
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | (b as u32) << (16 - 8 * i));
        for i in 0..4 {
            out.push(if i <= c.len() {
                ABC[(n >> (18 - 6 * i) & 63) as usize] as char
            } else {
                '='
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 3 × 3 font: `+` and `L`.
    fn font() -> Bitmap<'static> {
        const PLUS: [u8; 9] = [0, 1, 0, 1, 1, 1, 0, 1, 0];
        const ELL: [u8; 9] = [1, 0, 0, 1, 0, 0, 1, 1, 0];
        Bitmap {
            family: "Test Font",
            width: 3,
            height: 3,
            baseline: 3,
            glyphs: vec![
                Glyph {
                    code: b'+' as u16,
                    advance: 4,
                    pixels: &PLUS,
                },
                Glyph {
                    code: b'L' as u16,
                    advance: 3,
                    pixels: &ELL,
                },
                Glyph {
                    code: b' ' as u16,
                    advance: 2,
                    pixels: &[0; 9],
                },
            ],
        }
    }

    #[test]
    fn runs_grow_down_into_rectangles() {
        let f = font();
        let ell = rects(f.glyphs[1].pixels, 3, 3);
        assert_eq!(
            ell,
            [
                Rect {
                    x0: 0,
                    x1: 1,
                    y0: 0,
                    y1: 2
                },
                Rect {
                    x0: 0,
                    x1: 2,
                    y0: 2,
                    y1: 3
                }
            ]
        );
        assert_eq!(rects(f.glyphs[0].pixels, 3, 3).len(), 3);
    }

    #[test]
    fn the_file_has_its_tables_and_its_checksum() {
        let ttf = build(&font());
        assert_eq!(&ttf[..4], &[0, 1, 0, 0]);
        assert_eq!(u16::from_be_bytes([ttf[4], ttf[5]]), 10);
        // The whole file sums to the magic number once the adjustment is in.
        assert_eq!(checksum(&ttf), 0xb1b0_afba);
        // Each table's record points inside the file.
        for i in 0..10 {
            let r = &ttf[12 + 16 * i..28 + 16 * i];
            let at = u32::from_be_bytes([r[8], r[9], r[10], r[11]]) as usize;
            let len = u32::from_be_bytes([r[12], r[13], r[14], r[15]]) as usize;
            assert!(
                at % 4 == 0 && at + len <= ttf.len(),
                "{}",
                String::from_utf8_lossy(&r[..4])
            );
        }
    }

    #[test]
    fn each_glyph_has_its_own_outline() {
        // `loca` has one offset more than there are glyphs, `.notdef`'s range empty, then
        // ' ' (no ink, empty), '+' (3 rectangles) and 'L' (2).
        let ttf = build(&font());
        let table = |tag: &[u8]| {
            let r = (0..10)
                .map(|i| &ttf[12 + 16 * i..28 + 16 * i])
                .find(|r| &r[..4] == tag)
                .unwrap();
            let at = u32::from_be_bytes([r[8], r[9], r[10], r[11]]) as usize;
            let len = u32::from_be_bytes([r[12], r[13], r[14], r[15]]) as usize;
            &ttf[at..at + len]
        };
        let loca: Vec<usize> = table(b"loca")
            .chunks(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]) as usize)
            .collect();
        assert_eq!(loca.len(), 5);
        assert_eq!((loca[0], loca[1], loca[2]), (0, 0, 0));
        let glyf = table(b"glyf");
        let contours = |at: usize| i16::from_be_bytes([glyf[at], glyf[at + 1]]);
        assert_eq!(contours(loca[2]), 3);
        assert_eq!(contours(loca[3]), 2);
        assert_eq!(loca[4], glyf.len());
    }

    #[test]
    fn data_urls_pad_as_base64_does() {
        assert_eq!(
            data_url("text/plain", b"Man"),
            "data:text/plain;base64,TWFu"
        );
        assert_eq!(data_url("text/plain", b"Ma"), "data:text/plain;base64,TWE=");
        assert_eq!(data_url("text/plain", b"M"), "data:text/plain;base64,TQ==");
    }

    #[test]
    fn the_map_sends_each_code_to_its_glyph() {
        // ' ' 0x20, '+' 0x2b, 'L' 0x4c: three segments and the end.
        let cmap = cmap_table(&[0x20, 0x2b, 0x4c]);
        let seg_x2 = u16::from_be_bytes([cmap[12 + 6], cmap[12 + 7]]);
        assert_eq!(seg_x2, 8);
    }
}
