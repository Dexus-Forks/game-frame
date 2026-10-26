//! Glide's texture formats, sizes and NCC tables against the SDK's values.

use gameframe_texel::{decode, Tables, TexelFormat};
use gameframe_voodoo::{formats, ncc::NccTable};

fn one(f: TexelFormat, bytes: &[u8], t: Tables) -> [u8; 4] {
    let p = decode(f, 1, 1, bytes, t).unwrap().pixels;
    [p[0], p[1], p[2], p[3]]
}

#[test]
fn ncc_table() {
    let mut raw = vec![0u8; 64];
    for (k, v) in raw[..16].iter_mut().enumerate() {
        *v = (k * 16) as u8;
    }
    let put = |raw: &mut Vec<u8>, k: usize, v: i16| raw[16 + 2 * k..18 + 2 * k].copy_from_slice(&v.to_le_bytes());
    put(&mut raw, 3, 100); // i[1][0] = +100 red
    put(&mut raw, 12 + 2, -50); // q[0][2] = -50 blue
    let t = NccTable::from_gu_bytes(&raw).unwrap();
    // Y = 15 (240), I = 1, Q = 0: red 240 + 100 clamps to 255, blue 240 - 50.
    assert_eq!(t.decode(0xf4), [255, 240, 190]);
    let tables = Tables { palette: None, ncc: Some(&t) };
    assert_eq!(one(TexelFormat::Ayiq8422, &[0x00, 0x80], tables), [0, 0, 0, 0x80]);
}

#[test]
fn glide_constants_and_sizes() {
    assert_eq!(formats::Format::from_u32(10), Some(formats::Format::Rgb565));
    assert_eq!(formats::Format::from_u32(5).unwrap().texel(), TexelFormat::Palette8);
    assert_eq!(formats::Format::from_u32(6), None);
    assert_eq!(formats::size(0, 3), Some((256, 256)));
    assert_eq!(formats::size(1, 3), Some((128, 128)));
    assert_eq!(formats::size(0, 0), Some((256, 32)));
    assert_eq!(formats::size(2, 6), Some((8, 64)));
    assert_eq!(formats::size(8, 0), Some((1, 1)));
    assert_eq!(formats::size(9, 3), None);
    assert_eq!(formats::chain_bytes(0, 0, 3, formats::Format::Rgb565), Some(0x20000));
    assert_eq!(formats::chain_bytes(1, 1, 3, formats::Format::Rgb565), Some(0x8000));
    assert_eq!(formats::chain_bytes(1, 0, 3, formats::Format::P8), Some(0x10000 + 0x4000));
    assert_eq!(formats::palette_entry([0x12, 0x34, 0x56]), 0x123456);
}

#[test]
fn what_glide_accepts() {
    assert!(formats::TEXTURE_FORMATS.contains(&TexelFormat::Rgb565));
    assert!(formats::TEXTURE_FORMATS.contains(&TexelFormat::Yiq422));
    assert_eq!(formats::MAX_TEXTURE_SIDE, 256);
}
