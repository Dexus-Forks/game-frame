//! A software SGL 3.1 context: the calls by their names in VideoLogic's `sgl.h`, editing the
//! scene graph (`scene`) and rendering it with the PCX2 model (`pcx2`) into a 565 buffer at
//! any multiple of the device's size. Arguments are Rust values where `sgl.h` takes pointers
//! (`sgl_vector`, `sgl_colour`: three floats). Names are SGL's: below zero is an error.
//!
//! A game's own SGL calls map one to one: `sgl_create_list(1, 1, 0)` is
//! `sgl.sgl_create_list(true, true, false)`. Only the calls a scene of meshes needs are here
//! (POD Gold's PowerVR build imports 46; the rest are listed in the crate's README).

use crate::pcx2::{self, Frame, Viewport};
use crate::scene::graph::{Light, LightKind, MatOp, Texture, Vertex};
use crate::scene::math::{Mat, V3};
use crate::scene::{walk, Colour, Scene, ERR_BAD_NAME};

/// `sgl_map_types` (sgl.h).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapType {
    Map16Bit = 0,
    Map16BitMipmap = 1,
    Map8Bit = 2,
    Trans16 = 3,
    Trans16Mipmap = 4,
}

pub struct Sgl {
    pub scene: Scene,
    pub frame: Frame,
    /// The device's size, as `sgl_create_screen_device` was given it.
    pub device: (usize, usize),
    pub scale: f32,
    viewports: Vec<Viewport>,
    /// Convex objects the last renders could not draw (`walk::View::skipped`).
    pub skipped: u32,
}

impl Sgl {
    /// `sgl_create_screen_device(0, width, height, sgl_device_16bit, double buffer)`, drawn at
    /// `scale` times its size.
    pub fn create_screen_device(width: usize, height: usize, scale: f32) -> Sgl {
        let (w, h) = (((width as f32) * scale).round() as usize, ((height as f32) * scale).round() as usize);
        Sgl { scene: Scene::new(), frame: Frame::new(w.max(1), h.max(1)), device: (width, height), scale, viewports: Vec::new(), skipped: 0 }
    }

    /// The device at another scale, the scene kept.
    pub fn rescale(&mut self, scale: f32) {
        let (w, h) = (((self.device.0 as f32) * scale).round() as usize, ((self.device.1 as f32) * scale).round() as usize);
        self.frame = Frame::new(w.max(1), h.max(1));
        self.scale = scale;
    }

    /// `sgl_create_viewport(device, left, top, right, bottom, cam_left, cam_top, cam_right,
    /// cam_bottom)`: device pixels (inclusive corners) and the camera rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn sgl_create_viewport(&mut self, left: i32, top: i32, right: i32, bottom: i32, cl: f32, ct: f32, cr: f32, cb: f32) -> i32 {
        self.viewports.push(Viewport { rect: [left, top, right, bottom], cam: [cl, ct, cr, cb] });
        self.viewports.len() as i32
    }

    // Lists.

    pub fn sgl_create_list(&mut self, generate_name: bool, preserve_state: bool, _separate_list: bool) -> i32 {
        self.scene.create_list(generate_name, preserve_state)
    }

    pub fn sgl_to_parent(&mut self) {
        self.scene.to_parent();
    }

    pub fn sgl_modify_list(&mut self, name: i32, clear: bool) -> i32 {
        self.scene.modify_list(name, clear)
    }

    pub fn sgl_delete_list(&mut self, name: i32) -> i32 {
        self.scene.delete_list(name)
    }

    pub fn sgl_set_ignore_list(&mut self, name: i32, ignore: bool) -> i32 {
        self.scene.set_ignore_list(name, ignore)
    }

    // Transforms.

    pub fn sgl_create_transform(&mut self, generate_name: bool) -> i32 {
        self.scene.create_transform(generate_name)
    }

    pub fn sgl_modify_transform(&mut self, name: i32, clear: bool) -> i32 {
        self.scene.modify_transform(name, clear)
    }

    pub fn sgl_translate(&mut self, x: f32, y: f32, z: f32) {
        self.scene.transform(Mat::translate([x, y, z]));
    }

    /// `angle` in radians.
    pub fn sgl_rotate(&mut self, axis: V3, angle: f32) {
        self.scene.transform(Mat::rotate(axis, angle));
    }

    pub fn sgl_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scene.transform(Mat::scale([x, y, z]));
    }

    /// A whole matrix at once: not an SGL call; a port that already has the transform (a
    /// camera's axes) multiplies it in as the calls would.
    pub fn transform(&mut self, m: Mat) {
        self.scene.transform(m);
    }

    // Meshes.

    pub fn sgl_create_mesh(&mut self, generate_name: bool) -> i32 {
        self.scene.create_mesh(generate_name)
    }

    /// `sgl_add_vertices(n, vertices, normals, uvs)`; `normals` and `uvs` may be absent, as
    /// the null pointers SGL allows.
    pub fn sgl_add_vertices(&mut self, vertices: &[V3], normals: Option<&[V3]>, uvs: Option<&[[f32; 2]]>) -> i32 {
        let verts = vertices.iter().enumerate().map(|(i, p)| Vertex { p: *p, n: normals.map(|n| n[i]), uv: uvs.map(|u| u[i]) });
        self.scene.add_vertices(verts)
    }

    pub fn sgl_add_face(&mut self, ids: &[i32]) -> i32 {
        self.scene.add_face(ids)
    }

    // Materials.

    pub fn sgl_set_ambient(&mut self, c: Colour) {
        self.scene.material(MatOp::Ambient(c));
    }

    pub fn sgl_set_diffuse(&mut self, c: Colour) {
        self.scene.material(MatOp::Diffuse(c));
    }

    pub fn sgl_set_specular(&mut self, c: Colour, shininess: i32) {
        self.scene.material(MatOp::Specular(c, shininess));
    }

    pub fn sgl_set_glow(&mut self, c: Colour) {
        self.scene.material(MatOp::Glow(c));
    }

    pub fn sgl_set_opacity(&mut self, opacity: f32) {
        self.scene.material(MatOp::Opacity(opacity));
    }

    pub fn sgl_set_texture_map(&mut self, name: i32, flip_u: bool, flip_v: bool) {
        self.scene.material(MatOp::Texture(name, flip_u, flip_v));
    }

    pub fn sgl_set_texture_effect(&mut self, ambient: bool, diffuse: bool, specular: bool, glow: bool) {
        self.scene.material(MatOp::Effect([ambient, diffuse, specular, glow]));
    }

    pub fn sgl_new_translucent(&mut self) {
        self.scene.new_translucent();
    }

    pub fn sgl_qual_fog(&mut self, on: bool) {
        self.scene.qual_fog(on);
    }

    // Textures.

    /// `sgl_create_texture(map_type, size, mipmap, dither, map, 0)` from SGL's intermediate
    /// map: `w × h` RGBA bytes with alpha 0 for opaque and 255 for clear (sgl.h). On a map type
    /// that is not translucent alpha is ignored.
    /// Deviation: the texels keep 8 bits a channel; the chip's own 16-bit texel layouts (the
    /// driver's `sgl_preprocess_texture`) are not taken from SGLMID4.dll.
    pub fn sgl_create_texture(&mut self, map_type: MapType, w: u32, h: u32, intermediate: &[u8]) -> i32 {
        let translucent = matches!(map_type, MapType::Trans16 | MapType::Trans16Mipmap);
        let mut rgba = intermediate.to_vec();
        for p in rgba.chunks_exact_mut(4) {
            p[3] = if translucent { 255 - p[3] } else { 255 };
        }
        self.scene.create_texture(Texture { w, h, rgba, translucent })
    }

    pub fn sgl_delete_texture(&mut self, name: i32) {
        self.scene.delete_texture(name);
    }

    // Cameras.

    pub fn sgl_create_camera(&mut self, zoom: f32, foreground: f32, inv_background: f32) -> i32 {
        self.scene.create_camera(zoom, foreground, inv_background)
    }

    pub fn sgl_set_fog(&mut self, camera: i32, colour: Colour, density: f32) {
        if let Some(c) = self.scene.camera_mut(camera) {
            c.fog = Some((colour, density));
        }
    }

    pub fn sgl_set_background_colour(&mut self, camera: i32, colour: Colour) {
        if let Some(c) = self.scene.camera_mut(camera) {
            c.background = colour;
        }
    }

    // Lights.

    pub fn sgl_create_ambient_light(&mut self, generate_name: bool, colour: Colour, relative: bool) -> i32 {
        let light = Light { colour, kind: LightKind::Ambient { relative }, on: true, shadows: false, smooth: false };
        self.scene.create_light(generate_name, light)
    }

    pub fn sgl_set_ambient_light(&mut self, name: i32, colour: Colour, relative: bool) {
        if let Some(l) = self.scene.light_mut(name) {
            l.colour = colour;
            l.kind = LightKind::Ambient { relative };
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sgl_create_point_light(&mut self, generate_name: bool, colour: Colour, direction: V3, position: V3, concentration: i32, shadows: bool, smooth: bool) -> i32 {
        let light = Light { colour, kind: LightKind::Point { direction, position, concentration }, on: true, shadows, smooth };
        self.scene.create_light(generate_name, light)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sgl_set_point_light(&mut self, name: i32, colour: Colour, direction: V3, position: V3, concentration: i32, shadows: bool, smooth: bool) {
        if let Some(l) = self.scene.light_mut(name) {
            *l = Light { colour, kind: LightKind::Point { direction, position, concentration }, on: l.on, shadows, smooth };
        }
    }

    pub fn sgl_switch_light(&mut self, name: i32, on: bool, shadows: bool, smooth: bool) {
        if let Some(l) = self.scene.light_mut(name) {
            (l.on, l.shadows, l.smooth) = (on, shadows, smooth);
        }
    }

    // Rendering.

    /// `sgl_render(viewport, camera, swap)`: the scene seen from `camera` into the viewport's
    /// pixels. Returns `ERR_BAD_NAME` for an unknown viewport or a camera not in the graph.
    pub fn sgl_render(&mut self, viewport: i32, camera: i32) -> i32 {
        let Some(vp) = viewport.checked_sub(1).and_then(|i| self.viewports.get(i as usize)).copied() else { return ERR_BAD_NAME };
        let Some(view) = walk::view(&self.scene, camera as u32) else { return ERR_BAD_NAME };
        self.skipped += view.skipped;
        pcx2::render(&mut self.frame, &view, &vp, self.device, &self.scene.textures);
        0
    }

    /// The colour buffer as RGBA bytes.
    pub fn picture(&self) -> Vec<u8> {
        self.frame.picture()
    }
}
