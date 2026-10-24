//! The macroblocks of a tile (`FUN_10019540`): what each macroblock says, turned into Intel's
//! list of blocks (`blocks::Entry`). Intel's Pentium path predicts each moving macroblock as it
//! reads it; here the prediction is a pass of its own (`mc`), from the same list.

use super::bits::Bits;
use super::blocks::{CODED, Entry, INTRA, SECOND};
use super::huffman::Codebook;

/// What the macroblock layer needs to know about its band.
pub(super) struct MbBand<'a> {
    pub kind: u32,
    pub mb_size: u32,
    pub pitch: u32,
    pub quant: i32,
    pub inherit_mv: bool,
    pub inherit_quant: bool,
    /// The luma band 0 reads a quantiser delta for every macroblock (picture flag +0x98).
    pub quant_every_mb: bool,
    pub book: &'a Codebook,
    /// For a band that inherits: the luma band 0's list for this tile, and its macroblock size
    /// and whether its rows end in a padding block.
    pub reference: Option<(&'a [Entry], u32, bool)>,
}

/// Intel's map from a macroblock codebook symbol to a signed value (0x100AFE80, built at
/// start-up): 0, +1, −1, +2, −2, …, biased by 0x80.
fn biased(sym: u8) -> u32 {
    let s = sym as u32;
    (if s & 1 == 1 { 0x80 + s.div_ceil(2) } else { 0x80 - s / 2 }) & 0xFF
}

/// The macroblock types (0x10077210): intra, forward, backward, both.
const TYPES: [u32; 4] = [INTRA, 4, 8, 0xC];

/// Halve or quarter the luma's vectors for a band of smaller macroblocks (`FUN_10019540`, the
/// inherited vectors), on all four biased bytes at once.
fn scale_mv(v: u32, diff: i32) -> u32 {
    match diff {
        -1 => ((v & 0x8080_8080) >> 7 & v).wrapping_add(0x4040_4040).wrapping_add((v & 0xFEFE_FEFE) >> 1),
        -2 => ((((v & 0x8080_8080) >> 7).wrapping_add(v).wrapping_add(0x0101_0101) >> 2) & 0x3F3F_3F3F)
            .wrapping_add(0x6060_6060),
        _ => v,
    }
}

/// Read the macroblocks of a tile of `rows` × `cols` macroblocks starting at byte `offset` of
/// the band (`r` at the tile's macroblock data). Returns the block list.
pub(super) fn decode(r: &mut Bits, band: &MbBand, rows: u32, cols: u32, offset: u32) -> Vec<Entry> {
    let mb = band.mb_size;
    let mb16 = mb == 16;
    let base = band.pitch | if mb == 4 { 0x10 } else { 0 };
    // Inter pictures (kinds 2 to 4) read a type: 1 bit, 2 for the bidirectional kind 3
    // (0x10064870). A band that inherits its vectors reads none.
    let type_bits = match band.kind {
        2 | 4 => 1,
        3 => 2,
        _ => 0,
    };
    let type_bits = if band.inherit_mv { 0 } else { type_bits };
    let cbp_bits = if mb16 { 4 } else { 1 };
    let mut out = Vec::new();
    let mut pred = 0x8080u32;
    let mut second = false;
    let (refs, ref_mb, ref_pad) = band.reference.map_or((&[][..], 0, false), |(e, m, p)| (e, m, p));
    let ref_step = if ref_mb == 16 { 4 } else { 1 };
    let diff = (mb >> 3) as i32 - (ref_mb >> 3) as i32;
    let mut ref_at = 0usize;
    for row in 0..rows {
        for col in 0..cols {
            let dest = offset + row * band.pitch * mb + col * 2 * mb;
            let reference = refs.get(ref_at).copied().unwrap_or_default();
            let mut delta = 0i32;
            let mut flags;
            let mut mv = 0u32;
            let mut q = band.quant;
            let mut cbp = 0u32;
            if r.flag() {
                // An empty macroblock, no coefficients: intra in an intra picture, else forward
                // with no motion (0x10064858 through 0x10077210).
                if matches!(band.kind, 2..=4) {
                    flags = 4;
                    mv = 0x8080;
                } else {
                    flags = INTRA;
                }
                if band.quant_every_mb {
                    delta = biased(band.book.read(r)) as i32 - 0x80;
                }
            } else {
                let t = r.get(type_bits) as usize;
                flags = TYPES[t];
                cbp = r.get(cbp_bits);
                if band.inherit_mv {
                    flags = reference.flags & 0xE;
                }
                if band.inherit_quant {
                    let j = (reference.flags >> 26) as i32;
                    delta = (band.quant + j - 32).clamp(0, 31) - band.quant;
                } else if cbp != 0 || band.quant_every_mb {
                    delta = biased(band.book.read(r)) as i32 - 0x80;
                }
                q = band.quant + delta + if flags & 0xC == 0 { 32 } else { 0 };
                if flags & 0xC != 0 {
                    if band.inherit_mv {
                        mv = scale_mv(reference.mv, diff);
                    } else {
                        let v1 = biased(band.book.read(r));
                        let v2 = biased(band.book.read(r));
                        mv = pred.wrapping_sub(0x8080).wrapping_add(v1 << 8 | v2);
                        pred = mv;
                        if flags & 0xC == 0xC {
                            let v3 = biased(band.book.read(r));
                            let v4 = biased(band.book.read(r));
                            pred = mv.wrapping_sub(0x8080).wrapping_add(v3 << 8 | v4);
                            mv = mv.wrapping_add(pred << 16);
                        }
                    }
                    if flags & 4 == 0 {
                        mv |= 0x80_0000;
                    }
                }
            }
            let qfield = ((delta + 32) as u32 & 63) << 26;
            let blocks: &[(u32, u32)] = if mb16 { &[(0, 0), (1, 0), (0, 1), (1, 1)] } else { &[(0, 0)] };
            for (k, &(bx, by)) in blocks.iter().enumerate() {
                let coded = (cbp >> k) & 1;
                out.push(Entry {
                    dest: dest + bx * 16 + by * 8 * band.pitch,
                    mv,
                    q,
                    flags: base | flags | coded | qfield | if second { SECOND } else { 0 },
                });
                second = !second;
            }
            ref_at += ref_step;
        }
        if second {
            // Rows of an odd number of blocks end in an uncoded intra block, so that pairs
            // never cross rows.
            out.push(Entry { dest: offset + row * band.pitch * mb + cols * 2 * mb, mv: 0, q: 0, flags: base | INTRA | SECOND });
            second = false;
            if ref_pad {
                ref_at += 1;
            }
        }
    }
    let _ = CODED;
    out
}
