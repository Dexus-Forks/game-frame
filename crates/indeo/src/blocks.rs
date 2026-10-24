//! The blocks of a tile (`FUN_10001c90` and the routines its band context names): each block's
//! coefficients, the DC prediction, the inverse transform and the store into the band. Intel's
//! Pentium path decodes blocks in pairs (a block and its right-hand neighbour, one in each half
//! of a 32-bit word), and some of its stores add whole words, so the pairs are kept here.

use super::bits::Bits;
use super::huffman::Codebook;
use super::swar;
use super::tables::{RunValueMap, Tables};
use super::Error;

/// A block as the macroblock layer lists it: Intel's 16-byte record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Entry {
    /// Byte offset in the band (two bytes a sample, rows `pitch` bytes apart).
    pub dest: u32,
    /// Motion: the forward vector in the low 16 bits, a second (backward) one in the high 16;
    /// each a vertical byte over a horizontal one, both biased by 0x80.
    pub mv: u32,
    /// The quantiser: an index into the context's 64 tables (32 inter levels, then 32 intra).
    pub q: i32,
    /// Bit 0 coded, 1 intra, 2 forward, 3 backward, 4 a 4 × 4 block, 5..15 the pitch, 24 the
    /// right-hand block of a pair, 26..31 the quantiser delta + 32.
    pub flags: u32,
}

pub(super) const CODED: u32 = 1;
pub(super) const INTRA: u32 = 2;
pub(super) const SECOND: u32 = 0x100_0000;

/// What the transform number selects (Intel's descriptors at 0x1006A178, 0x38 bytes each).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Transform {
    Haar8x8,
    Haar1x8,
    Haar8x1,
    None8x8,
    Slant8x8,
    Slant4x4,
}

impl Transform {
    pub fn from_number(n: u8) -> Result<Self, Error> {
        Ok(match n {
            0 => Self::Haar8x8,
            1 => Self::Haar1x8,
            2 => Self::Haar8x1,
            3 => Self::None8x8,
            4 => Self::Slant8x8,
            11 => Self::Slant4x4,
            _ => return Err(Error::Picture("a transform POD's videos do not use")),
        })
    }

    pub fn size(self) -> usize {
        if self == Self::Slant4x4 { 4 } else { 8 }
    }

    /// Where an uncoded intra block takes the fill value; elsewhere it holds 0x4000. The
    /// one-way and no-transform fills (0x100165B0, 0x10016450, 0x10016BC0) set only the samples
    /// their transform's DC reaches: row 0, column 0, the first sample.
    fn filled(self, row: usize, col: usize) -> bool {
        match self {
            Self::Haar1x8 => row == 0,
            Self::Haar8x1 => col == 0,
            Self::None8x8 => row == 0 && col == 0,
            _ => true,
        }
    }

    /// The bias in each half of a coefficient word (descriptor data +4: 0x10064390 and
    /// 0x100645E0 hold 0x08000800, 0x10064000 holds 0x40004000).
    fn bias(self) -> u32 {
        match self {
            Self::Slant8x8 | Self::Slant4x4 => 0x0800,
            _ => 0x4000,
        }
    }

    fn run(self, m: &mut [u32]) {
        match self {
            Self::Haar8x8 => swar::haar_8x8(m),
            Self::Haar1x8 => swar::haar_1x8(m),
            Self::Haar8x1 => swar::haar_8x1(m),
            Self::None8x8 => swar::none_8x8(m),
            Self::Slant8x8 => swar::slant_8x8(m),
            Self::Slant4x4 => swar::slant_4x4(m),
        }
    }

    /// The word an uncoded intra block is filled with, from the DC predictor (the handler the
    /// descriptor names second: 0x10016780, 0x10017050, 0x10016340, 0x100165B0, 0x10016450,
    /// 0x10016BC0).
    fn fill(self, pred: u32) -> u32 {
        let v = match self {
            Self::Slant8x8 | Self::Slant4x4 => pred.wrapping_mul(2).wrapping_add(2) & !3,
            Self::Haar8x8 => (((pred & !7) as i32) >> 1) as u32,
            Self::Haar1x8 | Self::Haar8x1 => pred & !3,
            Self::None8x8 => return pred.wrapping_mul(4).wrapping_add(0x4000_4000).wrapping_add(pred << 18),
        };
        0x4000_4000u32.wrapping_add(v).wrapping_add(v << 16)
    }
}

/// Intel's dequantisation tables (`FUN_100014e0`): for each step s from 1 to 512, the values of
/// levels +1, −1, +2, −2, … (as many as 8192 / s pairs): k·s + s/2, less 1 when s is odd and
/// over 1. One pool, as Intel allocates it, so that a level past a table's end reads the next.
pub(super) struct Dequant {
    pool: Vec<i32>,
    start: Vec<usize>,
}

impl Dequant {
    pub fn new() -> Self {
        let (mut pool, mut start) = (Vec::new(), vec![0; 513]);
        for s in 1..=512i32 {
            start[s as usize] = pool.len();
            let adj = if s == 1 { 0 } else { s & 1 };
            for k in 1..=8192 / s {
                let v = k * s + (s >> 1) - adj;
                pool.push(v);
                pool.push(-v);
            }
        }
        Self { pool, start }
    }

    /// The value of level code `zz` (1 → +1, 2 → −1, 3 → +2, …) at step `s`.
    fn value(&self, s: u16, zz: u32) -> u32 {
        let i = self.start[s.clamp(1, 512) as usize] as isize + zz as isize - 1;
        if i < 0 { 0 } else { *self.pool.get(i as usize).unwrap_or(&0) as u32 }
    }
}

/// A band's decoding context (`FUN_1000a520` and what it builds): the codebook, the expanded
/// run/value map, the scan and the quantiser tables, for its transform.
pub(super) struct Context {
    pub transform: Transform,
    book: Codebook,
    eob: u8,
    esc: u8,
    run: [u8; 256],
    level: [u32; 256],
    /// Scan position → coefficient word (9 words a row).
    place: [u8; 64],
    /// For each of the 64 quantiser tables, the step at each scan position.
    steps: Vec<[u16; 64]>,
}

impl Context {
    pub fn new(
        tables: &Tables,
        transform: Transform,
        scan: usize,
        matrix: usize,
        book: Codebook,
        rv: &RunValueMap,
        corrections: &[(u8, u8)],
    ) -> Result<Self, Error> {
        let scan = tables.scans.get(scan).ok_or(Error::Picture("scan order out of range"))?;
        if matrix >= tables.quant.len() / (2 * 32 * 64) {
            return Err(Error::Picture("quantiser matrix out of range"));
        }
        // FUN_100015d0: the symbols the permutation sends to 0 and 1 end the block and escape;
        // then each run's levels −n … −1 (codes 2n … 2) and +1 … +n (codes 1 … 2n − 1).
        let mut inverse = [0u8; 256];
        for (sym, &c) in rv.permutation.iter().enumerate() {
            inverse[c as usize] = sym as u8;
        }
        let (mut run, mut level) = ([0u8; 256], [0u32; 256]);
        let mut c = 2;
        for (r, &n) in rv.run_counts.iter().enumerate() {
            for k in (1..=n as u32).rev() {
                let s = inverse[c] as usize;
                (run[s], level[s]) = (r as u8 + 1, 2 * k);
                c += 1;
            }
            for k in 1..=n as u32 {
                let s = inverse[c] as usize;
                (run[s], level[s]) = (r as u8 + 1, 2 * k - 1);
                c += 1;
            }
        }
        // The band's corrections swap two symbols' runs and levels (FUN_10001c90, 0x10001EE3).
        for &(a, b) in corrections {
            run.swap(a as usize, b as usize);
            level.swap(a as usize, b as usize);
        }
        let place = std::array::from_fn(|i| scan[i] + (scan[i] >> 3));
        let steps = (0..64)
            .map(|t| {
                let (set, q) = (t / 32, t % 32);
                let base = ((matrix * 2 + set) * 32 + q) * 64;
                std::array::from_fn(|p| tables.quant[base + scan[p] as usize])
            })
            .collect();
        Ok(Self { transform, book, eob: inverse[0], esc: inverse[1], run, level, place, steps })
    }

    /// Decode one block's coefficients into a half of the coefficient words (0x10002462 for
    /// the low half, 0x100025AF for the high half).
    fn coefficients(&self, r: &mut Bits, dq: &Dequant, q: i32, half: u32, m: &mut [u32]) {
        let steps = &self.steps[q.clamp(0, 63) as usize];
        // An end of block is seen only through Intel's fast table, which holds codes of up to
        // 10 bits (FUN_100019f0); a longer one is taken as an ordinary symbol.
        let eob_fast = self.book.len(self.eob) <= 10;
        let mut pos = 0xFFu8;
        loop {
            let sym = self.book.read(r);
            let zz;
            if sym == self.esc {
                let run = self.book.read(r);
                let lo = self.book.read(r) as u32;
                let hi = self.book.read(r) as u32;
                pos = pos.wrapping_add(run).wrapping_add(1);
                zz = hi << 6 | lo;
            } else if sym == self.eob && eob_fast {
                break;
            } else {
                pos = pos.wrapping_add(self.run[sym as usize]);
                zz = self.level[sym as usize];
            }
            // Deviation: a position past 63 reads past Intel's 64-byte scan copy (its own stack);
            // here it wraps.
            let p = (pos & 63) as usize;
            let w = self.place[p] as usize;
            m[w] = m[w].wrapping_add(dq.value(steps[p], zz) << (16 * half));
        }
    }
}

/// The band's samples, in Intel's logical layout: `pitch` bytes a row, two bytes a sample.
pub(super) struct Band {
    pub pitch: u32,
    pub samples: Vec<u16>,
}

impl Band {
    fn at(&self, dest: u32, row: usize, col: usize) -> usize {
        (dest as usize + row * self.pitch as usize) / 2 + col
    }
}

/// Decode a tile's blocks from `data` at byte `start`; return the bytes they take.
pub(super) fn decode_tile(
    ctx: &Context,
    dq: &Dequant,
    entries: &[Entry],
    data: &[u8],
    start: usize,
    band: &mut Band,
) -> usize {
    let mut r = Bits::at(data, start);
    let t = ctx.transform;
    let n = t.size();
    let bias = t.bias();
    let mut pred = 0u32;
    let mut i = 0;
    while i < entries.len() {
        // A run of uncoded blocks: the intra ones are filled at the DC predictor.
        let mut j = i;
        while j < entries.len() && entries[j].flags & CODED == 0 {
            j += 1;
        }
        // Intel writes a whole word where both blocks of a pair in the run are intra; any other
        // block takes the word's low half.
        let fill = t.fill(pred);
        for e in i..j {
            let f = entries[e].flags;
            if f & INTRA == 0 {
                continue;
            }
            let paired = f & SECOND != 0 && e > i && entries[e - 1].flags & (INTRA | SECOND) == INTRA;
            let v = if paired { fill >> 16 } else { fill } as u16;
            for row in 0..n {
                for col in 0..n {
                    let k = band.at(entries[e].dest, row, col);
                    band.samples[k] = if t.filled(row, col) { v } else { 0x4000 };
                }
            }
        }
        if j == entries.len() {
            break;
        }
        // A run of coded blocks, taken in pairs: a pair starts at an even entry.
        let mut k = j;
        while k < entries.len() && entries[k].flags & CODED != 0 {
            k += 1;
        }
        let mut p = if entries[j].flags & SECOND != 0 { j - 1 } else { j };
        while p < k {
            let a = entries[p];
            let b = entries.get(p + 1).copied().unwrap_or_default();
            let mut m = [0u32; 72];
            for row in 0..n {
                m[9 * row..9 * row + n].fill(bias << 16 | bias);
            }
            if p >= j {
                ctx.coefficients(&mut r, dq, a.q, 0, &mut m);
            }
            if p + 1 < k && b.flags & SECOND != 0 {
                ctx.coefficients(&mut r, dq, b.q, 1, &mut m);
            }
            m[0] = dc_prediction(m[0], a.flags & INTRA != 0, b.flags & INTRA != 0, &mut pred, bias);
            t.run(&mut m);
            store(band, a, b, &m, n);
            p += 2;
        }
        i = k;
    }
    r.bytes_used() - start
}

/// The DC prediction (0x1000221E): an intra block's DC is coded as its difference from the
/// last intra block's, in the tile's order.
fn dc_prediction(m0: u32, intra_a: bool, intra_b: bool, pred: &mut u32, bias: u32) -> u32 {
    let ma = if intra_a { !0u32 } else { 0 };
    let mb = if intra_b { !0u32 } else { 0 };
    let lo = (m0 & 0xFFFF).wrapping_sub(bias);
    let hi = (m0 >> 16).wrapping_sub(bias);
    let esi = (*pred & ma).wrapping_add(lo).wrapping_add(bias);
    let edi = pred.wrapping_add(lo & ma);
    let ecx = (mb & edi).wrapping_add(hi).wrapping_add(bias);
    *pred = (hi & mb).wrapping_add(edi);
    ecx << 16 | esi
}

/// Store a transformed pair (0x10015660 for 8 × 8 blocks, 0x10015A60 for 4 × 4): an intra
/// block replaces the band's samples, an inter block adds to them (`out + band − 0x4000`), and
/// where Intel adds whole words the carry between the halves is kept.
fn store(band: &mut Band, a: Entry, b: Entry, m: &[u32], n: usize) {
    #[derive(Clone, Copy, PartialEq)]
    enum Op {
        Keep,
        CopyLo,
        CopyHi,
        CopyBoth,
        AddLo,
        AddHi,
        AddBoth,
        AddLoCopyHi,
        CopyLoAddHi,
    }
    let (fa, fb) = (a.flags, b.flags);
    let both = fa & fb;
    let op = if fb & SECOND == 0 {
        if fa & INTRA != 0 { Op::CopyLo } else { Op::AddLo }
    } else if both & INTRA != 0 {
        if fa & CODED != 0 {
            Op::CopyBoth
        } else if fb & CODED != 0 {
            Op::CopyHi
        } else {
            Op::Keep
        }
    } else if both & CODED != 0 {
        if fb & INTRA != 0 {
            Op::AddLoCopyHi
        } else if fa & INTRA != 0 {
            Op::CopyLoAddHi
        } else {
            Op::AddBoth
        }
    } else if fa & CODED != 0 {
        if fb & INTRA == 0 {
            if fa & INTRA != 0 { Op::CopyLo } else { Op::AddLo }
        } else {
            Op::AddBoth
        }
    } else if fb & CODED != 0 {
        if fb & INTRA != 0 { Op::CopyHi } else { Op::AddHi }
    } else {
        Op::Keep
    };
    if op == Op::Keep {
        return;
    }
    // The right-hand block sits `n` samples on.
    for row in 0..n {
        for col in 0..n {
            let ka = band.at(a.dest, row, col);
            let kb = ka + n;
            let o = m[9 * row + col];
            let w = (band.samples.get(kb).copied().unwrap_or(0) as u32) << 16 | band.samples[ka] as u32;
            let sum_lo = o.wrapping_add(w).wrapping_sub(0x4000_4000);
            let sum_hi = o.wrapping_add(w).wrapping_sub(0x4000_0000) >> 16;
            let (lo, hi) = match op {
                Op::CopyLo => (Some(o), None),
                Op::CopyHi => (None, Some(o >> 16)),
                Op::CopyBoth => (Some(o), Some(o >> 16)),
                Op::AddLo => (Some(sum_lo), None),
                Op::AddHi => (None, Some(sum_hi)),
                Op::AddBoth => (Some(sum_lo), Some(sum_lo >> 16)),
                Op::AddLoCopyHi => (Some(sum_lo), Some(o >> 16)),
                Op::CopyLoAddHi => (Some(o), Some(sum_lo >> 16)),
                Op::Keep => (None, None),
            };
            if let Some(v) = lo {
                band.samples[ka] = v as u16;
            }
            if let Some(v) = hi {
                if let Some(s) = band.samples.get_mut(kb) {
                    *s = v as u16;
                }
            }
        }
    }
}
