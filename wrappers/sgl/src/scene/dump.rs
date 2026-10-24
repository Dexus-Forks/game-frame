//! A text dump of the scene graph, for finding which list holds what (`SGL_DUMP`): one line
//! per entry, indented by depth, with ignored lists marked.

use super::graph::{LightKind, Node, Obj, Scene};
use std::fmt::Write;

pub fn dump(s: &Scene) -> String {
    let mut out = String::new();
    list(s, s.root, 0, &mut out);
    out
}

fn list(s: &Scene, id: u32, depth: usize, out: &mut String) {
    let Some(l) = s.list(id) else { return };
    let pad = "  ".repeat(depth);
    let _ = writeln!(out, "{pad}list {id}{}{}", if l.preserve { " kept" } else { "" }, if l.ignore { " IGNORED" } else { "" });
    if depth > 40 {
        return;
    }
    for n in &l.nodes {
        let _ = match *n {
            Node::List(c) => {
                list(s, c, depth + 1, out);
                Ok(())
            }
            Node::Transform(t) => match s.objs.get(&t) {
                Some(Obj::Transform(m)) => writeln!(out, "{pad}  transform {t} {:?}", m.m),
                _ => Ok(()),
            },
            Node::Mesh(m) => match s.objs.get(&m) {
                Some(Obj::Mesh(mesh)) => writeln!(out, "{pad}  mesh {m}: {} vertices, {} faces, first {:?}, materials {:?}", mesh.verts.len(), mesh.faces.len(), mesh.verts.first().map(|v| v.p), mesh.ops),
                _ => Ok(()),
            },
            Node::Convex(c) => match s.objs.get(&c) {
                Some(Obj::Convex(cv)) => writeln!(out, "{pad}  convex {c}: light volume {} light {} planes {} materials {:?}", cv.light_volume, cv.light, cv.planes.len(), cv.ops),
                _ => Ok(()),
            },
            Node::Material(op) => writeln!(out, "{pad}  material {op:?}"),
            Node::Light(i) => match s.objs.get(&i) {
                Some(Obj::Light(l)) => match l.kind {
                    LightKind::Ambient { relative } => writeln!(out, "{pad}  ambient light {i} {:?} relative {relative} on {}", l.colour, l.on),
                    LightKind::Point { direction, position, concentration } => {
                        writeln!(out, "{pad}  point light {i} {:?} at {position:?} towards {direction:?} concentration {concentration} on {}", l.colour, l.on)
                    }
                },
                _ => Ok(()),
            },
            Node::Camera(c) => writeln!(out, "{pad}  camera {c}"),
            Node::Translucent => writeln!(out, "{pad}  new translucent pass"),
            Node::QualFog(on) => writeln!(out, "{pad}  fog {on}"),
        };
    }
}
