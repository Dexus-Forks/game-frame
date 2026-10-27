//! Lights and cameras. POD makes an ambient light for the whole scene (`FUN_00499f90`), point
//! lights it moves and recolours each frame (`FUN_004888e0`), and two cameras, one per player,
//! each the child of a named transform (`FUN_0049a020`: zoom 0.7). Fog and the background
//! colour belong to a camera (`FUN_0049a430`).

use super::{sgl, trace, v3};
use crate::scene::graph::{Light, LightKind};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_create_ambient_light(generate_name: i32, colour: *const f32, relative: i32) -> i32 {
    shim::call!("sgl_create_ambient_light", generate_name, colour, relative);
    let c = unsafe { v3(colour) };
    trace!("sgl_create_ambient_light", "{generate_name} {c:?} {relative}");
    let light = Light { colour: c, kind: LightKind::Ambient { relative: relative != 0 }, on: true, shadows: false, smooth: false };
    sgl().scene.create_light(generate_name != 0, light)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_ambient_light(name: i32, colour: *const f32, relative: i32) {
    shim::call!("sgl_set_ambient_light", name, colour, relative);
    let c = unsafe { v3(colour) };
    trace!("sgl_set_ambient_light", "{name} {c:?} {relative}");
    if let Some(l) = sgl().scene.light_mut(name) {
        l.colour = c;
        l.kind = LightKind::Ambient { relative: relative != 0 };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_create_point_light(
    generate_name: i32, colour: *const f32, direction: *const f32, position: *const f32, concentration: i32, casts_shadows: i32,
    smooth_highlights: i32,
) -> i32 {
    shim::call!("sgl_create_point_light", generate_name, colour, direction, position, concentration, casts_shadows, smooth_highlights);
    let (c, d, p) = unsafe { (v3(colour), v3(direction), v3(position)) };
    trace!("sgl_create_point_light", "{generate_name} {c:?} {d:?} {p:?} {concentration} {casts_shadows} {smooth_highlights}");
    let light = Light {
        colour: c,
        kind: LightKind::Point { direction: d, position: p, concentration },
        on: true,
        shadows: casts_shadows != 0,
        smooth: smooth_highlights != 0,
    };
    sgl().scene.create_light(generate_name != 0, light)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_point_light(
    name: i32, colour: *const f32, direction: *const f32, position: *const f32, concentration: i32, casts_shadows: i32,
    smooth_highlights: i32,
) {
    shim::call!("sgl_set_point_light", name, colour, direction, position, concentration, casts_shadows, smooth_highlights);
    let (c, d, p) = unsafe { (v3(colour), v3(direction), v3(position)) };
    trace!("sgl_set_point_light", "{name} {c:?} {d:?} {p:?} {concentration} {casts_shadows} {smooth_highlights}");
    if let Some(l) = sgl().scene.light_mut(name) {
        l.colour = c;
        l.kind = LightKind::Point { direction: d, position: p, concentration };
        l.shadows = casts_shadows != 0;
        l.smooth = smooth_highlights != 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_switch_light(name: i32, on: i32, casts_shadows: i32, smooth_highlights: i32) {
    shim::call!("sgl_switch_light", name, on, casts_shadows, smooth_highlights);
    trace!("sgl_switch_light", "{name} {on} {casts_shadows} {smooth_highlights}");
    if let Some(l) = sgl().scene.light_mut(name) {
        l.on = on != 0;
        l.shadows = casts_shadows != 0;
        l.smooth = smooth_highlights != 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sgl_create_camera(zoom_factor: f32, foreground: f32, inv_background: f32) -> i32 {
    shim::call!("sgl_create_camera", zoom_factor, foreground, inv_background);
    let n = sgl().scene.create_camera(zoom_factor, foreground, inv_background);
    trace!("sgl_create_camera", "{zoom_factor} {foreground} {inv_background} -> {n}");
    n
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_fog(camera_name: i32, fog_colour: *const f32, fog_density: f32) {
    shim::call!("sgl_set_fog", camera_name, fog_colour, fog_density);
    let c = unsafe { v3(fog_colour) };
    trace!("sgl_set_fog", "{camera_name} {c:?} {fog_density}");
    if let Some(cam) = sgl().scene.camera_mut(camera_name) {
        cam.fog = Some((c, fog_density));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sgl_set_background_colour(camera_name: i32, colour: *const f32) {
    shim::call!("sgl_set_background_colour", camera_name, colour);
    let c = unsafe { v3(colour) };
    trace!("sgl_set_background_colour", "{camera_name} {c:?}");
    if let Some(cam) = sgl().scene.camera_mut(camera_name) {
        cam.background = c;
    }
}
