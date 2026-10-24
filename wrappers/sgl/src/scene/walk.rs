//! The traversal `sgl_render` makes: from the root list, in order, carrying the transform, the
//! material, the lights and the fog switch; a list made with `preserve_state` gives them back
//! when it ends, and an ignored list is skipped. Meshes come out as triangles in the camera's
//! space, lit per vertex, grouped by texture and translucent pass for the render thread.
//!
//! The lighting follows SGL's terms (*inferred* from sgl.h, the calls' names and parameters):
//! ambient material × the ambient lights, diffuse material × each point light × the cosine at
//! the vertex (a spot light: × the cosine to its axis to the power `concentration`), specular
//! with its shininess, and glow. `sgl_set_texture_effect` picks which terms the texture
//! colours (see `light` for the rest).

use super::Colour;
use super::convex::HalfSpace;
use super::graph::{Camera, LightKind, MatOp, Node, Obj, Scene};
use super::math::{Mat, V3, cross, dot, normalize, sub};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub ambient: Colour,
    pub diffuse: Colour,
    pub specular: Colour,
    pub shininess: i32,
    pub glow: Colour,
    pub opacity: f32,
    pub texture: i32,
    pub flip: (bool, bool),
    pub effect: [bool; 4],
}

impl Default for Material {
    /// Deviation: SGL's default material is not documented in sgl.h; white ambient and diffuse,
    /// no specular or glow, opaque and untextured is assumed. POD sets every field it uses.
    fn default() -> Self {
        Material {
            ambient: [1.0; 3],
            diffuse: [1.0; 3],
            specular: [0.0; 3],
            shininess: 0,
            glow: [0.0; 3],
            opacity: 1.0,
            texture: 0,
            flip: (false, false),
            effect: [true, true, false, false],
        }
    }
}

impl Material {
    pub fn apply(&mut self, op: MatOp) {
        match op {
            MatOp::Ambient(c) => self.ambient = c,
            MatOp::Diffuse(c) => self.diffuse = c,
            MatOp::Specular(c, s) => (self.specular, self.shininess) = (c, s),
            MatOp::Glow(c) => self.glow = c,
            MatOp::Opacity(o) => self.opacity = o,
            MatOp::Texture(t, u, v) => (self.texture, self.flip) = (t, (u, v)),
            MatOp::Effect(e) => self.effect = e,
        }
    }
}

/// A vertex for the render thread: its place in the camera's space, its texture coordinates,
/// the colour the texture multiplies (alpha: the opacity) and the colour added after.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vert {
    pub p: V3,
    pub uv: [f32; 2],
    pub modulate: [f32; 4],
    pub add: Colour,
}

/// Triangles that share a texture, a pass and the fog switch.
#[derive(Clone, Debug, Default)]
pub struct Batch {
    pub texture: u32,
    /// 0 for opaque; translucent batches are drawn after, in the order of their passes.
    pub pass: u32,
    pub translucent: bool,
    pub fog: bool,
    pub verts: Vec<Vert>,
}

/// A light volume in the camera's space: its planes as `(n, d)` with `n · p <= d` inside, and
/// the light it adds where a surface lies inside all of them.
#[derive(Clone, Debug, Default)]
pub struct Volume {
    pub planes: Vec<[f32; 4]>,
    pub colour: Colour,
}

/// One camera's view of the scene.
#[derive(Clone, Debug)]
pub struct View {
    pub camera: Camera,
    pub batches: Vec<Batch>,
    pub volumes: Vec<Volume>,
}

#[derive(Clone, Copy)]
struct PlacedLight {
    colour: Colour,
    kind: LightKind,
    /// World position and axis of a point light.
    pos: V3,
    dir: V3,
}

#[derive(Clone)]
struct State {
    m: Mat,
    mat: Material,
    lights: Vec<PlacedLight>,
    fog: bool,
}

struct Walk<'a> {
    scene: &'a Scene,
    camera: u32,
    found: Option<(Camera, Mat)>,
    pass: u32,
    /// World-space triangles, keyed by batch.
    batches: Vec<Batch>,
    index: HashMap<(u32, u32, bool, bool), usize>,
    /// Light volumes: their planes, their placement and their light.
    volumes: Vec<(Vec<HalfSpace>, Mat, Colour)>,
}

/// The scene as `camera` sees it; `None` if the camera is not in the graph (or is in an
/// ignored list).
pub fn view(scene: &Scene, camera: u32) -> Option<View> {
    let mut w = Walk { scene, camera, found: None, pass: 0, batches: Vec::new(), index: HashMap::new(), volumes: Vec::new() };
    let mut st = State { m: Mat::IDENTITY, mat: Material::default(), lights: Vec::new(), fog: true };
    w.list(scene.root, &mut st, 0);
    let (cam, place) = w.found?;
    let to_cam = place.inverse();
    let mut batches = w.batches;
    for b in &mut batches {
        for v in &mut b.verts {
            v.p = to_cam.point(v.p);
        }
    }
    batches.retain(|b| !b.verts.is_empty());
    let mut volumes = Vec::new();
    for (planes, m, colour) in w.volumes {
        // Each plane through its point nearest the origin, carried into the camera's space.
        let c = to_cam.mul(&m);
        let nm = c.normal_matrix();
        let planes = planes
            .iter()
            .map(|h| {
                let p = c.point([h.n[0] * h.d, h.n[1] * h.d, h.n[2] * h.d]);
                let n = normalize(nm.vector(h.n));
                [n[0], n[1], n[2], dot(n, p)]
            })
            .collect();
        volumes.push(Volume { planes, colour });
    }
    Some(View { camera: cam, batches, volumes })
}

impl Walk<'_> {
    fn list(&mut self, id: u32, st: &mut State, depth: u32) {
        let Some(l) = self.scene.list(id) else { return };
        if depth > 64 {
            return; // a cycle; SGL refuses them (sgl_err_cyclic_reference)
        }
        for n in &l.nodes {
            match *n {
                Node::List(c) => {
                    let Some(child) = self.scene.list(c) else { continue };
                    if child.ignore {
                        continue;
                    }
                    if child.preserve {
                        let mut s = st.clone();
                        self.list(c, &mut s, depth + 1);
                    } else {
                        self.list(c, st, depth + 1);
                    }
                }
                Node::Transform(t) => {
                    if let Some(Obj::Transform(m)) = self.scene.objs.get(&t) {
                        st.m = st.m.mul(m);
                    }
                }
                Node::Material(op) => st.mat.apply(op),
                Node::QualFog(on) => st.fog = on,
                Node::Translucent => self.pass += 1,
                Node::Light(i) => {
                    if let Some(Obj::Light(l)) = self.scene.objs.get(&i) {
                        if l.on {
                            let (pos, dir) = match l.kind {
                                LightKind::Point { direction, position, .. } => (st.m.point(position), normalize(st.m.vector(direction))),
                                LightKind::Ambient { .. } => ([0.0; 3], [0.0; 3]),
                            };
                            st.lights.push(PlacedLight { colour: l.colour, kind: l.kind, pos, dir });
                        }
                    }
                }
                Node::Camera(c) => {
                    if c == self.camera && self.found.is_none() {
                        if let Some(Obj::Camera(cam)) = self.scene.objs.get(&c) {
                            self.found = Some((*cam, st.m));
                        }
                    }
                }
                Node::Mesh(i) => self.mesh(i, st),
                Node::Convex(i) => self.convex(i, st),
            }
        }
    }

    fn batch(&mut self, mat: &Material, fog: bool) -> usize {
        let tex = mat.texture.max(0) as u32;
        // Deviation: a translucent map's clear pixels are cut out (POD's are either clear or
        // opaque, `FUN_004997d0`); only the material's opacity makes a face see-through.
        let translucent = mat.opacity < 1.0;
        let pass = if translucent { self.pass + 1 } else { 0 };
        let key = (tex, pass, translucent, fog);
        *self.index.entry(key).or_insert_with(|| {
            self.batches.push(Batch { texture: tex, pass, translucent, fog, verts: Vec::new() });
            self.batches.len() - 1
        })
    }

    fn mesh(&mut self, id: u32, st: &State) {
        let Some(Obj::Mesh(mesh)) = self.scene.objs.get(&id) else { return };
        let world: Vec<V3> = mesh.verts.iter().map(|v| st.m.point(v.p)).collect();
        let nm = st.m.normal_matrix();
        let mut mat = st.mat;
        let mut ops = mesh.ops.iter().peekable();
        let ambient = st.lights.iter().filter(|l| matches!(l.kind, LightKind::Ambient { .. })).fold([0.0f32; 3], |a, l| [a[0] + l.colour[0], a[1] + l.colour[1], a[2] + l.colour[2]]);
        for (fi, &(n, idx)) in mesh.faces.iter().enumerate() {
            while let Some(&&(at, op)) = ops.peek() {
                if at > fi {
                    break;
                }
                mat.apply(op);
                ops.next();
            }
            let b = self.batch(&mat, st.fog);
            let ids = &idx[..n as usize];
            let face_n = normalize(cross(sub(world[ids[1] as usize], world[ids[0] as usize]), sub(world[ids[2] as usize], world[ids[0] as usize])));
            let textured = mat.texture > 0;
            let vert = |i: u32| {
                let v = &mesh.verts[i as usize];
                let p = world[i as usize];
                let normal = v.n.map_or(face_n, |n| normalize(nm.vector(n)));
                let mut uv = v.uv.unwrap_or([0.0; 2]);
                if mat.flip.0 {
                    uv[0] = 1.0 - uv[0];
                }
                if mat.flip.1 {
                    uv[1] = 1.0 - uv[1];
                }
                let (modulate, add) = light(&mat, textured, ambient, &st.lights, p, normal);
                Vert { p, uv, modulate: [modulate[0], modulate[1], modulate[2], mat.opacity], add }
            };
            let tri = |a, b, c| [vert(a), vert(b), vert(c)];
            let out = &mut self.batches[b].verts;
            out.extend(tri(ids[0], ids[1], ids[2]));
            if n == 4 {
                out.extend(tri(ids[0], ids[2], ids[3]));
            }
        }
    }
}

impl Walk<'_> {
    /// A light volume: its planes, placed by the current transform. Deviation: not ported: a
    /// convex object that is not a light volume (a shadow volume, or a solid drawn from its
    /// planes) is skipped; POD makes only the one light volume (`FUN_0049b770`).
    fn convex(&mut self, id: u32, st: &State) {
        let Some(Obj::Convex(c)) = self.scene.objs.get(&id) else { return };
        if !c.light_volume || c.planes.len() < 4 {
            return;
        }
        let planes: Vec<HalfSpace> = c.planes.iter().map(HalfSpace::from_plane).collect();
        // The light the volume adds: its light's colour times the volume's opacity (*inferred*;
        // sgl.h says only that a light volume's colour is added where it meets a surface).
        let light = match self.scene.objs.get(&(c.light as u32)) {
            Some(Obj::Light(l)) => l.colour,
            _ => [1.0; 3],
        };
        let mut mat = st.mat;
        for (_, op) in &c.ops {
            mat.apply(*op);
        }
        let k = mat.opacity.clamp(0.0, 1.0);
        self.volumes.push((planes, st.m, light.map(|v| v * k)));
    }
}

/// The colour at a vertex: what the texture multiplies and what is added after it.
fn light(mat: &Material, textured: bool, ambient: Colour, lights: &[PlacedLight], p: V3, n: V3) -> (Colour, Colour) {
    let mut diffuse = [0.0f32; 3];
    let mut specular = [0.0f32; 3];
    for l in lights {
        let LightKind::Point { concentration, .. } = l.kind else { continue };
        let to = normalize(sub(l.pos, p));
        // Deviation: faces are lit from both sides (SGL meshes are not culled by default, and
        // which side a face's normal points to is the game's choice).
        let c = dot(n, to).abs();
        let spot = if concentration > 0 { dot(l.dir, [-to[0], -to[1], -to[2]]).max(0.0).powi(concentration) } else { 1.0 };
        for k in 0..3 {
            diffuse[k] += l.colour[k] * c * spot;
        }
        if mat.shininess > 0 && mat.specular != [0.0; 3] {
            let s = c.powi(mat.shininess.min(128)) * spot;
            for k in 0..3 {
                specular[k] += l.colour[k] * s;
            }
        }
    }
    let terms = [
        [0, 1, 2].map(|k| mat.ambient[k] * ambient[k]),
        [0, 1, 2].map(|k| mat.diffuse[k] * diffuse[k]),
        [0, 1, 2].map(|k| mat.specular[k] * specular[k]),
        mat.glow,
    ];
    let mut modulate = [0.0f32; 3];
    let mut add = [0.0f32; 3];
    if !textured {
        for t in &terms {
            for k in 0..3 {
                modulate[k] += t[k];
            }
        }
    } else if mat.effect == [false; 4] {
        // A texture that no lighting term affects shows as it is: POD's sky sets glow 1, all
        // four effects off and fog off (FUN_0049a560), and is meant to show its picture
        // (*inferred*; sgl.h does not say).
        modulate = [1.0; 3];
    } else {
        // The terms the texture affects are multiplied by it. Of the others only the
        // specular highlight is added on top (*inferred*: the PCX chips shaded a texture by
        // one intensity and added a highlight).
        for (i, (t, on)) in terms.iter().zip(mat.effect).enumerate() {
            let into = if on {
                &mut modulate
            } else if i == 2 {
                &mut add
            } else {
                continue;
            };
            for k in 0..3 {
                into[k] += t[k];
            }
        }
    }
    (modulate.map(|v| v.min(1.0)), add.map(|v| v.min(1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::graph::{Light, Vertex};

    fn quad(s: &mut Scene) {
        s.create_mesh(false);
        let v = |x, y| Vertex { p: [x, y, 10.0], n: Some([0.0, 0.0, -1.0]), uv: Some([x, y]) };
        s.add_vertices([v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)].into_iter());
        s.add_face(&[0, 1, 2, 3]);
    }

    #[test]
    fn camera_space_and_ambient() {
        let mut s = Scene::new();
        s.create_list(false, true);
        s.transform(Mat::translate([0.0, 0.0, 5.0]));
        let cam = s.create_camera(1.0, 1.0, 0.0);
        s.to_parent();
        s.create_light(false, Light { colour: [0.5; 3], kind: LightKind::Ambient { relative: false }, on: true, shadows: false, smooth: false });
        quad(&mut s);
        let v = view(&s, cam as u32).unwrap();
        assert_eq!(v.batches.len(), 1);
        let b = &v.batches[0];
        assert_eq!(b.verts.len(), 6);
        assert_eq!(b.verts[0].p, [0.0, 0.0, 5.0], "the camera 5 forward sees the quad 5 ahead");
        assert_eq!(b.verts[0].modulate, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn ignored_and_preserved_lists() {
        let mut s = Scene::new();
        let cam = s.create_camera(1.0, 1.0, 0.0);
        let l = s.create_list(true, true);
        s.material(MatOp::Opacity(0.5));
        quad(&mut s);
        s.to_parent();
        quad(&mut s);
        let v = view(&s, cam as u32).unwrap();
        assert_eq!(v.batches.iter().filter(|b| b.translucent).count(), 1);
        assert_eq!(v.batches.iter().filter(|b| !b.translucent).count(), 1, "the material ends with its list");
        s.set_ignore_list(l, true);
        assert_eq!(view(&s, cam as u32).unwrap().batches.len(), 1);
    }
}
