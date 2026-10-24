//! The material calls. POD sets each piece's material right after `sgl_create_mesh`, so it
//! belongs to that mesh's faces: specular (shininess 1000), diffuse, ambient, the texture
//! effect (all four terms) and the texture map (`FUN_00497710`).

use super::{sgl, trace, v3};
use crate::scene::graph::MatOp;

fn op(o: MatOp) {
    sgl().scene.material(o);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_ambient(colour: *const f32) {
    let c = unsafe { v3(colour) };
    trace!("sgl_set_ambient", "{c:?}");
    op(MatOp::Ambient(c));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_diffuse(colour: *const f32) {
    let c = unsafe { v3(colour) };
    trace!("sgl_set_diffuse", "{c:?}");
    op(MatOp::Diffuse(c));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_specular(colour: *const f32, shininess: i32) {
    let c = unsafe { v3(colour) };
    trace!("sgl_set_specular", "{c:?} {shininess}");
    op(MatOp::Specular(c, shininess));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_glow(colour: *const f32) {
    let c = unsafe { v3(colour) };
    trace!("sgl_set_glow", "{c:?}");
    op(MatOp::Glow(c));
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_set_opacity(opacity: f32) {
    trace!("sgl_set_opacity", "{opacity}");
    op(MatOp::Opacity(opacity));
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_set_texture_map(texture_name: i32, flip_u: i32, flip_v: i32) {
    trace!("sgl_set_texture_map", "{texture_name} {flip_u} {flip_v}");
    op(MatOp::Texture(texture_name, flip_u != 0, flip_v != 0));
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_set_texture_effect(affect_ambient: i32, affect_diffuse: i32, affect_specular: i32, affect_glow: i32) {
    trace!("sgl_set_texture_effect", "{affect_ambient} {affect_diffuse} {affect_specular} {affect_glow}");
    op(MatOp::Effect([affect_ambient != 0, affect_diffuse != 0, affect_specular != 0, affect_glow != 0]));
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_new_translucent() {
    trace!("sgl_new_translucent");
    sgl().scene.new_translucent();
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_qual_fog(enable: i32) {
    trace!("sgl_qual_fog", "{enable}");
    sgl().scene.qual_fog(enable != 0);
}
