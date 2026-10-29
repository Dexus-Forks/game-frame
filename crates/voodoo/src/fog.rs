//! Fog. *Observed* in the SST-1 specification, section 5.17 (`fogMode` and the fog unit's
//! diagram): the table is indexed by the top 6 bits of the W buffer's floating value (4 bits
//! of exponent, 2 of mantissa, [`crate::frame::wfloat`]), and the next 8 bits blend to the
//! next entry; the fog alpha, plus one, scales the fog colour minus the pixel's colour, and
//! the product (its 8 low bits dropped) is added to the pixel's colour and clamped. With
//! `FOG_WITH_ITERATED_ALPHA` the iterated alpha replaces the table's value.
//!
//! The entry *i* stands for *w* = 2^(3 + i/4) / (8 − i mod 4) (`guFogTableIndexToW`): those are
//! the *w* where [`crate::frame::wfloat`]'s top 6 bits step.
//!
//! *Inferred*: the blend between entries is `table[i] + ((table[i+1] − table[i]) × frac) >> 8`
//! (Glide downloads each entry with its difference to the next).

/// The *w* of entry `i`.
pub fn index_to_w(i: usize) -> f32 {
    2f32.powi(3 + (i >> 2) as i32) / (8 - (i & 3)) as f32
}

/// `guFogGenerateLinear(table, nearW, farW)`: the Glide 2.x utility that fills the table with
/// a fog that rises linearly from 0 at `near` to 255 at `far` (in w), each entry the value at
/// its own w ([`index_to_w`]). *Inferred* from the Glide 2.x reference manual, which describes
/// it as linear fog between the two w. Deviation: the clamp to 0–1 and the cut to an integer
/// are this model's; whether Glide rounds instead is *unknown*.
#[allow(non_snake_case)]
pub fn guFogGenerateLinear(table: &mut [u8; 64], near: f32, far: f32) {
    for (i, e) in table.iter_mut().enumerate() {
        let w = index_to_w(i).min(65535.0);
        let f = ((w - near) / (far - near)).clamp(0.0, 1.0);
        *e = (f * 255.0) as u8;
    }
}

/// The fog alpha (0–255) at W buffer value `w`.
pub fn table_alpha(table: &[u8; 64], w: u16) -> u8 {
    let i = (w >> 10) as usize;
    let frac = ((w >> 2) & 0xff) as i32;
    let a = table[i] as i32;
    let next = table.get(i + 1).map_or(a, |&b| b as i32);
    (a + (((next - a) * frac) >> 8)).clamp(0, 255) as u8
}

/// One channel fogged: `c + ((fog − c) × (alpha + 1)) >> 8`, clamped.
pub fn apply(c: u8, fog: u8, alpha: u8) -> u8 {
    let d = fog as i32 - c as i32;
    (c as i32 + ((d * (alpha as i32 + 1)) >> 8)).clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::wfloat;

    #[test]
    fn entries_meet_the_w_buffer_steps() {
        // At each entry's w the W buffer's top 6 bits are the entry's index.
        for i in 0..60 {
            let w = index_to_w(i) * 1.0001;
            assert_eq!(wfloat(1.0 / w) >> 10, i as u16, "entry {i}");
        }
    }

    #[test]
    fn linear_fog_table() {
        let mut t = [0u8; 64];
        guFogGenerateLinear(&mut t, 100.0, 1000.0);
        assert_eq!(t[0], 0);
        assert_eq!(t[63], 255);
        assert!(t.windows(2).all(|p| p[0] <= p[1]));
        // Entry 28 is w = 2^10 / 8 = 128: (128 - 100) / 900 of the way.
        assert_eq!(index_to_w(28), 128.0);
        assert_eq!(t[28], (28.0f32 / 900.0 * 255.0) as u8);
    }

    #[test]
    fn lookup_and_blend() {
        let table: [u8; 64] = std::array::from_fn(|i| (i * 4) as u8);
        assert_eq!(table_alpha(&table, 0), 0);
        assert_eq!(table_alpha(&table, 5 << 10), 20);
        assert_eq!(table_alpha(&table, 5 << 10 | 0x80 << 2), 22);
        assert_eq!(table_alpha(&table, 0xffff), 252);
        assert_eq!(apply(0, 255, 255), 255);
        assert_eq!(apply(200, 0, 127), 100);
        assert_eq!(apply(100, 200, 0), 100);
    }
}
