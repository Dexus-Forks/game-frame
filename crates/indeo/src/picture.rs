//! The decoder: one AVI chunk at a time (`FUN_10026690`), each holding one picture or two
//! (`FUN_1002c710`), with Intel's three frame stores and its rules for which is the reference,
//! which receives the picture and which is shown (`FUN_1002c4c0`, `FUN_1002c570`,
//! `FUN_1002c620`). Names in brackets are the instance fields (as dword indices) they stand for.

use super::bits::Bits;
use super::blocks::{self, Band, Context, Dequant, Entry, Transform, INTRA};
use super::header::{BandHeader, BandShape, BookChoice, PictureHeader};
use super::huffman::Codebook;
use super::{mb, mc, output, Error, Frame, Tables};

/// The kept part of a band's state (Intel's band structure, zeroed when the layout changes).
#[derive(Clone, Default)]
struct BandState {
    /// Band +0x30: bit 0 the band was empty, bit 1 it was refilled.
    flags: u32,
    transform: (u8, u8, u8),
}

#[derive(Clone, PartialEq, Eq)]
struct Layout {
    height: u32,
    width: u32,
    tile_height: u32,
    tile_width: u32,
    chroma: (u32, u32),
    luma: Vec<BandShape>,
    chroma_bands: Vec<BandShape>,
}

impl Layout {
    fn plane_size(&self, p: usize) -> (u32, u32) {
        if p == 0 {
            (self.height, self.width)
        } else {
            (self.height.div_ceil(self.chroma.0), self.width.div_ceil(self.chroma.1))
        }
    }

    fn shapes(&self, p: usize) -> &[BandShape] {
        if p == 0 { &self.luma } else { &self.chroma_bands }
    }

    /// (height, width in samples, pitch in bytes) of a band.
    fn band_size(&self, p: usize, b: usize) -> (u32, u32, u32) {
        let (h, w) = self.plane_size(p);
        let s = self.shapes(p)[b];
        let (bh, bw) = (h >> s.height_shift, w >> s.width_shift);
        (bh, bw, (2 * bw + 31) & !31)
    }

    fn new_frame(&self) -> Vec<Vec<Band>> {
        (0..3)
            .map(|p| {
                (0..self.shapes(p).len())
                    .map(|b| {
                        let (bh, _, pitch) = self.band_size(p, b);
                        let rows = (bh + 15) & !15;
                        Band { pitch, samples: vec![0; (rows * pitch / 2 + 0x200) as usize] }
                    })
                    .collect()
            })
            .collect()
    }
}

pub struct Decoder {
    tables: Tables,
    dq: Dequant,
    /// [0], [1]: this picture's kind and the other context's; [0x11], [0x12]: their numbers.
    kind: [u32; 2],
    number: [u32; 2],
    /// [0x25]: the picture is held back.
    deferred: u32,
    /// [0x1E] receives, [0x1F] forward reference, [0x20] backward reference, [0x21] shown.
    bufs: [usize; 4],
    /// [0x1A] .. [0x1D]: whether each holds a picture.
    valid: [u32; 4],
    /// [0x22], [0x23]: a predicted, a bidirectional picture has been seen since the layout.
    seen_p: bool,
    seen_b: bool,
    /// [0x63D5]: the intra picture fills its chunk (or nothing that looks like a picture
    /// follows it).
    whole: bool,
    /// [0xE]: the picture's data size.
    data_size: u32,
    layout: Option<Layout>,
    frames: Vec<Vec<Vec<Band>>>,
    bands: Vec<Vec<BandState>>,
    shown: Option<Frame>,
}

/// What `FUN_1002c710` returns: 1 a picture to show, else the reason there is none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    Shown,
    NotShown,
}

impl Decoder {
    pub fn new(tables: Tables) -> Self {
        Self {
            tables,
            dq: Dequant::new(),
            kind: [0; 2],
            number: [0; 2],
            deferred: 0,
            bufs: [0, 1, 2, 1],
            valid: [0; 4],
            seen_p: false,
            seen_b: false,
            whole: true,
            data_size: 0,
            layout: None,
            frames: Vec::new(),
            bands: Vec::new(),
            shown: None,
        }
    }

    /// Decode one AVI video chunk. Returns the picture Intel's codec would hand out for it, or
    /// `None` when it hands out nothing new (the last picture stays).
    pub fn decode(&mut self, chunk: &[u8]) -> Result<Option<Frame>, Error> {
        let first = self.picture_call(chunk, false);
        let out = match first {
            Ok(Outcome::Shown) => self.shown.clone(),
            _ => None,
        };
        // FUN_10026690 at LAB_10026CDA: an intra picture with another picture after it.
        if self.kind[0] == 0 && !self.whole {
            let at = self.data_size as usize;
            if at < chunk.len() {
                let _ = self.picture_call(&chunk[at..], true);
            }
        }
        first.map(|_| out)
    }

    /// The last picture shown.
    pub fn last(&self) -> Option<&Frame> {
        self.shown.as_ref()
    }

    /// `FUN_1002c620`: swap the two contexts.
    fn swap_contexts(&mut self) {
        self.kind.swap(0, 1);
        self.number.swap(0, 1);
    }

    /// `FUN_1002c4c0`, before a picture.
    fn before(&mut self) {
        let [b1e, b1f, b20, _] = &mut self.bufs;
        let [v1a, v1b, v1c, _] = &mut self.valid;
        if self.deferred != 0 && *v1c != 0 {
            std::mem::swap(b1f, b20);
            *v1b = *v1c;
            *v1c = 0;
            return;
        }
        match self.kind[0] {
            0 => {
                std::mem::swap(b1f, b1e);
                *v1c = 0;
                *v1b = 0;
            }
            1 | 2 | 4 => *v1c = 0,
            6 => {
                std::mem::swap(b1e, b20);
                *v1a = *v1c;
                *v1c = 0;
            }
            _ => {}
        }
    }

    /// `FUN_1002c570`, after a picture.
    fn after(&mut self) {
        let [b1e, b1f, b20, b21] = &mut self.bufs;
        let [v1a, v1b, v1c, v1d] = &mut self.valid;
        if self.deferred != 0 {
            *b21 = *b1f;
            *v1d = *v1b;
            *v1b = *v1a & *v1b;
            *v1c = *v1a;
            std::mem::swap(b1e, b20);
            *v1a = 0;
            return;
        }
        match self.kind[0] {
            0..=2 => {
                std::mem::swap(b1f, b1e);
                *v1b = *v1a;
                *b21 = *b1f;
                *v1a = 0;
                *v1d = *v1b;
            }
            3 | 4 => {
                *b21 = *b1e;
                *v1d = *v1a;
            }
            _ => {}
        }
    }

    /// `FUN_1002c710`: the pictures from the start of `data`. `packed` for the second picture
    /// of a chunk (decoded in the other context, not shown).
    fn picture_call(&mut self, data: &[u8], packed: bool) -> Result<Outcome, Error> {
        if packed {
            self.swap_contexts();
        }
        let mut at = 0usize;
        let mut suppress = packed;
        loop {
            let hdr = PictureHeader::parse(data.get(at..).unwrap_or(&[]))?;
            self.header_state(&hdr, packed)?;
            self.before();
            let kind = self.kind[0];
            let [_, v1b, v1c, v1d] = self.valid;
            let decode = match kind {
                0 | 1 => true,
                2 | 4 => v1b != 0,
                3 => v1b != 0 && v1c != 0,
                _ => false,
            };
            match kind {
                5 => suppress = true,
                6 => self.swap_contexts(),
                _ if decode => {
                    let start = at + hdr.len;
                    let end = self.decode_bands(&hdr, data, start)?;
                    if self.data_size == 0 {
                        self.data_size = ((end - at + 8) & !3) as u32;
                    }
                    self.valid[0] = self.layout.as_ref().map_or(0, |l| l.luma.len() as u32);
                    if kind == 0 {
                        let next = at + self.data_size as usize;
                        self.whole = next == data.len() || data.get(next) != Some(&0xF8);
                    }
                }
                _ => {}
            }
            let _ = v1d;
            at += self.data_size as usize;
            if self.deferred != 0 {
                self.swap_contexts();
            }
            self.after();
            if !(self.deferred != 0 && self.whole) {
                break;
            }
        }
        if self.kind[1] == 6 {
            self.kind[1] = self.kind[0];
        }
        if !suppress && self.valid[3] != 0 {
            self.compose();
            Ok(Outcome::Shown)
        } else {
            Ok(Outcome::NotShown)
        }
    }

    /// What `FUN_100208d0` does to the instance besides reading the header.
    fn header_state(&mut self, h: &PictureHeader, packed: bool) -> Result<(), Error> {
        let mut kind = h.kind;
        self.data_size = h.data_size;
        if kind >= 5 {
            self.kind[0] = kind;
            self.deferred = 0;
            return Ok(());
        }
        let layout = Layout {
            height: h.height,
            width: h.width,
            tile_height: h.tile_height.unwrap_or(h.height),
            tile_width: h.tile_width.unwrap_or(h.width),
            chroma: h.chroma_div,
            luma: h.luma_bands.clone(),
            chroma_bands: h.chroma_bands.clone(),
        };
        let mut reset = (!self.seen_p && matches!(kind, 1 | 2 | 4)) || (!self.seen_b && kind == 3);
        if self.layout.as_ref() != Some(&layout) {
            if kind != 0 && self.layout.as_ref().is_some_and(|l| (l.height, l.width) != (layout.height, layout.width)) {
                return Err(Error::Picture("the picture size changes outside an intra picture"));
            }
            self.frames = (0..3).map(|_| layout.new_frame()).collect();
            self.bands = (0..3).map(|p| vec![BandState::default(); layout.shapes(p).len()]).collect();
            self.layout = Some(layout);
            self.seen_p = false;
            self.seen_b = false;
            reset = true;
        }
        // The picture number: a predicted picture with the same number as the last one is
        // null (kind 5).
        let n = h.number.unwrap_or(0);
        if h.number.is_some() && kind != 0 && !packed {
            let last = if self.deferred != 0 { self.number[1] } else { self.number[0] };
            if last == n {
                kind = 5;
            }
        }
        self.kind[0] = kind;
        self.number[0] = n;
        if kind == 0 {
            for b in self.bands.iter_mut().flatten() {
                b.flags = 0;
            }
        }
        self.deferred = h.deferred;
        if reset {
            let i = if kind == 0 { self.bufs[1] } else { self.bufs[0] };
            for b in self.frames[i].iter_mut().flatten() {
                b.samples.fill(0x4000);
            }
            match kind {
                1 | 2 | 4 => self.seen_p = true,
                3 => self.seen_b = true,
                _ => {}
            }
        }
        Ok(())
    }

    /// Decode the bands of the picture starting at byte `at`; returns where they end.
    fn decode_bands(&mut self, hdr: &PictureHeader, data: &[u8], mut at: usize) -> Result<usize, Error> {
        let layout = self.layout.clone().unwrap();
        let mb_book = match &hdr.mb_book {
            BookChoice::Table(i) => Codebook::new(&self.tables.mb_codebooks[*i]),
            BookChoice::Custom(d) => Codebook::new(d),
        }
        .ok_or(Error::Picture("bad macroblock codebook"))?;
        let kind = self.kind[0];
        let (cur, fwd, bwd) = (self.bufs[0], self.bufs[1], self.bufs[2]);
        // The luma band 0's lists, by tile, for the bands that inherit from it.
        let mut luma_lists: Vec<Vec<Entry>> = Vec::new();
        let mut luma_mb = 16;
        for p in 0..3 {
            for b in 0..layout.shapes(p).len() {
                let bh = BandHeader::parse(data.get(at..).unwrap_or(&[]))?;
                if (bh.plane, bh.band) != (p, b) {
                    return Err(Error::Picture("bands out of order"));
                }
                at += bh.len;
                if bh.empty {
                    self.bands[p][b].flags |= 1;
                    continue;
                }
                let state = &mut self.bands[p][b];
                if let Some(t) = bh.transform {
                    state.transform = t;
                }
                let valid = state.flags & 3 == 0;
                if !valid && kind != 3 && kind != 4 {
                    state.flags |= 2;
                }
                let (t, scan, matrix) = state.transform;
                let book = match bh.block_book.as_ref().unwrap_or(&hdr.block_book) {
                    BookChoice::Table(i) => Codebook::new(&self.tables.block_codebooks[*i]),
                    BookChoice::Custom(d) => Codebook::new(d),
                }
                .ok_or(Error::Picture("bad block codebook"))?;
                let rv = bh.rv_map.unwrap_or(hdr.rv_map);
                let rv = self.tables.rv_maps.get(rv).ok_or(Error::Picture("run/value map out of range"))?;
                let ctx = Context::new(
                    &self.tables,
                    Transform::from_number(t)?,
                    scan as usize,
                    matrix as usize,
                    book,
                    rv,
                    &bh.corrections,
                )?;
                let lists = self.decode_tiles(&layout, &bh, &ctx, &mb_book, hdr, data, &mut at, (p, b), (cur, fwd, bwd), valid, &luma_lists, luma_mb)?;
                if (p, b) == (0, 0) {
                    luma_lists = lists;
                    luma_mb = bh.mb_size;
                }
            }
        }
        Ok(at)
    }

    #[allow(clippy::too_many_arguments)]
    fn decode_tiles(
        &mut self,
        layout: &Layout,
        bh: &BandHeader,
        ctx: &Context,
        mb_book: &Codebook,
        hdr: &PictureHeader,
        data: &[u8],
        at: &mut usize,
        (p, b): (usize, usize),
        (cur, fwd, bwd): (usize, usize, usize),
        valid: bool,
        luma_lists: &[Vec<Entry>],
        luma_mb: u32,
    ) -> Result<Vec<Vec<Entry>>, Error> {
        let kind = self.kind[0];
        let (bh_h, bh_w, pitch) = layout.band_size(p, b);
        let (ph, pw) = (layout.height, layout.width);
        let tile_hb = layout.tile_height / (ph / bh_h.max(1)).max(1);
        let tile_wb = 2 * (layout.tile_width / (pw / bh_w.max(1)).max(1));
        let (trows, tcols) = (ph.div_ceil(layout.tile_height), pw.div_ceil(layout.tile_width));
        let mbs = bh.mb_size;
        if !valid {
            // FUN_10018810: a band coming back after being empty starts from a fill word, one
            // for each band (0x10077188 on the Pentium path), unless its first word is already
            // negative (filled). The word's halves are samples x and x + 8 (x + 4 for 4 × 4).
            const FILL: [u32; 4] = [0xC000_4000, 0xC004_4004, 0xC00C_400C, 0xC008_4008];
            let band = &mut self.frames[cur][p][b];
            let half = if mbs == 4 { 4 } else { 8 };
            let first = (band.samples.get(half).copied().unwrap_or(0) as u32) << 16 | band.samples[0] as u32;
            if (first as i32) >= 0 {
                let w = FILL.get(b).copied().unwrap_or(FILL[0]);
                let row = (pitch / 2) as usize;
                let n = (bh_h as usize * row).min(band.samples.len());
                for (i, s) in band.samples[..n].iter_mut().enumerate() {
                    *s = if (i % row) % (2 * half) < half { w as u16 } else { (w >> 16) as u16 };
                }
            }
        }
        let mut lists = Vec::new();
        let mut r = Bits::at(data, *at);
        let mut tile_start = *at;
        for t in 0..trows * tcols {
            let (tr, tc) = (t / tcols, t % tcols);
            let rows = if trows - tr == 1 { (bh_h - tile_hb * tr).div_ceil(mbs) } else { tile_hb.div_ceil(mbs) };
            let wbytes = if tcols - tc == 1 { 2 * bh_w - tile_wb * tc } else { tile_wb };
            let cols = (wbytes / 2).div_ceil(mbs);
            let offset = pitch * tile_hb * tr + tile_wb * tc;
            if r.flag() {
                // An empty tile: predicted pictures keep the forward reference's samples.
                if valid && kind > 1 {
                    let (src, dst) = if fwd < cur {
                        let (a, c) = self.frames.split_at_mut(cur);
                        (&a[fwd][p][b], &mut c[0][p][b])
                    } else {
                        let (a, c) = self.frames.split_at_mut(fwd);
                        (&c[0][p][b], &mut a[cur][p][b])
                    };
                    let n = if mbs == 16 { cols * 2 * 8 } else { cols.div_ceil(2) * 2 * mbs };
                    for row in 0..rows * mbs {
                        let s = ((offset + row * pitch) / 2) as usize;
                        let e = (s + n as usize).min(dst.samples.len());
                        dst.samples[s..e].copy_from_slice(&src.samples[s..e]);
                    }
                }
                lists.push(Vec::new());
                continue;
            }
            let size = if r.flag() {
                let s = r.get(8);
                if s == 255 { r.get(24) } else { s }
            } else {
                0
            };
            let mb_at = r.bytes_used();
            let reference = (bh.inherit_mv || bh.inherit_quant).then(|| {
                let l = luma_lists.get(t as usize).map(|v| &v[..]).unwrap_or(&[]);
                (l, luma_mb, false)
            });
            let mband = mb::MbBand {
                kind,
                mb_size: mbs,
                pitch,
                quant: bh.quant as i32,
                inherit_mv: bh.inherit_mv && kind >= 2,
                inherit_quant: bh.inherit_quant,
                quant_every_mb: hdr.quant_every_mb && (p, b) == (0, 0),
                book: mb_book,
                reference,
            };
            let mut mr = Bits::at(data, mb_at);
            let entries = mb::decode(&mut mr, &mband, rows, cols, offset);
            let blk_at = mr.bytes_used();
            // Motion first, then the residuals on top.
            let n = ctx.transform.size();
            let half = bh.mv_res != 0;
            {
                let mut tmp = std::mem::replace(&mut self.frames[cur][p][b], Band { pitch, samples: Vec::new() });
                for e in entries.iter().filter(|e| e.flags & 0xC != 0 && e.flags & INTRA == 0) {
                    mc::block(&mut tmp, &self.frames[fwd][p][b], &self.frames[bwd][p][b], e, n, half);
                }
                let used = blocks::decode_tile(ctx, &self.dq, &entries, data, blk_at, &mut tmp);
                self.frames[cur][p][b] = tmp;
                let end = blk_at + used;
                if size != 0 && (end - tile_start) as u32 != size {
                    return Err(Error::Picture("a tile's size does not match its data"));
                }
                let next = if size != 0 { tile_start + size as usize } else { end };
                tile_start = next;
                r = Bits::at(data, next);
            }
            lists.push(entries);
        }
        *at = r.bytes_used();
        Ok(lists)
    }

    /// `FUN_10025f10`: the shown frame store to 8-bit planes.
    fn compose(&mut self) {
        let Some(layout) = &self.layout else { return };
        let store = &self.frames[self.bufs[3]];
        let (w, h) = (layout.width as usize, layout.height as usize);
        let mut planes = Vec::new();
        for p in 0..3 {
            let (ph, pw) = layout.plane_size(p);
            let (ph, pw) = if p == 0 { (ph as usize, pw as usize) } else { (h.div_ceil(4), w.div_ceil(4)) };
            let mut out = vec![0u8; ph * pw];
            if store[p].len() == 4 {
                output::four(&store[p], &mut out, pw, ph);
            } else {
                output::single(&store[p][0], &mut out, pw, ph);
            }
            planes.push(out);
        }
        let u = planes.pop().unwrap();
        let v = planes.pop().unwrap();
        let y = planes.pop().unwrap();
        self.shown = Some(Frame { width: w, height: h, y, v, u });
    }
}
