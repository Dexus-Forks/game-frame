//! Convex objects as planes. SGL describes convex objects and light volumes as the space
//! inside a set of planes, each with its normal pointing out (sgl.h, `sgl_add_plane`,
//! `sgl_add_simple_plane`), and the PowerVR chip tested each pixel against the planes rather
//! than drawing faces. The render thread does the same, so no solid is built here.
//!
//! POD's one light volume (`FUN_0049b770`) is a cone of 40 planes through an apex at the
//! origin, closed by a plane across its open end.

use super::graph::Plane;
use super::math::{V3, cross, dot, normalize, sub};


/// A plane as `n · p = d`, `n` pointing out.
#[derive(Clone, Copy, Debug)]
pub struct HalfSpace {
    pub n: V3,
    pub d: f32,
}

impl HalfSpace {
    pub fn from_plane(p: &Plane) -> HalfSpace {
        let n = normalize(p.n);
        HalfSpace { n, d: dot(n, p.p) }
    }

    /// Whether a point is inside (on the plane counts).
    pub fn inside(&self, p: V3) -> bool {
        dot(self.n, p) <= self.d + 1e-6
    }
}

/// The plane through three points, its normal turned to the same side as `towards`. A plane
/// whose points are in a line is `None` (`sgl_err_colinear_plane_points`).
pub fn plane_through(a: V3, b: V3, c: V3, towards: V3) -> Option<V3> {
    let n = cross(sub(b, a), sub(c, a));
    if dot(n, n) < 1e-12 {
        return None;
    }
    let n = normalize(n);
    Some(if dot(n, towards) < 0.0 { [-n[0], -n[1], -n[2]] } else { n })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pod_cone_planes() {
        // FUN_0049b770: 40 planes through the apex and two points of the rim (radius 2 at
        // z = 10), normals out from the axis, and a plane across the end.
        let (r, len) = (2.0f32, 10.0f32);
        let step = 0.157075f32;
        let mut planes = Vec::new();
        let mut a = 0.0f32;
        for _ in 0..40 {
            let (a1, a2) = (a - step, a - 2.0 * step);
            a = a1;
            let p1 = [a1.cos() * r, a1.sin() * r, len];
            let p3 = [a2.cos() * r, a2.sin() * r, len];
            let n = plane_through(p1, [0.0; 3], p3, [a1.cos(), a1.sin(), 0.0]).unwrap();
            planes.push(HalfSpace { n, d: dot(n, p1) });
        }
        planes.push(HalfSpace { n: [0.0, 0.0, 1.0], d: len });
        let inside = |p: V3| planes.iter().all(|h| h.inside(p));
        assert!(inside([0.0, 0.0, 5.0]), "on the axis");
        assert!(inside([0.9, 0.0, 5.0]), "within the radius at half the length");
        assert!(!inside([1.2, 0.0, 5.0]), "outside it");
        assert!(!inside([0.0, 0.0, 11.0]), "past the end");
        assert!(!inside([0.0, 0.0, -1.0]), "behind the apex");
    }
}
