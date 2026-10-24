#![doc = include_str!("../README.md")]
#![doc = ""]
//! Whole-multiple scalers for a finished frame. Every function takes the frame row after row,
//! `w` pixels wide and `h` rows high, and returns the scaled frame the same way. They panic
//! when the slice is shorter than `w * h`, or when the factor is 0.
//!
//! Reference for Scale2x: Andrea Mazzoleni, *Scale2x* (scale2x.it), the same rules as Eric
//! Johnston's EPX (1992).

mod crt;
mod epx;

pub use crt::crt;
pub use epx::{scale2x, scale4x};

/// Each pixel becomes a `k` × `k` block.
pub fn nearest<P: Copy>(src: &[P], w: usize, h: usize, k: usize) -> Vec<P> {
    assert!(k > 0 && src.len() >= w * h, "nearest: factor 0, or a frame shorter than w * h");
    let mut out = Vec::with_capacity(w * h * k * k);
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let start = out.len();
        for &p in row {
            out.extend(std::iter::repeat(p).take(k));
        }
        for _ in 1..k {
            out.extend_from_within(start..start + w * k);
        }
    }
    out
}
