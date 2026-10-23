//! Known answers for each instruction family, edge cases, and two kernels written the way
//! 1997 hand-written MMX code wrote them (a stereo mixer step and a span bit mask).

use gameframe_mmx::*;

/// A small deterministic generator, so the tests need no dependencies.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn mm(&mut self) -> Mm {
        Mm(self.next() ^ (self.next() << 17))
    }
}

#[test]
fn saturating_words() {
    let a = Mm::from_swords([32000, -32000, 100, -1]);
    let b = Mm::from_swords([1000, -1000, -200, 1]);
    assert_eq!(paddsw(a, b).swords(), [32767, -32768, -100, 0]);
    assert_eq!(paddw(a, b).swords(), [-32536, 32536, -100, 0]);
    assert_eq!(psubsw(a, b).swords(), [31000, -31000, 300, -2]);
    assert_eq!(paddusw(Mm::splat_w(0xfff0), Mm::splat_w(0x20)).words(), [0xffff; 4]);
    assert_eq!(psubusw(Mm::splat_w(5), Mm::splat_w(9)).words(), [0; 4]);
}

#[test]
fn saturating_bytes() {
    let a = Mm::from_bytes([120, 0x88, 250, 3, 0, 0, 0, 0]);
    let b = Mm::from_bytes([10, 0xf0, 10, 4, 0, 0, 0, 0]);
    assert_eq!(paddsb(a, b).bytes()[..4], [127, 0x80, 4, 7]);
    assert_eq!(paddusb(a, b).bytes()[..4], [130, 255, 255, 7]);
    assert_eq!(psubusb(b, a).bytes()[..4], [0, 0x68, 0, 1]);
}

#[test]
fn multiplies() {
    let a = Mm::from_swords([-32768, 32767, -3, 0x4000]);
    let b = Mm::from_swords([-32768, 32767, 5, 0x4000]);
    assert_eq!(pmulhw(a, b).swords(), [0x4000, 0x3fff, -1, 0x1000]);
    assert_eq!(pmullw(a, b).swords(), [0, 1, -15, 0]);
    // Every word 0x8000: the one overflow of pmaddwd.
    let m = Mm::splat_w(0x8000);
    assert_eq!(pmaddwd(m, m).dwords(), [0x8000_0000; 2]);
    let c = Mm::from_swords([1, 2, 3, 4]);
    let d = Mm::from_swords([10, -20, 30, 40]);
    assert_eq!(pmaddwd(c, d).sdwords(), [10 - 40, 90 + 160]);
}

#[test]
fn multiplies_match_wide_arithmetic() {
    let mut r = Lcg(1997);
    for _ in 0..10_000 {
        let (a, b) = (r.mm(), r.mm());
        let (x, y) = (a.swords(), b.swords());
        let hi: Vec<i16> = (0..4).map(|i| ((x[i] as i64 * y[i] as i64) >> 16) as i16).collect();
        assert_eq!(pmulhw(a, b).swords().to_vec(), hi);
        let sat: Vec<i16> = (0..4).map(|i| (x[i] as i64 + y[i] as i64).clamp(-32768, 32767) as i16).collect();
        assert_eq!(paddsw(a, b).swords().to_vec(), sat);
    }
}

#[test]
fn compares() {
    let a = Mm::from_swords([1, -1, 5, 7]);
    let b = Mm::from_swords([1, 1, 4, 9]);
    assert_eq!(pcmpeqw(a, b).words(), [0xffff, 0, 0, 0]);
    assert_eq!(pcmpgtw(a, b).words(), [0, 0, 0xffff, 0]);
    assert_eq!(pcmpgtd(Mm::from_sdwords([-1, 2]), Mm::from_sdwords([-2, 3])).dwords(), [u32::MAX, 0]);
    assert_eq!(pcmpeqb(Mm(0x00ff), Mm(0x01ff)).bytes(), [0xff, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn packs_and_unpacks() {
    let a = Mm::from_swords([300, -300, 5, -5]);
    let b = Mm::from_swords([127, 128, -128, -129]);
    assert_eq!(packsswb(a, b).bytes(), [127, 0x80, 5, 0xfb, 127, 127, 0x80, 0x80]);
    assert_eq!(packuswb(a, b).bytes(), [255, 0, 5, 0, 127, 128, 0, 0]);
    assert_eq!(packssdw(Mm::from_sdwords([70000, -70000]), Mm::from_sdwords([1, -1])).swords(), [32767, -32768, 1, -1]);
    let l = Mm::from_words([1, 2, 3, 4]);
    let r = Mm::from_words([5, 6, 7, 8]);
    assert_eq!(punpcklwd(l, r).words(), [1, 5, 2, 6]);
    assert_eq!(punpckhwd(l, r).words(), [3, 7, 4, 8]);
    assert_eq!(punpckldq(l, r).words(), [1, 2, 5, 6]);
    assert_eq!(punpckhdq(l, r).words(), [3, 4, 7, 8]);
    assert_eq!(punpcklbw(Mm(0x0403_0201), Mm(0x0807_0605)).bytes(), [1, 5, 2, 6, 3, 7, 4, 8]);
    // movd then punpcklwd twice: one word in every lane.
    let v = Mm::from_d(0x1234);
    assert_eq!(punpcklwd(punpcklwd(v, v), punpcklwd(v, v)), Mm::splat_w(0x1234));
}

#[test]
fn shifts_past_the_lane_width() {
    let a = Mm::from_swords([-4, 4, -32768, 1]);
    assert_eq!(psraw(a, 1).swords(), [-2, 2, -16384, 0]);
    assert_eq!(psraw(a, 40).swords(), [-1, 0, -1, 0]);
    assert_eq!(psrlw(a, 16).words(), [0; 4]);
    assert_eq!(psllq(Mm(1), 64), Mm(0));
    assert_eq!(psrlq(Mm(u64::MAX), 64), Mm(0));
    assert_eq!(psrad(Mm::from_sdwords([-8, 8]), 2).sdwords(), [-2, 2]);
    assert_eq!(pandn(Mm(0xf0), Mm(0xff)), Mm(0x0f));
}

/// The stereo mixer step of 1997 sound code: four 16-bit mono samples scaled by a left and a
/// right volume with `pmulhw`, interleaved with `punpck[lh]wd` and added with `paddsw`.
fn mix4(samples: Mm, left: u16, right: u16, out: [Mm; 2]) -> [Mm; 2] {
    let l = pmulhw(samples, Mm::splat_w(left));
    let r = pmulhw(samples, Mm::splat_w(right));
    [paddsw(out[0], punpcklwd(l, r)), paddsw(out[1], punpckhwd(l, r))]
}

#[test]
fn stereo_mix_kernel() {
    let mut rng = Lcg(42);
    for _ in 0..2_000 {
        let s = rng.mm();
        let (left, right) = (rng.next() as u16 & 0x7fff, rng.next() as u16 & 0x7fff);
        let out = [rng.mm(), rng.mm()];
        let got = mix4(s, left, right, out);
        let o: Vec<i16> = out.iter().flat_map(|m| m.swords()).collect();
        let g: Vec<i16> = got.iter().flat_map(|m| m.swords()).collect();
        for (i, &x) in s.swords().iter().enumerate() {
            let want_l = (o[2 * i] as i32 + ((x as i32 * left as i32) >> 16)).clamp(-32768, 32767);
            let want_r = (o[2 * i + 1] as i32 + ((x as i32 * right as i32) >> 16)).clamp(-32768, 32767);
            assert_eq!((g[2 * i] as i32, g[2 * i + 1] as i32), (want_l, want_r));
        }
    }
}

/// A span of `len` pixels from bit position `start` (counted from the top bit) of a 64-bit
/// coverage word, built as hand-written code built it: shift all ones right by the length,
/// invert, then shift right by the start.
fn span_mask(start: u64, len: u64) -> Mm {
    let ones = Mm(u64::MAX);
    psrlq(pxor(psrlq(ones, len), ones), start)
}

#[test]
fn span_mask_kernel() {
    for start in 0..64 {
        for len in 0..=80 {
            let want = (start..(start + len).min(64)).fold(0u64, |m, x| m | 1 << (63 - x));
            assert_eq!(span_mask(start, len), Mm(want), "start {start} len {len}");
        }
    }
}
