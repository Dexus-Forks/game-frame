//! Properties of the runtime that hold without the original program at hand. The bit-for-bit
//! checks against the original runtime live in the Tyrian 2026 project, which has the program.

use gameframe_turbo_pascal::{PStr, PascalReader, PascalWriter, Real48, TurboRandom};

#[test]
fn encodings() {
    assert_eq!(Real48::from_f64(1.0).0, [0x81, 0, 0, 0, 0, 0]);
    assert_eq!(Real48::from_f64(0.75).0, [0x80, 0, 0, 0, 0, 0x40]);
    assert_eq!(Real48::from_f64(-2.0).0, [0x82, 0, 0, 0, 0, 0x80]);
    assert_eq!(Real48::from_f64(0.0).0, [0; 6]);
    assert_eq!(Real48::from_i32(-2), Real48::from_f64(-2.0));
}

#[test]
fn integers_are_exact() {
    for x in (-70_000..70_000).step_by(7) {
        assert_eq!(Real48::from_i32(x).to_f64(), x as f64, "{x}");
        assert_eq!(Real48::from_i32(x).trunc(), x);
        assert_eq!(Real48::from_i32(x).round(), x);
    }
}

#[test]
fn small_integer_arithmetic_is_exact() {
    for a in -50..50 {
        for b in -50..50 {
            let (ra, rb) = (Real48::from_i32(a), Real48::from_i32(b));
            assert_eq!(ra.mul(rb).to_f64(), (a * b) as f64, "{a} * {b}");
            assert_eq!(ra.add(rb).to_f64(), (a + b) as f64, "{a} + {b}");
            assert_eq!(ra.sub(rb).to_f64(), (a - b) as f64, "{a} - {b}");
        }
    }
}

#[test]
fn round_takes_halves_away_from_zero() {
    let half = |x: f64| Real48::from_f64(x).round();
    assert_eq!([half(0.5), half(1.5), half(2.5), half(-0.5), half(-2.5)], [1, 2, 3, -1, -3]);
    assert_eq!([Real48::from_f64(2.7).trunc(), Real48::from_f64(-2.7).trunc()], [2, -2]);
}

#[test]
fn int_and_frac() {
    let x = Real48::from_f64(-3.25);
    assert_eq!(x.int_part().to_f64(), -3.0);
    assert_eq!(x.frac().to_f64(), -0.25);
}

#[test]
fn sin_and_cos_are_close() {
    for i in -400..400 {
        let x = i as f64 * 0.0123;
        let r = Real48::from_f64(x);
        assert!((r.sin().to_f64() - x.sin()).abs() < 1e-9, "sin {x}");
        assert!((r.cos().to_f64() - x.cos()).abs() < 1e-9, "cos {x}");
    }
}

#[test]
fn random_follows_the_lcg() {
    let mut r = TurboRandom::new(0);
    assert_eq!(r.next_rand(), 1);
    assert_eq!(r.next_rand(), 134_775_814);
    let mut r = TurboRandom::new(42);
    let mut s = 42u32;
    for range in [1u16, 2, 10, 100, 1000, 65535] {
        s = s.wrapping_mul(134_775_813).wrapping_add(1);
        assert_eq!(r.random(range), ((s as u64 * range as u64) >> 32) as u16);
    }
    assert_eq!(TurboRandom::new(5).random(0), 0);
    let f = Real48::from_seed(0x8000_0000).to_f64();
    assert_eq!(f, 0.5);
}

#[test]
fn data_round_trip() {
    let mut buf = [0u8; 32];
    let mut w = PascalWriter::new(&mut buf);
    w.u16(0x1234);
    w.boolean(true);
    w.real48(Real48::from_i32(7));
    let mut name = PStr::<10>::default();
    name.len = 3;
    name.bytes[..3].copy_from_slice(b"Ace");
    w.pstr(&name);
    let mut r = PascalReader::new(&buf);
    assert_eq!(r.u16().unwrap(), 0x1234);
    assert!(r.boolean().unwrap());
    assert_eq!(r.real48().unwrap().to_f64(), 7.0);
    assert_eq!(r.pstr::<10>().unwrap().to_string_lossy(), "Ace");
    assert_eq!(r.pos(), 2 + 1 + 6 + 11);
}
