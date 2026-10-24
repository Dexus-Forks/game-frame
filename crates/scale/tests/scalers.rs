//! The scalers' rules on small frames whose results are known by hand.
use gameframe_scale::{crt, nearest, scale2x, scale4x};

#[test]
fn nearest_makes_blocks() {
    let out = nearest(&[1u16, 2, 3, 4, 5, 6], 3, 2, 3);
    assert_eq!(out.len(), 9 * 6);
    for y in 0..6 {
        for x in 0..9 {
            assert_eq!(out[y * 9 + x], [1, 2, 3, 4, 5, 6][(y / 3) * 3 + x / 3]);
        }
    }
}

#[test]
fn scale2x_keeps_flat_areas_and_straight_edges() {
    // A vertical edge: the left half 0, the right half 1. Straight edges stay straight.
    let frame: Vec<u8> = (0..16).map(|i| (i % 4 >= 2) as u8).collect();
    let out = scale2x(&frame, 4, 4);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(out[y * 8 + x], (x >= 4) as u8, "at {x},{y}");
        }
    }
}

#[test]
fn scale2x_rounds_a_diagonal() {
    // A 1-pixel staircase below the diagonal of a 3 x 3 frame:
    //   1 0 0
    //   1 1 0
    //   1 1 1
    let frame = [1u8, 0, 0, 1, 1, 0, 1, 1, 1];
    let out = scale2x(&frame, 3, 3);
    // The centre pixel's top-right quarter takes the 0 above and to its right: the step is cut.
    let quarter = |x: usize, y: usize| out[y * 6 + x];
    assert_eq!([quarter(2, 2), quarter(3, 2), quarter(2, 3), quarter(3, 3)], [1, 0, 1, 1]);
    // Nearest would have kept all four 1.
    assert_eq!(nearest(&frame, 3, 3, 2)[2 * 6 + 3], 1);
}

#[test]
fn scale4x_is_two_passes() {
    let frame = [1u8, 0, 0, 1, 1, 0, 1, 1, 1];
    assert_eq!(scale4x(&frame, 3, 3), scale2x(&scale2x(&frame, 3, 3), 6, 6));
    assert_eq!(scale4x(&frame, 3, 3).len(), 12 * 12);
}

#[test]
fn crt_darkens_the_last_row_of_each_block() {
    let out = crt(&[0x80_c8_64_32u32, 0xff_ff_ff_ff], 2, 1, 3, 50);
    assert_eq!(out.len(), 6 * 3);
    assert!(out[..12].iter().zip([0x80_c8_64_32u32, 0xff_ff_ff_ff].iter().flat_map(|&p| [p; 3]).cycle()).all(|(a, b)| *a == b));
    assert_eq!(&out[12..15], &[0x80_64_32_19; 3]);
    assert_eq!(&out[15..18], &[0xff_7f_7f_7f; 3]);
    assert_eq!(crt(&[5u32, 6], 2, 1, 1, 50), [5, 6]);
}
