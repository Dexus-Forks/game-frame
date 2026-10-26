//! The objects of SGL's scene graph and the state the calls edit: the current list and the
//! object being edited. One method per SGL call; the exported calls (`api`) only read the
//! game's pointers and call these. How POD uses each call, with its evidence, is in
//! POD's `docs/hardware/3d-cards.md` ("The PowerVR build").
//!
//! Names: every object gets one from a single counter. A call made without `generate_name`
//! returns 0 (`sgl_no_err`) and the object stays anonymous to the game, as in SGL.

use super::math::{Mat, V3};
use super::{Colour, DEFAULT_LIST, ERR_BAD_NAME};
use std::collections::HashMap;

/// An entry of a display list, in the order the game made it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Node {
    List(u32),
    Transform(u32),
    Mesh(u32),
    Convex(u32),
    Material(MatOp),
    Light(u32),
    Camera(u32),
    /// `sgl_new_translucent`: later translucent objects go in a later pass.
    Translucent,
    /// `sgl_qual_fog`: fog on or off for what follows.
    QualFog(bool),
}

/// One material change (`sgl_set_*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MatOp {
    Ambient(Colour),
    Diffuse(Colour),
    Specular(Colour, i32),
    Glow(Colour),
    Opacity(f32),
    /// A texture name (0 for none), and whether u and v are flipped.
    Texture(i32, bool, bool),
    /// Which lighting terms the texture colours: ambient, diffuse, specular, glow.
    Effect([bool; 4]),
}

#[derive(Clone, Debug, Default)]
pub struct List {
    pub nodes: Vec<Node>,
    /// `preserve_state`: the transform, material and lights go back to what they were after it.
    pub preserve: bool,
    pub ignore: bool,
    pub parent: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub p: V3,
    pub n: Option<V3>,
    pub uv: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub verts: Vec<Vertex>,
    /// Up to four vertex indices, and how many are used.
    pub faces: Vec<(u8, [u32; 4])>,
    /// Material changes made while the mesh was being edited, with the face count then: they
    /// apply to the faces added after them.
    pub ops: Vec<(usize, MatOp)>,
}

/// A plane of a convex object: a point on it, its normal, and (for `sgl_add_plane`) two more
/// points; invisible planes only bound the object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub p: V3,
    pub n: V3,
    pub invisible: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Convex {
    /// Made by `sgl_create_light_volume`: it adds light where it meets other objects.
    pub light_volume: bool,
    pub light: i32,
    pub planes: Vec<Plane>,
    pub ops: Vec<(usize, MatOp)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightKind {
    /// `relative`: scaled by the light above it in the list.
    Ambient { relative: bool },
    /// A point light, or a spot light pointing along `direction` with `concentration`.
    Point { direction: V3, position: V3, concentration: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub colour: Colour,
    pub kind: LightKind,
    pub on: bool,
    pub shadows: bool,
    pub smooth: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// 1 gives a field of view of 90 degrees across the viewport's camera rectangle.
    pub zoom: f32,
    /// The near clipping distance.
    pub foreground: f32,
    /// One over the far distance; 0 for none.
    pub inv_background: f32,
    pub fog: Option<(Colour, f32)>,
    pub background: Colour,
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub w: u32,
    pub h: u32,
    /// RGBA, alpha 255 for opaque (SGL's own pixels have 0 for opaque).
    pub rgba: Vec<u8>,
    /// A translucent map type (`sgl_map_trans16`, `sgl_map_trans16_mm`).
    pub translucent: bool,
}

#[derive(Clone, Debug)]
pub enum Obj {
    List(List),
    Transform(Mat),
    Mesh(Mesh),
    Convex(Convex),
    Light(Light),
    Camera(Camera),
}

/// What the material and plane calls go to.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Edit {
    None,
    Transform(u32),
    Mesh(u32),
    Convex(u32),
}

pub struct Scene {
    pub objs: HashMap<u32, Obj>,
    pub textures: HashMap<u32, Texture>,
    next: u32,
    pub root: u32,
    current: u32,
    edit: Edit,
    /// The last convex object made, for planes added after another object became current.
    last_convex: Option<u32>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Scene {
        let mut objs = HashMap::new();
        objs.insert(1, Obj::List(List { preserve: true, parent: 1, ..Default::default() }));
        Scene { objs, textures: HashMap::new(), next: 2, root: 1, current: 1, edit: Edit::None, last_convex: None }
    }

    fn name(&mut self) -> u32 {
        let n = self.next;
        self.next += 1;
        n
    }

    fn list_id(&self, name: i32) -> Option<u32> {
        if name == DEFAULT_LIST {
            return Some(self.root);
        }
        matches!(self.objs.get(&(name as u32)), Some(Obj::List(_))).then_some(name as u32)
    }

    pub fn list(&self, id: u32) -> Option<&List> {
        match self.objs.get(&id) {
            Some(Obj::List(l)) => Some(l),
            _ => None,
        }
    }

    fn list_mut(&mut self, id: u32) -> Option<&mut List> {
        match self.objs.get_mut(&id) {
            Some(Obj::List(l)) => Some(l),
            _ => None,
        }
    }

    /// Add a new object to the current list.
    fn add(&mut self, obj: Obj, node: fn(u32) -> Node) -> u32 {
        let id = self.name();
        self.objs.insert(id, obj);
        let cur = self.current;
        if let Some(l) = self.list_mut(cur) {
            l.nodes.push(node(id));
        }
        id
    }

    fn push(&mut self, node: Node) {
        let cur = self.current;
        if let Some(l) = self.list_mut(cur) {
            l.nodes.push(node);
        }
    }

    fn named(id: u32, generate: bool) -> i32 {
        if generate { id as i32 } else { 0 }
    }

    // Lists.

    pub fn create_list(&mut self, generate: bool, preserve: bool) -> i32 {
        let parent = self.current;
        let id = self.add(Obj::List(List { preserve, parent, ..Default::default() }), Node::List);
        self.current = id;
        self.edit = Edit::None;
        Self::named(id, generate)
    }

    pub fn to_parent(&mut self) {
        self.current = self.list(self.current).map_or(self.root, |l| l.parent);
        self.edit = Edit::None;
    }

    pub fn modify_list(&mut self, name: i32, clear: bool) -> i32 {
        let Some(id) = self.list_id(name) else { return ERR_BAD_NAME };
        if clear {
            let nodes = self.list_mut(id).map(|l| std::mem::take(&mut l.nodes)).unwrap_or_default();
            for n in nodes {
                self.delete_node(n);
            }
        }
        self.current = id;
        self.edit = Edit::None;
        0
    }

    pub fn delete_list(&mut self, name: i32) -> i32 {
        let Some(id) = self.list_id(name) else { return ERR_BAD_NAME };
        if id == self.root {
            return self.modify_list(name, true);
        }
        let parent = self.list(id).map_or(self.root, |l| l.parent);
        if let Some(p) = self.list_mut(parent) {
            p.nodes.retain(|n| *n != Node::List(id));
        }
        // The current list stays valid: if it was inside, its parent's parent chain is gone.
        let mut c = self.current;
        while c != self.root {
            if c == id {
                self.current = parent;
                self.edit = Edit::None;
                break;
            }
            c = self.list(c).map_or(self.root, |l| l.parent);
        }
        self.delete_node(Node::List(id));
        0
    }

    /// Remove an object and everything a list holds.
    fn delete_node(&mut self, n: Node) {
        let id = match n {
            Node::List(i) | Node::Transform(i) | Node::Mesh(i) | Node::Convex(i) | Node::Light(i) | Node::Camera(i) => i,
            _ => return,
        };
        if let Some(Obj::List(l)) = self.objs.remove(&id) {
            for c in l.nodes {
                self.delete_node(c);
            }
        }
        if matches!(self.edit, Edit::Transform(e) | Edit::Mesh(e) | Edit::Convex(e) if e == id) {
            self.edit = Edit::None;
        }
        if self.last_convex == Some(id) {
            self.last_convex = None;
        }
    }

    pub fn set_ignore_list(&mut self, name: i32, ignore: bool) -> i32 {
        let Some(id) = self.list_id(name) else { return ERR_BAD_NAME };
        if let Some(l) = self.list_mut(id) {
            l.ignore = ignore;
        }
        0
    }

    // Transforms.

    pub fn create_transform(&mut self, generate: bool) -> i32 {
        let id = self.add(Obj::Transform(Mat::IDENTITY), Node::Transform);
        self.edit = Edit::Transform(id);
        Self::named(id, generate)
    }

    pub fn modify_transform(&mut self, name: i32, clear: bool) -> i32 {
        let id = name as u32;
        let Some(Obj::Transform(m)) = self.objs.get_mut(&id) else { return ERR_BAD_NAME };
        if clear {
            *m = Mat::IDENTITY;
        }
        self.edit = Edit::Transform(id);
        0
    }

    /// Multiply the transform being edited by `t` on the right; with none being edited, an
    /// anonymous one is made in the current list.
    pub fn transform(&mut self, t: Mat) {
        let id = match self.edit {
            Edit::Transform(id) if self.objs.contains_key(&id) => id,
            _ => {
                self.create_transform(false);
                match self.edit {
                    Edit::Transform(id) => id,
                    _ => return,
                }
            }
        };
        if let Some(Obj::Transform(m)) = self.objs.get_mut(&id) {
            *m = m.mul(&t);
        }
    }

    // Materials.

    pub fn material(&mut self, op: MatOp) {
        match self.edit {
            Edit::Mesh(id) => {
                if let Some(Obj::Mesh(m)) = self.objs.get_mut(&id) {
                    m.ops.push((m.faces.len(), op));
                }
            }
            Edit::Convex(id) => {
                if let Some(Obj::Convex(c)) = self.objs.get_mut(&id) {
                    c.ops.push((c.planes.len(), op));
                }
            }
            _ => self.push(Node::Material(op)),
        }
    }

    pub fn new_translucent(&mut self) {
        self.push(Node::Translucent);
    }

    pub fn qual_fog(&mut self, on: bool) {
        self.push(Node::QualFog(on));
    }

    // Meshes.

    pub fn create_mesh(&mut self, generate: bool) -> i32 {
        let id = self.add(Obj::Mesh(Mesh::default()), Node::Mesh);
        self.edit = Edit::Mesh(id);
        Self::named(id, generate)
    }

    pub fn add_vertices(&mut self, verts: impl Iterator<Item = Vertex>) -> i32 {
        let Edit::Mesh(id) = self.edit else { return super::ERR_BAD_PARAMETER };
        let Some(Obj::Mesh(m)) = self.objs.get_mut(&id) else { return ERR_BAD_NAME };
        m.verts.extend(verts);
        0
    }

    pub fn add_face(&mut self, ids: &[i32]) -> i32 {
        let Edit::Mesh(id) = self.edit else { return super::ERR_BAD_PARAMETER };
        let Some(Obj::Mesh(m)) = self.objs.get_mut(&id) else { return ERR_BAD_NAME };
        if ids.len() < 3 || ids.len() > 4 || ids.iter().any(|&i| i < 0 || i as usize >= m.verts.len()) {
            return super::ERR_BAD_PARAMETER;
        }
        let mut f = [0u32; 4];
        for (d, s) in f.iter_mut().zip(ids) {
            *d = *s as u32;
        }
        m.faces.push((ids.len() as u8, f));
        0
    }

    // Convex objects and light volumes.

    /// The object goes into the current list now, as PowerVR's driver does (SGLMID4.dll
    /// `FUN_10025270`, *observed*); planes added later go to it even after another object was
    /// made (POD makes a transform and an empty mesh between, `FUN_0049b770`).
    pub fn create_light_volume(&mut self, generate: bool, light_volume: bool, light: i32) -> i32 {
        let id = self.add(Obj::Convex(Convex { light_volume, light, ..Default::default() }), Node::Convex);
        self.edit = Edit::Convex(id);
        self.last_convex = Some(id);
        Self::named(id, generate)
    }

    pub fn add_plane(&mut self, plane: Plane) -> i32 {
        let id = match self.edit {
            Edit::Convex(id) => id,
            _ => match self.last_convex {
                Some(id) => id,
                None => return super::ERR_BAD_PARAMETER,
            },
        };
        if let Some(Obj::Convex(c)) = self.objs.get_mut(&id) {
            c.planes.push(plane);
        }
        0
    }

    // Lights.

    pub fn create_light(&mut self, generate: bool, light: Light) -> i32 {
        let id = self.add(Obj::Light(light), Node::Light);
        self.edit = Edit::None;
        Self::named(id, generate)
    }

    pub fn light_mut(&mut self, name: i32) -> Option<&mut Light> {
        match self.objs.get_mut(&(name as u32)) {
            Some(Obj::Light(l)) => Some(l),
            _ => None,
        }
    }

    // Cameras.

    pub fn create_camera(&mut self, zoom: f32, foreground: f32, inv_background: f32) -> i32 {
        let cam = Camera { zoom, foreground, inv_background, fog: None, background: [0.0; 3] };
        let id = self.add(Obj::Camera(cam), Node::Camera);
        self.edit = Edit::None;
        id as i32
    }

    pub fn camera_mut(&mut self, name: i32) -> Option<&mut Camera> {
        match self.objs.get_mut(&(name as u32)) {
            Some(Obj::Camera(c)) => Some(c),
            _ => None,
        }
    }

    // Textures.

    pub fn create_texture(&mut self, t: Texture) -> i32 {
        let id = self.name();
        self.textures.insert(id, t);
        id as i32
    }

    pub fn delete_texture(&mut self, name: i32) -> bool {
        self.textures.remove(&(name as u32)).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_nest_and_return() {
        let mut s = Scene::new();
        let a = s.create_list(true, true);
        let b = s.create_list(true, false);
        s.to_parent();
        s.to_parent();
        assert_eq!(s.list(s.root).unwrap().nodes, vec![Node::List(a as u32)]);
        assert_eq!(s.list(a as u32).unwrap().nodes, vec![Node::List(b as u32)]);
        assert_eq!(s.create_list(false, true), 0, "anonymous lists return sgl_no_err");
    }

    #[test]
    fn mesh_materials_stay_with_their_faces() {
        let mut s = Scene::new();
        s.create_mesh(false);
        s.material(MatOp::Opacity(0.5));
        let v = |x| Vertex { p: [x, 0.0, 1.0], n: None, uv: None };
        s.add_vertices([v(0.0), v(1.0), v(2.0)].into_iter());
        assert_eq!(s.add_face(&[0, 1, 2]), 0);
        assert!(s.add_face(&[0, 1, 3]) < 0);
        let Some(Obj::Mesh(m)) = s.objs.values().find(|o| matches!(o, Obj::Mesh(_))) else { panic!() };
        assert_eq!(m.ops, vec![(0, MatOp::Opacity(0.5))]);
        assert_eq!(m.faces.len(), 1);
    }

    #[test]
    fn clearing_the_root_deletes_its_objects() {
        let mut s = Scene::new();
        s.create_list(true, true);
        let t = s.create_transform(true);
        s.to_parent();
        s.modify_list(DEFAULT_LIST, true);
        assert!(s.modify_transform(t, false) < 0);
        assert_eq!(s.objs.len(), 1);
    }

    #[test]
    fn light_volume_placed_where_it_is_made() {
        let mut s = Scene::new();
        s.create_light_volume(false, true, 0);
        s.create_transform(false);
        s.create_mesh(false);
        s.add_plane(Plane { p: [0.0; 3], n: [0.0, 0.0, 1.0], invisible: false });
        s.add_plane(Plane { p: [0.0; 3], n: [1.0, 0.0, 0.0], invisible: false });
        let root = s.list(s.root).unwrap();
        assert!(matches!(root.nodes[..], [Node::Convex(_), Node::Transform(_), Node::Mesh(_)]));
        let Some(Obj::Convex(c)) = s.objs.values().find(|o| matches!(o, Obj::Convex(_))) else { panic!() };
        assert_eq!(c.planes.len(), 2, "planes after the mesh still go to the volume");
    }

    #[test]
    fn translate_without_a_transform_makes_one() {
        let mut s = Scene::new();
        s.create_mesh(false);
        s.transform(Mat::translate([1.0, 2.0, 3.0]));
        let root = s.list(s.root).unwrap();
        assert!(matches!(root.nodes[1], Node::Transform(_)));
    }
}
