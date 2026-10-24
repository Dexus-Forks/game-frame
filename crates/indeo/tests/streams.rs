//! The decoder on streams made here, with tables made here: no Intel file is read. The rules
//! the streams check are Intel's (POD's docs/formats/video.md): an empty intra macroblock is
//! filled at the DC predictor, a coded DC is a difference from the last intra block's, the
//! Slant transform of a lone DC is flat, a forward macroblock copies the reference from where
//! its vector points, and an empty macroblock of a predicted picture copies it unmoved.

use indeo::{Decoder, Frame, Tables};

/// Bits in Intel's order: a field's first bit is the lowest unread bit of the next byte.
#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    bits: usize,
}

impl Writer {
    fn put(&mut self, n: u32, v: u32) {
        for i in 0..n {
            if self.bits % 8 == 0 {
                self.bytes.push(0);
            }
            let b = (v >> i) & 1;
            *self.bytes.last_mut().unwrap() |= (b as u8) << (self.bits % 8);
            self.bits += 1;
        }
    }

    fn align(&mut self) {
        self.bits = self.bytes.len() * 8;
    }

    /// A symbol of the one-row, 8-bit codebook: the 8 bits as they are.
    fn sym(&mut self, s: u8) {
        self.put(8, s as u32);
    }
}

/// The tables: both codebooks one row of 8 bits; one run/value map with only run 0 and
/// 127 levels and the identity permutation (so symbol 0 ends a block, 1 escapes, 128 + k is
/// level +k and 129 − k is level −k); the zigzag-free row-by-row scan; every step 1.
fn tables() -> Tables {
    let book = vec![vec![1, 8]; 8];
    let mut perm = [0u8; 256];
    for (i, p) in perm.iter_mut().enumerate() {
        *p = i as u8;
    }
    let scan: [u8; 64] = std::array::from_fn(|i| i as u8);
    let mut scan4 = [0u8; 64];
    for i in 0..16 {
        scan4[i] = (i / 4 * 8 + i % 4) as u8;
    }
    let scans = vec![scan, scan, scan, scan, scan, scan4, scan4];
    Tables::from_parts(book.clone(), book, vec![(vec![127], perm)], scans, vec![1; 2 * 32 * 64])
}

/// Macroblock codebook values: 0, +1, −1, +2, −2, … are symbols 0, 1, 2, 3, 4, …
fn mb_sym(v: i32) -> u8 {
    if v > 0 { (2 * v - 1) as u8 } else { (-2 * v) as u8 }
}

/// A 32 × 32 picture: one luma band of 16 × 16 macroblocks (Slant 8 × 8), one 4 × 4-macroblock
/// band in each chroma plane (Slant 4 × 4), one tile each.
fn picture(kind: u32, luma: impl Fn(&mut Writer), chroma: impl Fn(&mut Writer)) -> Vec<u8> {
    let mut w = Writer::default();
    w.put(18, 0x3FFF8);
    w.put(3, kind);
    w.put(2, 0);
    w.put(1, 0); // no data size
    w.put(1, 0); // not locked
    w.put(3, 7);
    w.put(16, 32);
    w.put(16, 32);
    w.put(1, 0); // one tile
    w.put(2, 0); // chroma a quarter each way
    w.put(2, 3); // one luma band
    w.put(2, 3); // one chroma band
    w.put(1, 0); // no picture number
    w.put(1, 0);
    for _ in 0..2 {
        // Both codebooks: the stream's own, one row of 8 bits.
        w.put(1, 1);
        w.put(3, 7);
        w.put(4, 1);
        w.put(4, 8);
    }
    w.put(1, 1);
    w.put(3, 0); // run/value map 0
    w.put(1, 0);
    w.put(1, 0); // a quantiser delta only for coded macroblocks
    w.put(5, 0);
    w.put(1, 0); // not held back
    w.put(1, 0);
    w.put(1, 0);
    w.put(1, 0);
    w.align();
    for plane in 0..3 {
        let mut band = Writer::default();
        band.put(2, plane);
        band.put(4, 0);
        band.put(1, 0);
        band.put(1, 0);
        band.put(2, 0); // whole-sample vectors
        band.put(1, 0);
        band.put(2, if plane == 0 { 0 } else { 2 });
        band.put(1, 0);
        band.put(1, 0);
        band.put(5, 0); // band quantiser 0
        band.put(1, 0); // the transform follows
        band.put(5, if plane == 0 { 4 } else { 11 });
        band.put(4, if plane == 0 { 0 } else { 5 });
        band.put(5, 0);
        band.put(1, 0);
        band.put(1, 0);
        band.put(1, 0);
        band.align();
        let mut mbs = Writer::default();
        if plane == 0 { luma(&mut mbs) } else { chroma(&mut mbs) }
        // The tile: not empty, a size, then the macroblocks and the blocks.
        let size = 2 + mbs.bytes.len();
        band.put(1, 0);
        band.put(1, 1);
        band.put(8, size as u32);
        band.align();
        w.bytes.extend(&band.bytes);
        w.bytes.extend(&mbs.bytes);
        w.bits = w.bytes.len() * 8;
    }
    w.bytes
}

/// Every macroblock of a chroma band empty (the 8 × 8 plane has four of 4 × 4), and no blocks.
fn empty_chroma(w: &mut Writer) {
    for _ in 0..4 {
        w.put(1, 1);
    }
    w.align();
}

fn decode(dec: &mut Decoder, chunk: &[u8]) -> Frame {
    dec.decode(chunk).expect("decodes").expect("a picture to show")
}

/// The block at (bx, by) in 8 × 8 blocks.
fn block(f: &Frame, bx: usize, by: usize) -> Vec<u8> {
    (0..8).flat_map(|y| (0..8).map(move |x| (y, x))).map(|(y, x)| f.y[(by * 8 + y) * f.width + bx * 8 + x]).collect()
}

#[test]
fn an_empty_intra_picture_is_mid_grey() {
    let mut dec = Decoder::new(tables());
    let chunk = picture(
        0,
        |w| {
            for _ in 0..4 {
                w.put(1, 1);
            }
            w.align();
        },
        empty_chroma,
    );
    let f = decode(&mut dec, &chunk);
    assert_eq!((f.width, f.height), (32, 32));
    assert!(f.y.iter().all(|&p| p == 128), "{:?}", &f.y[..8]);
    assert!(f.v.iter().chain(&f.u).all(|&p| p == 128));
}

/// Macroblock 0 codes its top-left block's DC as +40; the rest of the picture is uncoded and
/// takes the predictor. A lone DC through the Slant transform is a flat block, and its value is
/// the one the fill gives the same predictor.
#[test]
fn the_dc_predictor_carries_from_block_to_block() {
    let mut dec = Decoder::new(tables());
    let chunk = picture(
        0,
        |w| {
            w.put(1, 0); // coded
            w.put(4, 1); // only the top-left block
            w.sym(mb_sym(0)); // quantiser delta
            for _ in 0..3 {
                w.put(1, 1);
            }
            w.align();
            w.sym(128 + 40); // DC +40, run 0
            w.sym(0); // end of block
            w.align();
        },
        empty_chroma,
    );
    let f = decode(&mut dec, &chunk);
    let first = block(&f, 0, 0);
    assert!(first.iter().all(|&p| p == first[0]), "flat: {first:?}");
    // The fill: 0x4000 + ((2 × 40 + 2) & !3) = 0x4050, sample / 4 − 0xF80 = 148.
    assert_eq!(first[0], 148);
    assert!(f.y.iter().all(|&p| p == 148));
}

/// A predicted picture whose macroblocks point 8 samples left, then a picture of empty
/// macroblocks, which copy their reference unmoved.
#[test]
fn forward_macroblocks_copy_the_reference() {
    let mut dec = Decoder::new(tables());
    // An intra picture with a DC of its own in each macroblock: +16, then +8 on the last.
    let intra = picture(
        0,
        |w| {
            for d in [16, 8, 8, 8] {
                w.put(1, 0);
                w.put(4, 0xF);
                w.sym(mb_sym(0));
                let _ = d;
            }
            w.align();
            for d in [16, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0] {
                if d != 0 {
                    w.sym(128 + d);
                }
                w.sym(0);
            }
            w.align();
        },
        empty_chroma,
    );
    let i = decode(&mut dec, &intra);
    let px = |f: &Frame, x: usize, y: usize| f.y[y * f.width + x];
    let (a, b) = (px(&i, 0, 0), px(&i, 16, 0));
    assert_ne!(a, b);
    // Each macroblock forward, no blocks coded, vector (0, +8): from 8 samples to the right.
    let shifted = picture(
        2,
        |w| {
            let mut pred = 0;
            for _ in 0..4 {
                w.put(1, 0);
                w.put(1, 1); // forward
                w.put(4, 0);
                w.sym(mb_sym(0)); // vertical, a difference from the last vector
                w.sym(mb_sym(8 - pred)); // horizontal
                pred = 8;
            }
            w.align();
        },
        empty_chroma,
    );
    let p = decode(&mut dec, &shifted);
    for y in 0..32 {
        for x in 0..24 {
            assert_eq!(px(&p, x, y), px(&i, x + 8, y), "({x}, {y})");
        }
    }
    let same = picture(
        2,
        |w| {
            for _ in 0..4 {
                w.put(1, 1);
            }
            w.align();
        },
        empty_chroma,
    );
    let q = decode(&mut dec, &same);
    assert_eq!(q.y, p.y);
}
