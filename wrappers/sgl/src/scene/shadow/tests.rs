//! Host tests of `scene::shadow`.

use super::*;

/// A camera-space point projected as Direct3D's transform stage would (as `sgltritest.c`
/// does): 640 × 480, focal length 400.
fn project(p: [f32; 3]) -> Vertex {
    let k = 400.0 / p[2];
    Vertex { x: 320.0 + p[0] * k, y: 240.0 - p[1] * k, z: 1.0 - 1.0 / p[2], inv_w: 1.0 / p[2], ..Vertex::default() }
}

/// The same point where `tri` places it: `(x w, -y w, w)` of its screen place.
fn placed(p: [f32; 3]) -> [f32; 3] {
    let v = project(p);
    let w = 1.0 / v.inv_w;
    [v.x * w, -v.y * w, w]
}

/// Twelve triangles of a box from `lo` to `hi`, each wound so that its screen-space normal
/// `(v1 - v0) × (v2 - v0)` in `(x, y, fZ)` points out of the box, as callers must.
fn boxed(lo: [f32; 3], hi: [f32; 3]) -> (Vec<[u32; 3]>, Vec<Vertex>) {
    let corner = |i: u32| [if i & 1 != 0 { hi[0] } else { lo[0] }, if i & 2 != 0 { hi[1] } else { lo[1] }, if i & 4 != 0 { hi[2] } else { lo[2] }];
    let verts: Vec<Vertex> = (0..8).map(|i| project(corner(i))).collect();
    let s = |v: &Vertex| [v.x as f64, v.y as f64, v.z as f64];
    let centre = verts.iter().map(s).fold([0.0; 3], |a, b| [a[0] + b[0] / 8.0, a[1] + b[1] / 8.0, a[2] + b[2] / 8.0]);
    let quads = [[0, 1, 3, 2], [4, 5, 7, 6], [0, 1, 5, 4], [2, 3, 7, 6], [0, 2, 6, 4], [1, 3, 7, 5]];
    let mut faces = Vec::new();
    for q in quads {
        for t in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
            let p = t.map(|i| s(&verts[i as usize]));
            let n = cross(sub(p[1], p[0]), sub(p[2], p[0]));
            let mid = [(p[0][0] + p[1][0] + p[2][0]) / 3.0, (p[0][1] + p[1][1] + p[2][1]) / 3.0, (p[0][2] + p[1][2] + p[2][2]) / 3.0];
            faces.push(if dot(n, sub(mid, centre)) > 0.0 { t } else { [t[0], t[2], t[1]] });
        }
    }
    (faces, verts)
}

fn inside(planes: &[[f32; 4]], p: [f32; 3]) -> bool {
    let q = placed(p);
    planes.iter().all(|n| n[0] * q[0] + n[1] * q[1] + n[2] * q[2] <= n[3] + 1e-4 * n[3].abs().max(1.0))
}

#[test]
fn a_box_is_the_space_inside_its_faces() {
    let (faces, verts) = boxed([-1.0, -2.0, 5.0], [1.0, -1.0, 7.0]);
    let planes = volume(&faces, &verts);
    assert_eq!(planes.len(), 12);
    assert!(inside(&planes, [0.0, -1.5, 6.0]));
    assert!(inside(&planes, [0.9, -1.9, 6.9]));
    for out in [[0.0, -1.5, 4.5], [0.0, -1.5, 7.5], [1.5, -1.5, 6.0], [-1.5, -1.5, 6.0], [0.0, -0.5, 6.0], [0.0, -2.5, 6.0]] {
        assert!(!inside(&planes, out), "{out:?}");
    }
}

#[test]
fn a_face_through_the_eye_is_a_cut_across_the_screen() {
    // The face x = 0 projects to the line x = 320: edge-on.
    let (faces, verts) = boxed([0.0, -2.0, 5.0], [1.0, -1.0, 7.0]);
    let planes = volume(&faces, &verts);
    let cuts = planes.iter().filter(|p| p[3] == 0.0).count();
    assert_eq!(cuts, 2);
    assert!(inside(&planes, [0.5, -1.5, 6.0]));
    assert!(!inside(&planes, [-0.5, -1.5, 6.0]));
}

#[test]
fn a_face_too_steep_for_the_depth_plane_is_a_cut() {
    // Nearly edge-on: a hair off the line x = 320, far apart in depth.
    let v = [project([0.0, 0.0, 2.0]), project([0.0, 1.0, 50.0]), project([0.0, -1.0, 50.0])];
    let mut tilted = v;
    tilted[1].x += 1e-3;
    assert_eq!(face_plane(&tilted).unwrap()[3], 0.0);
    // Edge-on exactly: the same kind of plane.
    assert_eq!(face_plane(&v).unwrap()[3], 0.0);
}

#[test]
fn faces_it_cannot_use_give_no_plane() {
    let (faces, mut verts) = boxed([-1.0, -2.0, 5.0], [1.0, -1.0, 7.0]);
    verts[0].inv_w = 0.0;
    let used = faces.iter().filter(|f| !f.contains(&0)).count();
    assert_eq!(volume(&faces, &verts).len(), used);
    assert_eq!(volume(&[[0, 1, 99]], &verts).len(), 0);
}

#[test]
fn the_brightness_and_the_light_colour() {
    assert_eq!(kind(1), VolumeKind::Shadow);
    assert_eq!(kind(2), VolumeKind::Light);
    assert_eq!(volume_colour(VolumeKind::Shadow, 0.25f32.to_bits()), [0.25; 3]);
    assert_eq!(volume_colour(VolumeKind::Shadow, 2.0f32.to_bits()), [1.0; 3]);
    assert_eq!(volume_colour(VolumeKind::Shadow, f32::NAN.to_bits()), [1.0; 3]);
    assert_eq!(volume_colour(VolumeKind::Light, 0x00ff8000), [1.0, 128.0 / 255.0, 0.0]);
}

#[test]
fn shadow_arguments_in_the_drivers_order() {
    assert_eq!(shadow_args(0, None, false), Some(0));
    assert_eq!(shadow_args(101, Some(1), true), Some(-252));
    assert_eq!(shadow_args(5, Some(0), false), Some(0));
    assert_eq!(shadow_args(5, None, true), Some(-252));
    assert_eq!(shadow_args(5, Some(1), false), Some(-252));
    assert_eq!(shadow_args(-1, Some(2), true), Some(-252));
    assert_eq!(shadow_args(100, Some(2), true), None);
}
