//! Meshes, convex objects and light volumes. POD's track is one small list per face, each
//! with a mesh of four vertices and two triangles (`FUN_00497710`): positions, normals and
//! texture coordinates as arrays of floats, faces as vertex indices.

use super::{sgl, trace, v2, v3};
use crate::scene::graph::{Plane, Vertex};

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_mesh(generate_name: i32) -> i32 {
    shim::call!("sgl_create_mesh", generate_name);
    trace!("sgl_create_mesh", "{generate_name}");
    sgl().scene.create_mesh(generate_name != 0)
}

/// `num_to_add` vertices; the normal and texture coordinate arrays may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_add_vertices(num_to_add: i32, vertices: *const f32, normals: *const f32, uvs: *const f32) {
    shim::call!("sgl_add_vertices", num_to_add, vertices, normals, uvs);
    let n = num_to_add.max(0) as usize;
    if vertices.is_null() {
        return;
    }
    let verts: Vec<Vertex> = (0..n)
        .map(|i| unsafe {
            Vertex {
                p: v3(vertices.add(3 * i)),
                n: (!normals.is_null()).then(|| v3(normals.add(3 * i))),
                uv: (!uvs.is_null()).then(|| v2(uvs.add(2 * i))),
            }
        })
        .collect();
    trace!("sgl_add_vertices", "{n} {:?}", verts.first());
    sgl().scene.add_vertices(verts.into_iter());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_add_face(num_face_points: i32, vertex_ids: *const i32) {
    shim::call!("sgl_add_face", num_face_points, vertex_ids);
    if vertex_ids.is_null() || !(3..=4).contains(&num_face_points) {
        return;
    }
    let ids = unsafe { std::slice::from_raw_parts(vertex_ids, num_face_points as usize) };
    trace!("sgl_add_face", "{ids:?}");
    sgl().scene.add_face(ids);
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_light_volume(generate_name: i32, light_volume: i32, light_name: i32) -> i32 {
    shim::call!("sgl_create_light_volume", generate_name, light_volume, light_name);
    trace!("sgl_create_light_volume", "{generate_name} {light_volume} {light_name}");
    sgl().scene.create_light_volume(generate_name != 0, light_volume != 0, light_name)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_add_simple_plane(surface_point: *const f32, normal: *const f32, invisible: i32) {
    shim::call!("sgl_add_simple_plane", surface_point, normal, invisible);
    let (p, n) = unsafe { (v3(surface_point), v3(normal)) };
    trace!("sgl_add_simple_plane", "{p:?} {n:?} {invisible}");
    sgl().scene.add_plane(Plane { p, n, invisible: invisible != 0 });
}

/// A plane through three points, with a normal at each (for smooth shading) and texture
/// coordinates. Only the plane is kept: a light volume has no surface of its own.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_add_plane(
    surface_point: *const f32, point2: *const f32, point3: *const f32, invisible: i32, normal1: *const f32,
    normal2: *const f32, normal3: *const f32, uv1: *const f32, uv2: *const f32, uv3: *const f32,
) {
    shim::call!("sgl_add_plane", surface_point, point2, point3, invisible, normal1, normal2, normal3, uv1, uv2, uv3);
    let (p, p2, p3, n1) = unsafe { (v3(surface_point), v3(point2), v3(point3), v3(normal1)) };
    let _ = (normal2, normal3, uv1, uv2, uv3);
    trace!("sgl_add_plane", "{p:?} {p2:?} {p3:?} {invisible} {n1:?}");
    // The plane is the one through the three points; the first normal says which side is out
    // (*inferred*: POD's normals point away from its cone's axis, FUN_0049b770).
    let n = crate::scene::convex::plane_through(p, p2, p3, n1).unwrap_or(n1);
    sgl().scene.add_plane(Plane { p, n, invisible: invisible != 0 });
}
