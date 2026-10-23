//! Known texels in every layout, the API tables against their SDKs' values, sizes and the
//! 2:1 reduction.

use gameframe_gpu_1997::texel::{decode, Tables};
use gameframe_gpu_1997::{ati, card, glide, ncc::NccTable, resample, s3, TexelFormat};

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
    assert_eq!(glide::Format::from_u32(10), Some(glide::Format::Rgb565));
    assert_eq!(glide::Format::from_u32(5).unwrap().texel(), TexelFormat::Palette8);
    assert_eq!(glide::Format::from_u32(6), None);
    assert_eq!(glide::size(0, 3), Some((256, 256)));
    assert_eq!(glide::size(1, 3), Some((128, 128)));
    assert_eq!(glide::size(0, 0), Some((256, 32)));
    assert_eq!(glide::size(2, 6), Some((8, 64)));
    assert_eq!(glide::size(8, 0), Some((1, 1)));
    assert_eq!(glide::size(9, 3), None);
    assert_eq!(glide::chain_bytes(0, 0, 3, glide::Format::Rgb565), Some(0x20000));
    assert_eq!(glide::chain_bytes(1, 1, 3, glide::Format::Rgb565), Some(0x8000));
    assert_eq!(glide::chain_bytes(1, 0, 3, glide::Format::P8), Some(0x10000 + 0x4000));
    assert_eq!(glide::palette_entry([0x12, 0x34, 0x56]), 0x123456);
}

#[test]
fn ati_and_s3_tables() {
    assert_eq!(ati::render_state_name(2), Some("PRIM_TYPE"));
    assert_eq!(ati::render_state_name(14), Some("SURF_DRAW_PITCH"));
    assert_eq!(ati::render_state_name(35), Some("SPECULAR_EN"));
    assert_eq!(ati::TexFormat::from_u32(4).and_then(|f| f.texel()), Some(TexelFormat::Rgb565));
    assert_eq!(ati::TexFormat::from_u32(11).and_then(|f| f.texel()), None);
    assert_eq!(s3::state_name(10), Some("RENDERINGTYPE"));
    assert_eq!(s3::state_name(19), Some("ALPHABLENDING"));
    assert_eq!(s3::texel(s3::TEXTURE | s3::TEXARGB1555), Some(TexelFormat::Argb1555));
    assert_eq!(s3::texel(s3::TEXPALETTIZED8), Some(TexelFormat::Palette8));
    assert_eq!(s3::FUNCTIONS[2], "S3DTK_TriangleSet");
}

#[test]
fn cards() {
    assert!(card::Card::Voodoo.accepts(TexelFormat::Rgb565));
    assert!(!card::Card::Virge.accepts(TexelFormat::Rgb565));
    assert!(card::Card::Rage.accepts(TexelFormat::Rgb565));
    assert!(card::Card::Software.accepts(TexelFormat::Yiq422));
    assert_eq!(card::Card::Voodoo.max_texture_side(), Some(256));
    assert_eq!(card::ALL.len(), 4);
}

#[test]
fn halving() {
    let w: Vec<u8> = (0..16u8).flat_map(|v| [v, 0x80 | v]).collect(); // 4 x 4 of 16-bit texels
    let h = resample::halve_point(&w, 4, 4, 2);
    assert_eq!(h, vec![0, 0x80, 2, 0x82, 8, 0x88, 10, 0x8a]);
}
