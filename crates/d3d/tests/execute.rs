//! Execute buffers end to end: written, parsed back and drawn.

use gameframe_d3d::consts::*;
use gameframe_d3d::exec::{self, Instruction};
use gameframe_d3d::{Builder, Device, Format, Texture, TlVertex};

fn v(sx: f32, sy: f32, color: u32) -> TlVertex {
    TlVertex { sx, sy, sz: 0.5, rhw: 1.0, color, specular: 0xff00_0000, ..TlVertex::default() }
}

fn square(b: &mut Builder, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) -> [u16; 4] {
    [b.vertex(v(x0, y0, color)), b.vertex(v(x1, y0, color)), b.vertex(v(x1, y1, color)), b.vertex(v(x0, y1, color))]
}

#[test]
fn a_buffer_parses_back_as_written() {
    let mut b = Builder::new();
    let q = square(&mut b, 0.0, 0.0, 4.0, 4.0, 0xffffffff);
    b.process_vertices(PROCESS_COPY, 0, 0, 4);
    b.render_states(&[(rs::SHADEMODE, SHADE_FLAT), (rs::ZENABLE, 0)]);
    b.fan(&q);
    let (buf, data) = b.finish();
    assert_eq!(data.vertex_count, 4);
    assert_eq!(data.instruction_offset, 128);
    let parsed = exec::parse(&buf, &data);
    assert_eq!(parsed[0], Instruction::ProcessVertices { flags: PROCESS_COPY, start: 0, dest: 0, count: 4 });
    assert_eq!(parsed[1], Instruction::RenderStates(vec![(rs::SHADEMODE, SHADE_FLAT), (rs::ZENABLE, 0)]));
    assert_eq!(parsed[2], Instruction::Triangles(vec![([0, 1, 2], 0), ([0, 2, 3], 0)]));
    assert_eq!(parsed[3], Instruction::Exit);
    assert_eq!(exec::vertices(&buf, &data)[2].sx, 4.0);
}

#[test]
fn the_depth_test_keeps_the_nearer_square() {
    let mut b = Builder::new();
    let mut far = square(&mut b, 0.0, 0.0, 8.0, 8.0, 0xff0000ff);
    let near: Vec<u16> = (0..4).map(|i| {
        let mut t = v([2.0, 6.0, 6.0, 2.0][i], [2.0, 2.0, 6.0, 6.0][i], 0xffff0000);
        t.sz = 0.25;
        b.vertex(t)
    }).collect();
    b.process_vertices(PROCESS_COPY, 0, 0, 8);
    b.render_states(&[(rs::ZENABLE, 1), (rs::CULLMODE, CULL_NONE)]);
    b.fan(&near);
    far.reverse();
    b.fan(&far);
    let (buf, data) = b.finish();
    let mut d = Device::new(8, 8, 1.0);
    d.execute(&buf, &data);
    assert_eq!(d.colour[3 * 8 + 3], 0xf800, "the near red square stays in front");
    assert_eq!(d.colour[0], 0x001f, "the far blue square shows around it");
}

#[test]
fn a_scaled_device_draws_the_same_picture_larger() {
    let mut b = Builder::new();
    let q = square(&mut b, 1.0, 1.0, 3.0, 3.0, 0xff00ff00);
    b.process_vertices(PROCESS_COPY, 0, 0, 4);
    b.render_states(&[(rs::CULLMODE, CULL_NONE)]);
    b.fan(&q);
    let (buf, data) = b.finish();
    let mut d = Device::new(4, 4, 2.0);
    d.execute(&buf, &data);
    assert_eq!((d.target_w, d.target_h), (8, 8));
    let drawn = d.colour.iter().filter(|&&c| c == 0x07e0).count();
    assert_eq!(drawn, 16);
}

#[test]
fn texture_blends_alpha_test_and_blending() {
    let tex = Texture { width: 2, height: 1, format: Format::Argb1555, texels: vec![0x0000, 0xfc00], key: None };
    let mut d = Device::new(4, 1, 1.0);
    let h = d.add_texture(tex);
    let mut b = Builder::new();
    let mut corner = |sx: f32, sy: f32, tu: f32| {
        b.vertex(TlVertex { sx, sy, sz: 0.5, rhw: 1.0, color: 0xff80_8080, specular: 0xff00_0000, tu, tv: 0.5 })
    };
    let q = [corner(0.0, -1.0, 0.0), corner(4.0, -1.0, 1.0), corner(4.0, 2.0, 1.0), corner(0.0, 2.0, 0.0)];
    b.process_vertices(PROCESS_COPY, 0, 0, 4);
    b.render_states(&[
        (rs::CULLMODE, CULL_NONE),
        (rs::TEXTUREHANDLE, h),
        (rs::TEXTUREMAPBLEND, TBLEND_MODULATE),
        (rs::ALPHATESTENABLE, 1),
        (rs::ALPHAREF, 0),
        (rs::ALPHAFUNC, CMP_NOTEQUAL),
    ]);
    b.fan(&q);
    let (buf, data) = b.finish();
    d.clear(0x001f, 1.0);
    d.execute(&buf, &data);
    // Left half: texel alpha 0, dropped by the alpha test; right half: red × grey.
    assert_eq!(d.colour[0], 0x001f);
    assert_eq!(d.colour[3] >> 11, 0x80 >> 3);
    // ONE + INVSRCCOLOR over blue: white lightens it to white.
    let mut b = Builder::new();
    let q = square(&mut b, 0.0, -1.0, 4.0, 2.0, 0xffffffff);
    b.process_vertices(PROCESS_COPY, 0, 0, 4);
    b.render_states(&[
        (rs::TEXTUREHANDLE, 0),
        (rs::ALPHATESTENABLE, 0),
        (rs::BLENDENABLE, 1),
        (rs::SRCBLEND, BLEND_ONE),
        (rs::DESTBLEND, BLEND_INVSRCCOLOR),
    ]);
    b.fan(&q);
    let (buf, data) = b.finish();
    d.execute(&buf, &data);
    assert_eq!(d.colour[0], 0xffff);
}

#[test]
fn flat_shading_takes_the_first_vertex() {
    let mut b = Builder::new();
    let i = [b.vertex(v(0.0, 0.0, 0xffff0000)), b.vertex(v(8.0, 0.0, 0xff0000ff)), b.vertex(v(0.0, 8.0, 0xff0000ff))];
    b.process_vertices(PROCESS_COPY, 0, 0, 3);
    b.render_states(&[(rs::SHADEMODE, SHADE_FLAT), (rs::CULLMODE, CULL_NONE)]);
    b.triangle(i);
    let (buf, data) = b.finish();
    let mut d = Device::new(8, 8, 1.0);
    d.execute(&buf, &data);
    assert_eq!(d.colour[2 * 8 + 3], 0xf800);
}
