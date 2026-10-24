//! The picture header (`FUN_100208d0`) and the band header (`FUN_100188d0`), field by field in
//! Intel's order (POD's docs/formats/video.md, "The picture header", "The band header").

use super::bits::Bits;
use super::Error;

/// Picture sizes by the 3-bit size code (heights 0x10064930, widths 0x10064908).
const HEIGHTS: [u32; 7] = [480, 240, 120, 480, 240, 288, 144];
const WIDTHS: [u32; 7] = [640, 320, 160, 704, 352, 352, 176];

/// A codebook choice: one of Intel's eight descriptors, or one the stream gives.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum BookChoice {
    Table(usize),
    Custom(Vec<u8>),
}

/// One band's place in a plane: the halvings on its path through the band tree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct BandShape {
    pub height_shift: u32,
    pub width_shift: u32,
}

#[derive(Clone, Debug)]
pub(super) struct PictureHeader {
    /// 0 intra, 1 to 4 the predicted kinds, 5 and 6 null (no picture data).
    pub kind: u32,
    /// Bit 0 of the 2-bit field after the kind (instance +0xA0; transparency, not in POD's
    /// videos).
    #[allow(dead_code)]
    pub flags: u32,
    /// The 24-bit size of the picture's data (instance +0x38), 0 when absent.
    pub data_size: u32,
    pub height: u32,
    pub width: u32,
    pub tile_height: Option<u32>,
    pub tile_width: Option<u32>,
    /// Chroma plane divisors (height, width).
    pub chroma_div: (u32, u32),
    pub luma_bands: Vec<BandShape>,
    pub chroma_bands: Vec<BandShape>,
    /// The 20-bit picture number (instance +0x44), if present.
    pub number: Option<u32>,
    pub mb_book: BookChoice,
    pub block_book: BookChoice,
    /// The run/value map for bands that do not choose their own (8 when absent).
    pub rv_map: usize,
    /// Instance +0x9C and +0x98: the second decides whether the luma band 0 reads a quantiser
    /// delta for every macroblock.
    pub flag_27: bool,
    pub quant_every_mb: bool,
    /// Instance +0x4C, 5 bits.
    pub field_13: u32,
    /// Instance +0x94 (3 bits): non-zero for a picture held back for later display.
    pub deferred: u32,
    /// Bytes the header takes.
    pub len: usize,
}

fn band_tree(r: &mut Bits, node: &mut [(u32, u32); 20]) -> Vec<BandShape> {
    // Intel's walk (FUN_100208d0): one open node at the start; codes 0, 1 and 2 halve the
    // width, the height or both and open 1, 1 and 3 more nodes; code 3 is a band, whose shape is
    // the halvings so far, and undoes its node's halving. The chroma walk reuses the luma
    // walk's nodes, as Intel's does.
    const OPENS: [i32; 4] = [1, 1, 3, -1];
    const HALF_W: [u32; 4] = [1, 0, 1, 0];
    const HALF_H: [u32; 4] = [0, 1, 1, 0];
    let (mut open, mut hs, mut ws) = (1i32, 0u32, 0u32);
    let mut bands = Vec::new();
    while open != 0 && open < 20 {
        let c = r.get(2) as usize;
        let slot = open as usize;
        if c == 3 {
            bands.push(BandShape { height_shift: hs, width_shift: ws });
            hs = hs.wrapping_sub(node[slot].0);
            ws = ws.wrapping_sub(node[slot].1);
            node[slot] = (0, 0);
        } else {
            node[slot] = (HALF_H[c], HALF_W[c]);
            hs += HALF_H[c];
            ws += HALF_W[c];
        }
        open += OPENS[c];
    }
    bands
}

fn book(r: &mut Bits) -> Option<BookChoice> {
    if !r.flag() {
        return None;
    }
    let i = r.get(3) as usize;
    if i != 7 {
        return Some(BookChoice::Table(i));
    }
    let n = r.get(4) as usize;
    let mut d = vec![n as u8];
    d.extend((0..n).map(|_| r.get(4) as u8));
    Some(BookChoice::Custom(d))
}

impl PictureHeader {
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let mut r = Bits::new(data);
        if r.get(18) != 0x3FFF8 {
            return Err(Error::Picture("no picture start code"));
        }
        let kind = r.get(3);
        if kind == 7 {
            return Err(Error::Picture("picture kind 7"));
        }
        let flags = r.get(2);
        if flags & 2 != 0 {
            return Err(Error::Picture("reserved picture flag"));
        }
        let data_size = if r.flag() { r.get(24) } else { 0 };
        let mut h = Self {
            kind,
            flags: flags & 1,
            data_size,
            height: 0,
            width: 0,
            tile_height: None,
            tile_width: None,
            chroma_div: (1, 1),
            luma_bands: Vec::new(),
            chroma_bands: Vec::new(),
            number: None,
            mb_book: BookChoice::Table(7),
            block_book: BookChoice::Table(7),
            rv_map: 8,
            flag_27: false,
            quant_every_mb: false,
            field_13: 0,
            deferred: 0,
            len: 0,
        };
        if kind >= 5 {
            h.len = r.bytes_used();
            return Ok(h);
        }
        if r.flag() {
            return Err(Error::Picture("locked picture"));
        }
        let size = r.get(3) as usize;
        (h.height, h.width) = if size == 7 { (r.get(16), r.get(16)) } else { (HEIGHTS[size], WIDTHS[size]) };
        if r.flag() {
            // Tile sizes (n + 1) × 32 (0x10064D10); 15 is the picture's own size.
            let th = r.get(4);
            let tw = r.get(4);
            h.tile_height = (th != 15).then_some((th + 1) * 32);
            h.tile_width = (tw != 15).then_some((tw + 1) * 32);
        }
        h.chroma_div = match r.get(2) {
            // Width divisors 0x10064958, height divisors 0x10064978; 3 gives both (1 to 4).
            0 => (4, 4),
            1 => (2, 2),
            2 => (1, 2),
            _ => {
                let w = r.get(2) + 1;
                let hh = r.get(2) + 1;
                (hh, w)
            }
        };
        let mut node = [(0u32, 0u32); 20];
        h.luma_bands = band_tree(&mut r, &mut node);
        h.chroma_bands = band_tree(&mut r, &mut node);
        if r.flag() {
            h.number = Some(r.get(20));
        }
        if r.flag() {
            r.skip(8);
        }
        if let Some(b) = book(&mut r) {
            h.mb_book = b;
        }
        if let Some(b) = book(&mut r) {
            h.block_book = b;
        }
        if r.flag() {
            h.rv_map = r.get(3) as usize;
        }
        h.flag_27 = r.flag();
        h.quant_every_mb = r.flag();
        h.field_13 = r.get(5);
        if r.flag() {
            h.deferred = r.get(3);
        }
        if r.flag() {
            r.skip(16);
        }
        while r.flag() {
            r.skip(8);
        }
        if r.flag() {
            // One bit for each 64 × 64 square of the picture.
            r.skip((h.width.div_ceil(64) * h.height.div_ceil(64)) as usize);
        }
        h.len = r.bytes_used();
        Ok(h)
    }
}

/// What a band header sets. Fields marked as kept are carried over from the band's last
/// picture when the header leaves them out (Intel keeps them in the band's structure).
#[derive(Clone, Debug, Default)]
pub(super) struct BandHeader {
    pub plane: usize,
    pub band: usize,
    pub empty: bool,
    /// Band +0x18: a 16-bit value (0 when absent).
    pub field_06: u32,
    /// Band +0x1C: the motion vectors' resolution (0 whole samples, else halves).
    pub mv_res: u32,
    /// Band +0x20: a 16-bit value, -1 when absent.
    pub field_08: i32,
    /// The macroblock size (16, 8 or 4; 0x10064820).
    pub mb_size: u32,
    /// Band +0x2C and +0x28: this band takes its motion and its quantiser deltas from the
    /// luma band 0's macroblocks.
    pub inherit_mv: bool,
    pub inherit_quant: bool,
    /// Band +0x34: the band's quantiser, 0 to 31.
    pub quant: u32,
    /// Kept: the transform (+0x38), scan (+0x39) and quantiser matrix (+0x3B).
    pub transform: Option<(u8, u8, u8)>,
    pub block_book: Option<BookChoice>,
    pub rv_map: Option<usize>,
    /// Pairs of symbols whose runs and levels swap (+0x194).
    pub corrections: Vec<(u8, u8)>,
    pub len: usize,
}

impl BandHeader {
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let mut r = Bits::new(data);
        let mut b = BandHeader { plane: r.get(2) as usize, band: r.get(4) as usize, ..Default::default() };
        b.empty = r.flag();
        if b.empty {
            b.len = r.bytes_used();
            return Ok(b);
        }
        b.field_06 = if r.flag() { r.get(16) } else { 0 };
        b.mv_res = r.get(2);
        b.field_08 = if r.flag() { r.get(16) as i32 } else { -1 };
        b.mb_size = match r.get(2) {
            0 => 16,
            1 => 8,
            2 => 4,
            _ => return Err(Error::Picture("macroblock size code 3")),
        };
        b.inherit_mv = r.flag();
        b.inherit_quant = r.flag();
        b.quant = r.get(5);
        if !r.flag() {
            let transform = r.get(5) as u8;
            let scan = r.get(4) as u8;
            if scan == 15 {
                return Err(Error::Picture("custom scan order (not in POD's videos)"));
            }
            let matrix = r.get(5) as u8;
            if matrix == 31 {
                return Err(Error::Picture("custom quantiser matrix (not in POD's videos)"));
            }
            b.transform = Some((transform, scan, matrix));
        }
        b.block_book = book(&mut r);
        if r.flag() {
            b.rv_map = Some(r.get(3) as usize);
        }
        if r.flag() {
            let n = r.get(8);
            if n > 61 {
                return Err(Error::Picture("more than 61 corrections"));
            }
            b.corrections = (0..n).map(|_| (r.get(8) as u8, r.get(8) as u8)).collect();
        }
        b.len = r.bytes_used();
        Ok(b)
    }
}
