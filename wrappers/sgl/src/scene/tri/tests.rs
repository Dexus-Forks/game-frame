//! Host tests of `scene::tri`.

use super::*;

fn v(x: f32, y: f32, inv_w: f32, colour: u32) -> Vertex {
    Vertex { x, y, z: 0.5, inv_w, colour, specular: 0xff000000, u: 0.0, v: 0.0 }
}

fn ctx(flags: u32) -> Context {
    Context { flags, ..Context::default() }
}

#[test]
fn reads_the_context_at_its_offsets() {
    let mut b = vec![0u8; CONTEXT_SIZE];
    b[0x00] = 1;
    b[0x04..0x08].copy_from_slice(&0.5f32.to_le_bytes());
    b[0x10] = 12;
    b[0x14] = 1;
    b[0x18..0x1c].copy_from_slice(&(GOURAUD | TEXTURE).to_le_bytes());
    b[0x1c] = 7;
    b[0x2c..0x30].copy_from_slice(&0.25f32.to_le_bytes());
    b[0x34] = 2;
    b[0x40] = 1;
    b[0x44] = 1;
    b[0x60] = 0x80;
    let c = Context::read(&b);
    assert!(c.fog_on && c.cull && c.uv_times_inv_w);
    assert_eq!((c.fog[0], c.fog_density, c.flags, c.texture), (0.5, 12, GOURAUD | TEXTURE, 7));
    assert_eq!((c.background[1], c.shadow_mode, c.flip, c.global_trans), (0.25, 2, (false, true), 0x80));
}

#[test]
fn reads_a_vertex() {
    let mut b = [0u8; VERTEX_SIZE];
    for (i, f) in [10.0f32, 20.0, 0.5, 2.0].iter().enumerate() {
        b[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
    }
    b[16..20].copy_from_slice(&0x80ff0000u32.to_le_bytes());
    b[24..28].copy_from_slice(&0.75f32.to_le_bytes());
    let v = Vertex::read(&b);
    assert_eq!((v.x, v.y, v.inv_w, v.colour, v.u), (10.0, 20.0, 2.0, 0x80ff0000, 0.75));
    assert_eq!(rgba(v.colour), [1.0, 0.0, 0.0, 128.0 / 255.0]);
}

#[test]
fn culls_and_drops_faces_as_the_chip_does() {
    let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0), v(0.1, 0.0, 1.0, 0)];
    let mut f = Frame::default();
    // Clockwise on the screen (y down): kept either way.
    f.add(&Context { cull: true, ..ctx(0) }, &[[0, 1, 2]], &vs, false);
    // Counter-clockwise: culled, or kept turned round.
    f.add(&Context { cull: true, ..ctx(0) }, &[[0, 2, 1]], &vs, false);
    f.add(&ctx(0), &[[0, 2, 1]], &vs, false);
    // Less than 1/32 of a pixel: dropped.
    f.add(&ctx(0), &[[0, 3, 0]], &vs, false);
    // An index past the vertices: skipped.
    f.add(&ctx(0), &[[0, 1, 9]], &vs, false);
    assert_eq!((f.sent, f.kept), (5, 2));
}

#[test]
fn places_vertices_so_the_projection_gives_them_back() {
    let mut f = Frame::default();
    f.add(&ctx(0), &[[0, 1, 2]], &[v(100.0, 50.0, 0.5, 0), v(200.0, 50.0, 0.25, 0), v(100.0, 150.0, 0.5, 0)], false);
    let view = f.view(&ctx(0));
    let p = view.batches[0].verts[1].p;
    // The shader puts (x, y, z) at (x / z, -y / z) with this camera.
    assert_eq!(p, [800.0, -200.0, 4.0]);
    assert_eq!(view.camera.zoom, 1.0);
    assert!((view.camera.foreground - 2.0 * 0.99).abs() < 1e-6);
}

#[test]
fn shades_flat_from_the_first_vertex_or_the_average() {
    let vs = [v(0.0, 0.0, 1.0, 0xffff0000), v(10.0, 0.0, 1.0, 0xff0000ff), v(0.0, 10.0, 1.0, 0xff00ff00)];
    let mut f = Frame::default();
    f.add(&ctx(0), &[[0, 1, 2]], &vs, false);
    f.add(&ctx(GOURAUD), &[[0, 1, 2]], &vs, false);
    f.add(&ctx(AVERAGECOLOUR), &[[0, 1, 2]], &vs, false);
    let all: Vec<&Vert> = f.batches.iter().flat_map(|b| &b.verts).collect();
    assert_eq!(all[1].modulate, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(all[4].modulate, [0.0, 0.0, 1.0, 1.0]);
    let third = 1.0 / 3.0;
    assert!(all[7].modulate.iter().zip([third, third, third, 1.0]).all(|(a, b)| (a - b).abs() < 1e-6));
}

#[test]
fn one_translucency_level_per_face() {
    let vs = [v(0.0, 0.0, 1.0, 0x80ffffff), v(10.0, 0.0, 1.0, 0x80ffffff), v(0.0, 10.0, 1.0, 0x80ffffff)];
    let full = [v(0.0, 0.0, 1.0, 0xffffffff); 3];
    // Global alone: 15 - (gt >> 4); opaque from 0xf0.
    assert_eq!(trans_level(GLOBALTRANS, 0x80, &[]), Some(7));
    assert_eq!(trans_level(GLOBALTRANS, 0xf0, &[]), None);
    // Vertex alpha: 3 × 0x80 × 170 >> 13 is 7, so level 8.
    assert_eq!(trans_level(VERTEXTRANS, 0, &vs), Some(8));
    // Full alpha is level 0, but still a translucent face.
    assert_eq!(trans_level(VERTEXTRANS, 0, &full), Some(0));
    // Both: gt = 0x80 scales by 85, so full alpha gives level 8, one more than global alone.
    assert_eq!(trans_level(VERTEXTRANS | GLOBALTRANS, 0x80, &full), Some(8));
    assert_eq!(level_alpha(0), 1.0);
    assert_eq!(level_alpha(15), 0.0);
}

#[test]
fn translucency_and_decals() {
    let vs = [v(0.0, 0.0, 1.0, 0x80ffffff), v(10.0, 0.0, 1.0, 0xffffffff), v(0.0, 10.0, 1.0, 0x00ffffff)];
    let mut f = Frame::default();
    f.add(&ctx(VERTEXTRANS | GOURAUD), &[[0, 1, 2]], &vs, false);
    f.add(&Context { global_trans: 0xf0, ..ctx(GLOBALTRANS) }, &[[0, 1, 2]], &vs, false);
    f.add(&Context { texture: 3, ..ctx(TEXTURE | DECAL | VERTEXTRANS | HIGHLIGHT) }, &[[0, 1, 2]], &vs, false);
    f.add(&Context { texture: 3, ..ctx(TEXTURE) }, &[[0, 1, 2]], &vs, true);
    let b = &f.batches;
    // One alpha for the whole face, from the sum of its alphas.
    assert!(b[0].translucent && b[0].verts.iter().all(|v| v.modulate[3] == b[0].verts[0].modulate[3]));
    assert!(!b[1].translucent);
    assert_eq!((b[2].verts[0].modulate, b[2].texture, b[2].translucent), ([1.0; 4], 3, false));
    assert!(b[3].translucent);
}

#[test]
fn translucent_passes_split_by_facing_and_sort_farthest_first() {
    let near = [v(0.0, 0.0, 0.5, 0), v(10.0, 0.0, 0.5, 0), v(0.0, 10.0, 0.5, 0)];
    let far = [v(0.0, 0.0, 0.1, 0), v(10.0, 0.0, 0.1, 0), v(0.0, 10.0, 0.2, 0)];
    let c = Context { global_trans: 0x80, ..ctx(GLOBALTRANS) };
    let mut f = Frame::default();
    // One call with a face kept and a face turned round: two passes.
    f.add(&c, &[[0, 1, 2], [0, 2, 1]], &near, false);
    f.add(&c, &[[0, 1, 2]], &far, false);
    f.add(&ctx(0), &[[0, 1, 2]], &far, false);
    f.add(&Context { global_trans: 0x80, ..ctx(GLOBALTRANS | OPAQUE) }, &[[0, 1, 2]], &near, false);
    assert_eq!(f.batches.len(), 5);
    let view = f.view(&ctx(0));
    let pass: Vec<u32> = view.batches.iter().map(|b| b.pass).collect();
    // The far call (key 0.1) first, then the near call's two facings, the later one first;
    // the opaque batch at 0 and the SGLTT_OPAQUE batch at 1.
    assert_eq!(pass, [4, 3, 2, 0, 1]);
}

#[test]
fn opaque_calls_of_one_state_share_a_batch() {
    let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0), v(10.0, 10.0, 1.0, 0)];
    let mut f = Frame::default();
    f.add(&ctx(0), &[[0, 1, 2]], &vs, false);
    f.add(&ctx(0), &[[0, 1, 3, 2]], &vs, false);
    f.add(&ctx(DISABLEZBUFFER), &[[0, 1, 2]], &vs, false);
    assert_eq!(f.batches.len(), 2);
    assert_eq!(f.batches[0].verts.len(), 9);
    assert!(f.batches[1].no_depth);
}

#[test]
fn texture_coordinates_and_wrapping() {
    let mut a = v(0.0, 0.0, 0.5, 0);
    (a.u, a.v) = (0.25, 0.5);
    assert_eq!(uv(&ctx(0), &a), [0.5, 1.0]);
    assert_eq!(uv(&Context { uv_times_inv_w: true, flip: (true, false), ..ctx(0) }, &a), [0.75, 0.5]);
    let mut uvs = [[0.9, 0.1], [0.1, 0.9], [0.5, 0.5]];
    wrap(&mut uvs, WRAPU);
    assert!((uvs[1][0] - 1.1).abs() < 1e-6 && uvs[1][1] == 0.9);
}

/// The shift `wrap` gives vertices 1 and 2 on one axis.
fn shifts(u: [f32; 3]) -> [f32; 2] {
    let mut uvs = u.map(|x| [x, 0.0]);
    wrap(&mut uvs, WRAPU);
    [uvs[1][0] - u[1], uvs[2][0] - u[2]]
}

#[test]
fn wrapping_moves_one_step_only_as_the_driver_measured() {
    // Rows of the measured table (`sgl2-measure.md`, 10036b00 against 10036cd0).
    assert_eq!(shifts([0.0, 0.49999997, -0.49999997]), [0.0, 0.0]);
    assert_eq!(shifts([0.0, 0.5, -0.5]), [-1.0, 1.0]);
    assert_eq!(shifts([0.0, 1.49, -1.49]), [-1.0, 1.0]);
    assert_eq!(shifts([0.0, 1.5, -1.5]), [-1.0, 1.0]);
    assert_eq!(shifts([0.0, 1.7, -2.6]), [-1.0, 1.0]);
    assert_eq!(shifts([0.0, 2.5, 3.2]), [-1.0, -1.0]);
    assert_eq!(shifts([5.25, 5.0, 4.6]), [0.0, 1.0]);
    assert_eq!(shifts([0.0, -0.0, 0.0]), [0.0, 0.0]);
    // d = 0.6 - 0.1 rounds to 0.5 in f32.
    let s = shifts([0.1, 0.6, -0.4]);
    assert!((s[0] + 1.0).abs() < 1e-6 && (s[1] - 1.0).abs() < 1e-6);
    // WRAPV alone leaves U.
    let mut uvs = [[0.0, 0.0], [0.9, 0.9], [0.0, 0.0]];
    wrap(&mut uvs, WRAPV);
    assert_eq!(uvs[1], [0.9, 0.9 - 1.0]);
}

#[test]
fn shadow_mode_draws_smooth_translucent_textured_faces_flat() {
    let vs = [v(0.0, 0.0, 1.0, 0xffff0000), v(10.0, 0.0, 1.0, 0xff0000ff), v(0.0, 10.0, 1.0, 0xff00ff00)];
    let c = Context { texture: 3, shadow_mode: 1, ..ctx(GOURAUD | TEXTURE) };
    assert!(shadow_drops_gouraud(&c, c.flags, true));
    assert!(!shadow_drops_gouraud(&c, c.flags, false));
    assert!(!shadow_drops_gouraud(&Context { shadow_mode: 0, ..c }, c.flags, true));
    assert!(!shadow_drops_gouraud(&c, c.flags | GLOBALTRANS, true));
    assert!(!shadow_drops_gouraud(&c, c.flags | VERTEXTRANS, true));
    assert!(!shadow_drops_gouraud(&c, GOURAUD, true));
    let mut f = Frame::default();
    f.add(&c, &[[0, 1, 2]], &vs, true);
    f.add(&Context { shadow_mode: 2, ..c }, &[[0, 1, 2]], &vs, true);
    f.add(&Context { shadow_mode: 0, ..c }, &[[0, 1, 2]], &vs, true);
    let col = |b: usize, i: usize| f.batches[b].verts[i].modulate;
    // Flat: every vertex takes the first one's red.
    assert_eq!((col(0, 1), col(1, 2)), ([1.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, 1.0]));
    assert_eq!(col(2, 1), [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn volumes_take_the_last_brightness_or_colour_of_the_frame() {
    let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)];
    // A thin wedge: one face is enough for a volume of one plane.
    let face = [v(100.0, 100.0, 0.2, 0), v(200.0, 100.0, 0.2, 0), v(100.0, 200.0, 0.25, 0)];
    let mut f = Frame::default();
    let shadow = Context { shadow_mode: 1, shadow_value: 0.5f32.to_bits(), ..ctx(0) };
    let light = Context { shadow_mode: 2, shadow_value: 0x00ff0000, ..ctx(0) };
    f.shadow(&shadow, &[[0, 1, 2]], &face);
    f.shadow(&light, &[[0, 1, 2]], &face);
    f.shadow(&Context { shadow_mode: 2, shadow_value: 0x000000ff, ..ctx(0) }, &[[0, 1, 9]], &face);
    let view = f.view(&ctx(0));
    assert_eq!(view.volumes.len(), 2);
    assert_eq!((view.volumes[0].kind, view.volumes[0].colour), (VolumeKind::Shadow, [0.5; 3]));
    assert_eq!((view.volumes[1].kind, view.volumes[1].colour), (VolumeKind::Light, [1.0, 0.0, 0.0]));
    // A later triangle call in shadow mode sets the brightness of every shadow volume.
    f.add(&Context { shadow_value: 0.25f32.to_bits(), ..shadow }, &[[0, 1, 2]], &vs, false);
    f.add(&Context { shadow_value: 0x0000ff00, ..light }, &[[0, 1, 2]], &vs, false);
    let view = f.view(&ctx(0));
    assert_eq!((view.volumes[0].colour, view.volumes[1].colour), ([0.25; 3], [0.0, 1.0, 0.0]));
    assert_eq!(f.shadows, 3);
    f.start(&ctx(0));
    assert!(f.view(&ctx(0)).volumes.is_empty());
}

#[test]
fn fog_ignores_the_vertex_fog_flag() {
    let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)];
    let mut f = Frame::default();
    f.add(&Context { fog_on: true, ..ctx(VERTEXFOG) }, &[[0, 1, 2]], &vs, false);
    f.add(&ctx(VERTEXFOG), &[[0, 1, 2]], &vs, false);
    assert!(f.batches[0].fog && !f.batches[1].fog);
}

#[test]
fn fog_rate_doubles_per_step_of_the_register() {
    let r = rate(10);
    assert!((rate(11) / r - 2.0).abs() < 1e-5);
    // Register 12 halves the colour every 10.24 units of w (0.01 × 1024 / c, c = 1).
    assert!((std::f32::consts::LN_2 / rate(12) - 10.24).abs() < 0.001);
    let mut f = Frame::default();
    f.add(&Context { fog_on: true, ..ctx(0) }, &[[0, 1, 2]], &[v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)], false);
    // A density out of range writes register 0, as sgltri_render does: a faint fog.
    let lost = f.view(&Context { fog_density: 40, ..ctx(0) }).camera.fog.unwrap().1;
    assert!((lost - rate(0)).abs() / rate(0) < 1e-3);
}
