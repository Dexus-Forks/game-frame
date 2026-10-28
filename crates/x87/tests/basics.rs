//! Small cases whose x87 result follows from the rules alone.

use gameframe_x87::{Control, Ext, Rounding};

fn x(v: f64) -> Ext {
    Ext::from_f64(v)
}

#[test]
fn exact_operations_equal_f64() {
    assert_eq!(x(1.5).fadd(x(2.25)).to_f64(), 3.75);
    assert_eq!(x(1.5).fsub(x(2.25)).to_f64(), -0.75);
    assert_eq!(x(-3.0).fmul(x(7.0)).to_f64(), -21.0);
    assert_eq!(Ext::from_i32(-100).fdiv(Ext::from_i32(8)).to_f64(), -12.5);
    assert_eq!(x(2.25).fsqrt().to_f64(), 1.5);
    assert_eq!(Ext::from_i64(i64::MIN).to_f64(), -(2f64.powi(63)));
}

#[test]
fn zeros_keep_their_signs() {
    let five = Ext::from_i32(5);
    assert_eq!(five.fsub(five), Ext::ZERO);
    let down = Control::DEFAULT.with_rounding(Rounding::Down);
    assert_eq!(five.fsub_with(five, down), Ext::NEG_ZERO);
    assert_eq!(Ext::NEG_ZERO.fadd(Ext::NEG_ZERO), Ext::NEG_ZERO);
    assert_eq!(x(-0.25).frndint(Control::DEFAULT), Ext::NEG_ZERO);
    assert!(x(-0.0).to_f64().is_sign_negative());
    assert_eq!(Ext::NEG_ZERO.fcom(Ext::ZERO), core::cmp::Ordering::Equal);
}

#[test]
fn a_quotient_rounds_twice() {
    // 1/3 to 64 bits, then to 53: equal to the f64 quotient here.
    assert_eq!(Ext::from_i32(1).fdiv(Ext::from_i32(3)).to_f64(), 1.0 / 3.0);
    // Here the two roundings differ from one.
    let twice = Ext::from_i32(17616).fdiv(Ext::from_i32(-23007)).to_f64();
    assert_ne!(twice, 17616.0 / -23007.0);
}

#[test]
fn a_small_addend_only_rounds() {
    let big = x(2f64.powi(64));
    assert_eq!(big.fadd(x(1.0)), big, "half a unit: to even");
    assert_ne!(big.fadd(x(3.0)), big, "past half a unit");
    assert_eq!(big.fsub(x(1.0)).to_f64(), 2f64.powi(64), "64 ones, then 53");
}

#[test]
fn precision_control_rounds_each_result() {
    let single = Control::from_word(0x007f);
    let third = Ext::from_i32(1)
        .checked_fdiv_with(Ext::from_i32(3), single)
        .unwrap();
    assert_eq!(third.to_f64(), f64::from(1.0f32 / 3.0));
}

#[test]
fn integers_round_by_the_mode() {
    assert_eq!(x(2.5).round_i32(), 2);
    assert_eq!(x(3.5).round_i32(), 4);
    assert_eq!(x(-2.5).round_i32(), -2);
    assert_eq!(x(-2.6).round_i32(), -3);
    assert_eq!(x(-2.6).trunc_i32(), -2);
    assert_eq!(x(2.1).fist_i32(Control::from_word(0x0b7f)), 3);
    assert_eq!(x(-2.1).fist_i32(Control::from_word(0x077f)), -3);
    assert_eq!(x(1e300).round_i32(), i32::MIN, "the integer indefinite");
    assert_eq!(x(32767.4).fist_i16(Control::DEFAULT), 32767);
    assert_eq!(x(32767.6).fist_i16(Control::DEFAULT), i16::MIN);
    assert_eq!(x(-32768.4).fist_i16(Control::DEFAULT), -32768);
}

#[test]
fn stores_make_denormals_and_infinities() {
    let tiny = x(f64::MIN_POSITIVE).fmul(x(0.25));
    assert_eq!(tiny.to_f64(), f64::MIN_POSITIVE / 4.0);
    let huge = x(f64::MAX).fmul(x(2.0));
    assert_eq!(huge.to_f64(), f64::INFINITY);
    let chop = Control::DEFAULT.with_rounding(Rounding::Zero);
    assert_eq!(huge.to_f64_with(chop), f64::MAX);
    assert_eq!(huge.to_f32(), f32::INFINITY);
    assert!(huge.to_bytes().is_some(), "still inside the extended range");
}

#[test]
fn ten_bytes_round_trip() {
    for v in [1.0, -0.1, 3.5e-300, 1e300, 0.0] {
        let e = x(v);
        assert_eq!(Ext::from_bytes(e.to_bytes().unwrap()), Some(e));
    }
    assert_eq!(
        Ext::ONE.to_bytes().unwrap(),
        [0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0x3f]
    );
    assert_eq!(
        Ext::from_bytes([0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0x7f]),
        None,
        "infinity"
    );
}
