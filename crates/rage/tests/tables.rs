//! ATI3DCIF's tables against the programmer's guide.

use gameframe_rage as ati;
use gameframe_texel::TexelFormat;

#[test]
fn render_states_and_formats() {
    assert_eq!(ati::render_state_name(2), Some("PRIM_TYPE"));
    assert_eq!(ati::render_state_name(14), Some("SURF_DRAW_PITCH"));
    assert_eq!(ati::render_state_name(35), Some("SPECULAR_EN"));
    assert_eq!(ati::TexFormat::from_u32(4).and_then(|f| f.texel()), Some(TexelFormat::Rgb565));
    assert_eq!(ati::TexFormat::from_u32(11).and_then(|f| f.texel()), None);
    assert!(ati::TEXTURE_FORMATS.contains(&TexelFormat::Rgb565));
}
