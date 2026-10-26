//! 3dfx NCC ("narrow channel compression"): an 8-bit texel `YYYYIIQQ` read through a table of
//! 16 luminance values and 4 + 4 signed colour offsets, the `GuNccTable` of Glide 2.x.

/// One NCC table: `y[Y] + i[I] + q[Q]` per channel, clamped to 0..255.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NccTable {
    pub y: [u8; 16],
    /// Offsets for the 2-bit I value, R, G, B each; 9-bit signed (-256..255) on the chip.
    pub i: [[i16; 3]; 4],
    /// Offsets for the 2-bit Q value.
    pub q: [[i16; 3]; 4],
}

impl NccTable {
    /// The first 64 bytes of a `GuNccTable` in memory: `yRGB[16]` (u8), `iRGB[4][3]` and
    /// `qRGB[4][3]` (i16 little-endian). The `packed_data` words that follow are the same
    /// table in the chip's register layout and are not needed to decode.
    pub fn from_gu_bytes(b: &[u8]) -> Option<NccTable> {
        if b.len() < 64 {
            return None;
        }
        let mut t = NccTable { y: [0; 16], i: [[0; 3]; 4], q: [[0; 3]; 4] };
        t.y.copy_from_slice(&b[..16]);
        let w = |k: usize| i16::from_le_bytes([b[16 + 2 * k], b[17 + 2 * k]]);
        for n in 0..4 {
            for c in 0..3 {
                t.i[n][c] = w(n * 3 + c);
                t.q[n][c] = w(12 + n * 3 + c);
            }
        }
        Some(t)
    }

    /// One texel to RGB.
    pub fn decode(&self, texel: u8) -> [u8; 3] {
        gameframe_texel::Yiq::decode(self, texel)
    }
}

impl gameframe_texel::Yiq for NccTable {
    fn decode(&self, texel: u8) -> [u8; 3] {
        let (y, i, q) = ((texel >> 4) as usize, ((texel >> 2) & 3) as usize, (texel & 3) as usize);
        [0, 1, 2].map(|c| (self.y[y] as i32 + self.i[i][c] as i32 + self.q[q][c] as i32).clamp(0, 255) as u8)
    }
}
