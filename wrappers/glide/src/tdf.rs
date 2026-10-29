//! 3DF texture files (`gu3dfGetInfo`, `gu3dfLoad`), 3dfx's format for a texture and its mip
//! levels. Plain Rust, tested on the host.
//!
//! A 3DF file starts with four text lines, then the binary data:
//!
//! ```text
//! 3df v1.0
//! rgb565
//! lod range: 1 256
//! aspect ratio: 1 1
//! ```
//!
//! The format name is one of `FORMATS`. The lod range is the smallest and largest side in
//! texels, and the aspect ratio is width to height. Glide 2.43 reads the header with
//! `sscanf("3df v%s %s lod range: %i %i aspect ratio: %i %i")` (the format string is in
//! `glide2x.dll`). After the header, the palettised formats have a palette of 256 big-endian
//! 32-bit words `0x00RRGGBB`, and the NCC formats have an NCC table of 16 Y, 12 I and 12 Q
//! values, each a big-endian 16-bit word. Then the levels follow, largest first, packed. Texels
//! of 16 bits are big-endian in the file and little-endian in memory.

use voodoo::formats::{Format, size};

/// The format names, indexed by `GR_TEXFMT_*` (6 and 7 are reserved).
pub const FORMATS: [&str; 15] =
    ["rgb332", "yiq", "a8", "i8", "ai44", "p8", "", "", "argb8332", "ayiq8422", "rgb565", "argb1555", "argb4444", "ai88", "ap88"];

/// `Gu3dfHeader`, and where the data starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub width: u32,
    pub height: u32,
    pub small_lod: u32,
    pub large_lod: u32,
    pub aspect: u32,
    pub format: Format,
    /// The byte after the fourth line.
    pub data_start: usize,
}

/// A side in texels to `GR_LOD_*`: 256 is 0, 1 is 8.
fn lod(side: u32) -> Option<u32> {
    (0..=8).find(|l| 256 >> l == side)
}

/// Width and height to `GR_ASPECT_*`: 8:1 is 0, 1:1 is 3, 1:8 is 6.
fn aspect(w: u32, h: u32) -> Option<u32> {
    match (w, h) {
        (8, 1) => Some(0),
        (4, 1) => Some(1),
        (2, 1) => Some(2),
        (1, 1) => Some(3),
        (1, 2) => Some(4),
        (1, 4) => Some(5),
        (1, 8) => Some(6),
        _ => None,
    }
}

/// Read the header.
pub fn header(file: &[u8]) -> Option<Header> {
    let mut lines = 0;
    let end = file.iter().position(|&b| {
        lines += (b == b'\n') as u32;
        lines == 4
    })?;
    let text = std::str::from_utf8(&file[..end]).ok()?;
    let t: Vec<&str> = text.split_whitespace().collect();
    if t.len() != 11 || t[0] != "3df" || !t[1].starts_with('v') || t[3..5] != ["lod", "range:"] || t[7..9] != ["aspect", "ratio:"] {
        return None;
    }
    let name = t[2].to_ascii_lowercase();
    let format = FORMATS.iter().position(|f| !f.is_empty() && *f == name).and_then(|i| Format::from_u32(i as u32))?;
    let n = |i: usize| t[i].parse::<u32>().ok();
    let (small_lod, large_lod) = (lod(n(5)?)?, lod(n(6)?)?);
    let aspect = aspect(n(9)?, n(10)?)?;
    if small_lod < large_lod {
        return None;
    }
    let (w, h) = size(large_lod, aspect)?;
    Some(Header { width: w as u32, height: h as u32, small_lod, large_lod, aspect, format, data_start: end + 1 })
}

/// The table a 3DF file carries before its texels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Table {
    None,
    /// 256 words `0x00RRGGBB`.
    Palette(Vec<u32>),
    /// A `GuNccTable` as it lies in memory: 112 bytes.
    Ncc(Vec<u8>),
}

/// The texels of every level (16-bit ones turned little-endian) and the table.
pub fn load(file: &[u8], h: &Header) -> Option<(Table, Vec<u8>)> {
    let mut at = h.data_start;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = file.get(at..at + n)?;
        at += n;
        Some(s)
    };
    let table = match h.format {
        Format::P8 | Format::Ap88 => {
            let b = take(1024)?;
            Table::Palette(b.chunks(4).map(|w| u32::from_be_bytes([w[0], w[1], w[2], w[3]])).collect())
        }
        Format::Yiq422 | Format::Ayiq8422 => {
            let b = take(80)?;
            let v: Vec<i16> = b.chunks(2).map(|w| i16::from_be_bytes([w[0], w[1]])).collect();
            Table::Ncc(ncc_bytes(&v))
        }
        _ => Table::None,
    };
    let n: usize = (h.large_lod..=h.small_lod).map(|l| crate::layout::host_level_bytes(l, h.aspect, h.format)).sum::<Option<usize>>()?;
    let mut data = take(n)?.to_vec();
    if h.format.texel().bits() == 16 {
        for p in data.chunks_mut(2) {
            p.swap(0, 1);
        }
    }
    Some((table, data))
}

/// A `GuNccTable` from 40 values (16 Y, 12 I, 12 Q): `yRGB[16]` bytes, `iRGB[4][3]` and
/// `qRGB[4][3]` 16-bit words, then `packed_data[12]`, the same table in the layout of the
/// TMU's NCC registers. Y packs four to a word; each I and Q entry packs its 9-bit R, G and B
/// at bits 18, 9 and 0.
pub fn ncc_bytes(v: &[i16]) -> Vec<u8> {
    let mut b = Vec::with_capacity(112);
    b.extend(v[..16].iter().map(|y| *y as u8));
    for w in &v[16..40] {
        b.extend_from_slice(&w.to_le_bytes());
    }
    let mut packed = [0u32; 12];
    for k in 0..4 {
        packed[k] = (0..4).map(|j| (v[4 * k + j] as u8 as u32) << (8 * j)).sum();
    }
    let rgb = |e: &[i16]| (e[0] as u32 & 0x1ff) << 18 | (e[1] as u32 & 0x1ff) << 9 | e[2] as u32 & 0x1ff;
    for k in 0..4 {
        packed[4 + k] = rgb(&v[16 + 3 * k..]);
        packed[8 + k] = rgb(&v[28 + 3 * k..]);
    }
    for w in packed {
        b.extend_from_slice(&w.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use voodoo::ncc::NccTable;

    fn file(head: &str, body: &[u8]) -> Vec<u8> {
        let mut f = head.as_bytes().to_vec();
        f.extend_from_slice(body);
        f
    }

    #[test]
    fn rgb565_chain() {
        // 4x2 down to 1x1 at 2:1: levels 4x2, 2x1, 1x1, 16-bit big-endian.
        let body: Vec<u8> = (0..11u16).flat_map(|i| (0x1100 + i).to_be_bytes()).collect();
        let f = file("3df v1.0\nrgb565\nlod range: 1 4\naspect ratio: 2 1\n", &body);
        let h = header(&f).unwrap();
        assert_eq!((h.width, h.height, h.small_lod, h.large_lod, h.aspect, h.format), (4, 2, 8, 6, 2, Format::Rgb565));
        let (t, data) = load(&f, &h).unwrap();
        assert_eq!(t, Table::None);
        assert_eq!(data.len(), (8 + 2 + 1) * 2);
        assert_eq!(&data[..4], &[0x00, 0x11, 0x01, 0x11]);
        // One byte short is a failure.
        assert!(load(&f[..f.len() - 1], &h).is_none());
    }

    #[test]
    fn palette_and_bad_headers() {
        let mut body = Vec::new();
        for i in 0..256u32 {
            body.extend_from_slice(&(i * 0x010101).to_be_bytes());
        }
        body.extend_from_slice(&[7]);
        let f = file("3DF v1.0\nP8\nlod range: 1 1\naspect ratio: 1 1\n", &body);
        assert!(header(&f).is_none(), "the magic is lower case");
        let f = file("3df v1.0\nP8\nlod range: 1 1\naspect ratio: 1 1\n", &body);
        let h = header(&f).unwrap();
        let (t, data) = load(&f, &h).unwrap();
        assert_eq!(data, vec![7]);
        match t {
            Table::Palette(p) => assert_eq!((p[1], p[255]), (0x010101, 0xffffff)),
            _ => panic!("no palette"),
        }
        assert!(header(b"3df v1.0\nrgb565\nlod range: 3 256\naspect ratio: 1 1\n").is_none());
        assert!(header(b"3df v1.0\nrgb999\nlod range: 1 256\naspect ratio: 1 1\n").is_none());
    }

    #[test]
    fn ncc_table_decodes() {
        let mut v: Vec<i16> = (0..16).map(|i| i * 16).collect();
        v.extend([10, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        v.extend([0, 0, 5, 0, 0, 0, 0, 0, 0, -256, 255, 0]);
        let b = ncc_bytes(&v);
        assert_eq!(b.len(), 112);
        let t = NccTable::from_gu_bytes(&b).unwrap();
        // Y 15, I 0, Q 0: 240 + (10, -10, 0) + (0, 0, 5).
        assert_eq!(t.decode(0xf0), [250, 230, 245]);
        // Q 3 is (-256, 255, 0): clamped.
        assert_eq!(t.decode(0x13), [0, 255, 16]);
        // packed_data: Y 0-3, then I[0] = (10, -10, 0) as 9-bit fields.
        assert_eq!(u32::from_le_bytes(b[64..68].try_into().unwrap()), 0x3020_1000);
        assert_eq!(u32::from_le_bytes(b[80..84].try_into().unwrap()), 10 << 18 | (0x1ff & -10i32 as u32) << 9);
    }
}
