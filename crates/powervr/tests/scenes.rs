//! Small scenes through the whole model, checked pixel by pixel.

use gameframe_powervr::{MapType, Sgl};

/// An 8 × 8 device with one viewport over it, a square camera rectangle and zoom 1, and a
/// camera at the origin looking along +z.
fn device(scale: f32) -> (Sgl, i32, i32) {
    let mut s = Sgl::create_screen_device(8, 8, scale);
    let vp = s.sgl_create_viewport(0, 0, 7, 7, 0.0, 0.0, 8.0, 8.0);
    s.sgl_create_list(false, true, false);
    s.sgl_create_transform(false);
    let cam = s.sgl_create_camera(1.0, 0.1, 0.0);
    s.sgl_to_parent();
    (s, vp, cam)
}

/// A quad at depth `z` covering x and y from `-h` to `h`, facing the camera.
fn quad(s: &mut Sgl, z: f32, h: f32) {
    s.sgl_create_mesh(false);
    let v = [[-h, -h, z], [h, -h, z], [h, h, z], [-h, h, z]];
    let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    s.sgl_add_vertices(&v, Some(&[[0.0, 0.0, -1.0]; 4]), Some(&uv));
    s.sgl_add_face(&[0, 1, 2, 3]);
}

fn rgb(s: &Sgl, x: usize, y: usize) -> [u8; 3] {
    let p = s.picture();
    let o = (y * s.frame.width + x) * 4;
    [p[o], p[o + 1], p[o + 2]]
}

#[test]
fn background_and_ambient() {
    let (mut s, vp, cam) = device(1.0);
    s.sgl_set_background_colour(cam, [0.0, 0.0, 1.0]);
    s.sgl_create_ambient_light(false, [0.5, 0.5, 0.5], false);
    s.sgl_set_ambient([1.0, 0.0, 0.0]);
    s.sgl_set_diffuse([0.0; 3]);
    quad(&mut s, 2.0, 0.5);
    assert_eq!(s.sgl_render(vp, cam), 0);
    // The quad covers x and y from -0.25 to 0.25 of the view at zoom 1: pixels 3 and 4.
    assert_eq!(rgb(&s, 3, 3), [132, 0, 0], "ambient 0.5 of red, in 565");
    assert_eq!(rgb(&s, 0, 0), [0, 0, 255], "the background");
}

#[test]
fn the_nearest_opaque_surface_wins_in_any_order() {
    for near_first in [false, true] {
        let (mut s, vp, cam) = device(1.0);
        s.sgl_create_ambient_light(false, [1.0; 3], false);
        s.sgl_set_diffuse([0.0; 3]);
        // Each quad in its own list: a material call while a mesh is being edited goes to
        // that mesh's later faces, as in SGL.
        let put = |s: &mut Sgl, near: bool| {
            s.sgl_create_list(false, true, false);
            s.sgl_set_ambient(if near { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] });
            quad(s, if near { 2.0 } else { 4.0 }, if near { 0.5 } else { 4.0 });
            s.sgl_to_parent();
        };
        put(&mut s, near_first);
        put(&mut s, !near_first);
        s.sgl_render(vp, cam);
        assert_eq!(rgb(&s, 4, 4), [0, 255, 0]);
        assert_eq!(rgb(&s, 0, 0), [255, 0, 0]);
    }
}

#[test]
fn a_texture_multiplied_by_its_effects_and_glow_added() {
    let (mut s, vp, cam) = device(1.0);
    // 2 × 2: red, green / blue, white; alpha 0 is opaque in SGL's intermediate map.
    let map = [255, 0, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 255, 255, 255, 0];
    let t = s.sgl_create_texture(MapType::Map16Bit, 2, 2, &map);
    s.sgl_create_ambient_light(false, [1.0; 3], false);
    s.sgl_set_ambient([0.5; 3]);
    s.sgl_set_diffuse([0.0; 3]);
    s.sgl_set_glow([0.0, 0.0, 0.25]);
    s.sgl_set_texture_map(t, false, false);
    // Ambient through the texture, glow added after it.
    s.sgl_set_texture_effect(true, true, false, false);
    quad(&mut s, 1.0, 1.0);
    s.sgl_render(vp, cam);
    // v 0 at the top: row 0 of the map (red, green) on the screen's top half.
    assert_eq!(rgb(&s, 1, 1), [132, 0, 66]);
    assert_eq!(rgb(&s, 6, 1), [0, 130, 66]);
    assert_eq!(rgb(&s, 1, 6), [0, 0, 189]);
}

#[test]
fn fog_keeps_one_minus_density_per_unit() {
    let (mut s, vp, cam) = device(1.0);
    s.sgl_set_fog(cam, [1.0, 1.0, 1.0], 0.5);
    s.sgl_create_ambient_light(false, [1.0; 3], false);
    s.sgl_set_ambient([0.0; 3]);
    s.sgl_set_diffuse([0.0; 3]);
    quad(&mut s, 2.0, 8.0);
    s.sgl_render(vp, cam);
    // Black at depth 2 keeps 0.25 of itself: 0.75 white.
    assert_eq!(rgb(&s, 4, 4), [189, 190, 189]);
    // Without fog for it, black.
    let (mut s, vp, cam) = device(1.0);
    s.sgl_set_fog(cam, [1.0, 1.0, 1.0], 0.5);
    s.sgl_qual_fog(false);
    s.sgl_set_ambient([0.0; 3]);
    s.sgl_set_diffuse([0.0; 3]);
    quad(&mut s, 2.0, 8.0);
    s.sgl_render(vp, cam);
    assert_eq!(rgb(&s, 4, 4), [0, 0, 0]);
}

#[test]
fn translucent_passes_blend_over_what_is_behind() {
    let (mut s, vp, cam) = device(1.0);
    s.sgl_create_ambient_light(false, [1.0; 3], false);
    s.sgl_set_diffuse([0.0; 3]);
    s.sgl_create_list(false, true, false);
    s.sgl_set_ambient([1.0, 0.0, 0.0]);
    quad(&mut s, 4.0, 8.0);
    s.sgl_to_parent();
    s.sgl_new_translucent();
    s.sgl_set_ambient([0.0, 0.0, 1.0]);
    s.sgl_set_opacity(0.5);
    quad(&mut s, 2.0, 8.0);
    s.sgl_render(vp, cam);
    assert_eq!(rgb(&s, 4, 4), [132, 0, 132]);
}

#[test]
fn a_spot_light_from_the_camera() {
    let (mut s, vp, cam) = device(1.0);
    s.sgl_create_point_light(false, [1.0; 3], [0.0, 0.0, 1.0], [0.0; 3], 32, false, false);
    s.sgl_set_ambient([0.0; 3]);
    s.sgl_set_diffuse([1.0; 3]);
    // Lit per vertex: a small quad on the axis (corners 15.8 degrees off, cos 0.962: N.L 0.962
    // times the spot (1 - (1 - cos) 5)^2 = 0.659), and one beyond the 37 degrees it reaches.
    quad(&mut s, 1.0, 0.2);
    s.sgl_create_mesh(false);
    let v = [[2.0, 2.0, 1.0], [3.0, 2.0, 1.0], [3.0, 3.0, 1.0]];
    s.sgl_add_vertices(&v, Some(&[[0.0, 0.0, -1.0]; 3]), None);
    s.sgl_add_face(&[0, 1, 2]);
    s.sgl_render(vp, cam);
    assert_eq!(rgb(&s, 4, 4), [165, 162, 165]);
    let v = gameframe_powervr::scene::walk::view(&s.scene, cam as u32).unwrap();
    assert!(v.batches[0].verts[6..].iter().all(|v| v.modulate[..3] == [0.0; 3]));
}

#[test]
fn any_multiple_of_the_device() {
    let (mut s, vp, cam) = device(3.0);
    s.sgl_create_ambient_light(false, [1.0; 3], false);
    s.sgl_set_ambient([1.0; 3]);
    s.sgl_set_diffuse([0.0; 3]);
    quad(&mut s, 2.0, 0.5);
    s.sgl_render(vp, cam);
    assert_eq!((s.frame.width, s.frame.height), (24, 24));
    // Pixels 3 and 4 of 8 become 9 to 14 of 24.
    assert_eq!(rgb(&s, 9, 9), [255; 3]);
    assert_eq!(rgb(&s, 14, 14), [255; 3]);
    assert_eq!(rgb(&s, 8, 8), [0; 3]);
    assert_eq!(rgb(&s, 15, 15), [0; 3]);
}

#[test]
fn ignored_lists_are_not_drawn() {
    let (mut s, vp, cam) = device(1.0);
    s.sgl_create_ambient_light(false, [1.0; 3], false);
    let l = s.sgl_create_list(true, true, false);
    s.sgl_set_ambient([1.0; 3]);
    s.sgl_set_diffuse([0.0; 3]);
    quad(&mut s, 2.0, 8.0);
    s.sgl_to_parent();
    s.sgl_set_ignore_list(l, true);
    s.sgl_render(vp, cam);
    assert_eq!(rgb(&s, 4, 4), [0; 3]);
}

