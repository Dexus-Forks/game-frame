//! Display lists and transforms. POD makes one list per drawn piece with `preserve_state`
//! (`sgl_create_list(1, 1, 0)`, 33 sites), hides and shows them each frame with
//! `sgl_set_ignore_list` (52 sites) and moves cars, wheels and the camera by editing named
//! transforms (`sgl_modify_transform`, then `sgl_rotate` / `sgl_translate`, `FUN_004868c0`).

use super::{sgl, trace, v3};
use crate::scene::math::Mat;

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_list(generate_name: i32, preserve_state: i32, separate_list: i32) -> i32 {
    trace!("sgl_create_list", "{generate_name} {preserve_state} {separate_list}");
    sgl().scene.create_list(generate_name != 0, preserve_state != 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_to_parent() {
    trace!("sgl_to_parent");
    sgl().scene.to_parent();
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_modify_list(name: i32, clear: i32) {
    trace!("sgl_modify_list", "{name} {clear}");
    sgl().scene.modify_list(name, clear != 0);
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_delete_list(name: i32) {
    trace!("sgl_delete_list", "{name}");
    sgl().scene.delete_list(name);
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_set_ignore_list(name: i32, ignore: i32) {
    trace!("sgl_set_ignore_list", "{name} {ignore}");
    sgl().scene.set_ignore_list(name, ignore != 0);
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_transform(generate_name: i32) -> i32 {
    trace!("sgl_create_transform", "{generate_name}");
    sgl().scene.create_transform(generate_name != 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_modify_transform(name: i32, clear: i32) {
    trace!("sgl_modify_transform", "{name} {clear}");
    sgl().scene.modify_transform(name, clear != 0);
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_translate(x: f32, y: f32, z: f32) {
    trace!("sgl_translate", "{x} {y} {z}");
    sgl().scene.transform(Mat::translate([x, y, z]));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_rotate(axis: *const f32, angle: f32) {
    let a = unsafe { v3(axis) };
    trace!("sgl_rotate", "{a:?} {angle}");
    sgl().scene.transform(Mat::rotate(a, angle));
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_scale(x: f32, y: f32, z: f32) {
    trace!("sgl_scale", "{x} {y} {z}");
    sgl().scene.transform(Mat::scale([x, y, z]));
}
