//! Table fog (`GR_FOG_WITH_TABLE`): 64 entries, entry *i* for *w* = 2^(3 + i/4) / (8 − i mod
//! 4) (`guFogTableIndexToW`), and the Voodoo blends between neighbouring entries. The same
//! lookup as the fragment shader of GameFrame's `wrappers/glide`.

/// The *w* of entry `i`.
pub fn index_to_w(i: usize) -> f32 {
    2f32.powi(3 + (i >> 2) as i32) / (8 - (i & 3)) as f32
}

/// The fog amount (0–1) at `w`. Deviation: the chip's own interpolation weights (from the
/// bits of its floating *w*) are *unknown*; this is linear in *w* between the entries.
pub fn amount(table: &[u8; 64], w: f32) -> f32 {
    if !(w > 1.0) {
        return table[0] as f32 / 255.0;
    }
    let e = w.log2().floor();
    let fr = w / e.exp2();
    let (m, b0, b1) = if fr < 8.0 / 7.0 {
        (0, 1.0, 8.0 / 7.0)
    } else if fr < 4.0 / 3.0 {
        (1, 8.0 / 7.0, 4.0 / 3.0)
    } else if fr < 1.6 {
        (2, 4.0 / 3.0, 1.6)
    } else {
        (3, 1.6, 2.0)
    };
    let i = e as usize * 4 + m;
    if i >= 63 {
        return table[63] as f32 / 255.0;
    }
    let (a, b) = (table[i] as f32, table[i + 1] as f32);
    (a + (b - a) * (fr - b0) / (b1 - b0)) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_and_their_w() {
        assert_eq!(index_to_w(0), 1.0);
        assert_eq!(index_to_w(4), 2.0);
        let table: [u8; 64] = std::array::from_fn(|i| (i * 4) as u8);
        assert_eq!(amount(&table, 1.0), 0.0);
        assert_eq!(amount(&table, 2.0), 16.0 / 255.0);
        assert_eq!(amount(&table, 1e9), 252.0 / 255.0);
        // Halfway between entries 4 (w 2) and 5 (w 16/7).
        let mid = (2.0 + 16.0 / 7.0) / 2.0;
        assert!((amount(&table, mid) - 18.0 / 255.0).abs() < 1e-6);
    }
}
