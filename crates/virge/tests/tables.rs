//! The S3D Toolkit's tables against `S3DTK.H`.

use gameframe_texel::TexelFormat;
use gameframe_virge as s3;

#[test]
fn state_keys_and_formats() {
    assert_eq!(s3::state_name(10), Some("RENDERINGTYPE"));
    assert_eq!(s3::state_name(19), Some("ALPHABLENDING"));
    assert_eq!(s3::texel(s3::TEXTURE | s3::TEXARGB1555), Some(TexelFormat::Argb1555));
    assert_eq!(s3::texel(s3::TEXPALETTIZED8), Some(TexelFormat::Palette8));
    assert_eq!(s3::FUNCTIONS[2], "S3DTK_TriangleSet");
    assert!(!s3::TEXTURE_FORMATS.contains(&TexelFormat::Rgb565));
}
