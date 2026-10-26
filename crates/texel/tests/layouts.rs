//! Known texels in every layout, and the 2:1 reduction.

use gameframe_texel::{decode, resample, Tables, TexelFormat};

fn one(f: TexelFormat, bytes: &[u8], t: Tables) -> [u8; 4] {
    let p = decode(f, 1, 1, bytes, t).unwrap().pixels;
    [p[0], p[1], p[2], p[3]]
}

#[test]
fn sixteen_bit_layouts() {
    let none = Tables::default();
    assert_eq!(one(TexelFormat::Rgb565, &0xf800u16.to_le_bytes(), none), [255, 0, 0, 255]);
    assert_eq!(one(TexelFormat::Rgb565, &0x07e0u16.to_le_bytes(), none), [0, 255, 0, 255]);
    assert_eq!(one(TexelFormat::Argb1555, &0x801fu16.to_le_bytes(), none), [0, 0, 255, 255]);
    assert_eq!(one(TexelFormat::Argb1555, &0x7c00u16.to_le_bytes(), none), [255, 0, 0, 0]);
    assert_eq!(one(TexelFormat::Argb4444, &0x8f40u16.to_le_bytes(), none), [255, 0x44, 0, 0x88]);
    assert_eq!(one(TexelFormat::Argb8332, &0x80e0u16.to_le_bytes(), none), [255, 0, 0, 0x80]);
    assert_eq!(one(TexelFormat::AlphaIntensity88, &0x40c0u16.to_le_bytes(), none), [0xc0, 0xc0, 0xc0, 0x40]);
}

#[test]
fn eight_bit_layouts() {
    let none = Tables::default();
    assert_eq!(one(TexelFormat::Rgb332, &[0xff], none), [255, 255, 255, 255]);
    assert_eq!(one(TexelFormat::Rgb332, &[0x03], none), [0, 0, 255, 255]);
    assert_eq!(one(TexelFormat::Rgb332, &[0x20], none), [0x24, 0, 0, 255]);
    assert_eq!(one(TexelFormat::Alpha8, &[0x7f], none), [255, 255, 255, 0x7f]);
    assert_eq!(one(TexelFormat::Intensity8, &[0x31], none), [0x31, 0x31, 0x31, 255]);
    assert_eq!(one(TexelFormat::AlphaIntensity44, &[0xf1], none), [0x11, 0x11, 0x11, 255]);
    assert_eq!(one(TexelFormat::Argb8888, &[1, 2, 3, 4], none), [3, 2, 1, 4]);
}

#[test]
fn palettes() {
    let pal: Vec<[u8; 3]> = (0..=255u8).map(|i| [i, 255 - i, 7]).collect();
    let t = Tables { palette: Some(&pal), ncc: None };
    assert_eq!(one(TexelFormat::Palette8, &[10], t), [10, 245, 7, 255]);
    assert_eq!(one(TexelFormat::AlphaPalette88, &[10, 0x55], t), [10, 245, 7, 0x55]);
    let two = decode(TexelFormat::Palette4, 2, 1, &[0x21], t).unwrap().pixels;
    assert_eq!((two[0], two[4]), (1, 2));
    assert!(decode(TexelFormat::Palette8, 1, 1, &[0], Tables::default()).is_err());
    assert!(decode(TexelFormat::Rgb565, 2, 2, &[0; 7], Tables::default()).is_err());
}

#[test]
fn halving() {
    let w: Vec<u8> = (0..16u8).flat_map(|v| [v, 0x80 | v]).collect(); // 4 x 4 of 16-bit texels
    let h = resample::halve_point(&w, 4, 4, 2);
    assert_eq!(h, vec![0, 0x80, 2, 0x82, 8, 0x88, 10, 0x8a]);
}
