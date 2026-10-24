//! `sgltri_shadow`: one convex shadow or light volume from up to 100 faces already on the
//! screen. PowerVR's driver turns each face into one plane and the chip tests every pixel
//! against all of them (SGLMID4.dll of driver 3.1.0, `sgltri_shadow` at `10003080`, the
//! shared builder `FUN_100189d0`; *observed*). This module builds the same planes in the
//! camera space the render thread uses, as `walk::Volume`. Plain Rust, tested on the host.
//!
//! The rule, from the driver (*observed*) and what it means (*inferred*, `sgl2-planes.md`):
//!
//! - A face whose screen area `(x1 - x0)(y2 - y0) - (y1 - y0)(x2 - x0)` is at most 0 goes to
//!   the driver's list A: the near side, a face whose front the eye sees. A pixel is inside
//!   when it lies behind it.
//! - A face whose area is above 0 goes to list B: the far side. A pixel is inside when it lies
//!   in front of it.
//! - A face whose area is below 1e-20 either way is edge-on (`100c92f0`, `100c931c`; that the
//!   test takes the area's size is *inferred*, as list A gets faces too). It
//!   becomes a cut across the screen: `E = -N_xy · (p - v1)`, with `N = (v1 - v0) × (v2 - v0)`
//!   in screen `(x, y, z)` (`fZ` at +8). Inside is `E >= 0`. So is a face too steep for the
//!   chip's depth plane.
//! - A pixel is inside the volume when it is inside every plane: the volume is the
//!   intersection of the half-spaces, not a mesh. So callers wind each face so that `N`
//!   points out of the volume.
//!
//! Deviation: the planes are computed in `f64` and handed to OpenGL as `f32`; the driver packs
//! them into the chip's own float format and a fixed-point depth (`FUN_10035aa0`).

use super::tri::{NEAR_Z, Vertex, rgba};
use super::walk::VolumeKind;
use super::{Colour, ERR_BAD_PARAMETER};

/// The most faces one `sgltri_shadow` call may give: one convex volume of at most 100 planes.
pub const MAX_SHADOW_FACES: i32 = 100;

/// `sgltri_shadow`'s checks, in the driver's order (`10003080`; *observed*): no faces is a
/// success; more than 100 is `sgl_err_bad_parameter`; with no shadows or light volumes
/// (`eShadowLightVolMode` 0) it is a success; then a null pointer or a negative count is
/// `sgl_err_bad_parameter`. `mode`: the context's mode, `None` for a null context, which the
/// driver reads without checking (and crashes on). Returns the error to report and stop with,
/// or `None` to go on with the volume.
pub fn shadow_args(n: i32, mode: Option<u32>, pointers: bool) -> Option<i32> {
    if n == 0 {
        return Some(0);
    }
    if n > MAX_SHADOW_FACES {
        return Some(ERR_BAD_PARAMETER);
    }
    if mode == Some(0) {
        return Some(0);
    }
    if mode.is_none() || !pointers || n < 0 {
        return Some(ERR_BAD_PARAMETER);
    }
    None
}

/// The kind of volume a mode makes: mode 1 shadows, any other mode light volumes (the builder
/// `FUN_100189d0` is called with `mode == 1`; *observed*).
pub fn kind(mode: u32) -> VolumeKind {
    if mode == 1 { VolumeKind::Shadow } else { VolumeKind::Light }
}

/// What the context's +0x38 means for a kind of volume. Shadows: `fShadowBrightness`, 0 black
/// to 1 no shadow (sgl.h), as a grey that multiplies the picture. Light volumes:
/// `u32LightVolColour`, added to the picture. Deviation: it is read as a `D3DCOLOR`
/// (0x00RRGGBB). The driver's light packer puts red in the low bits, the shadow packer blue
/// (`1003c4c0`, `1003c6c0`; *measured*); what the chip makes of that is not known.
pub fn volume_colour(kind: VolumeKind, value: u32) -> Colour {
    match kind {
        VolumeKind::Shadow => {
            let b = f32::from_bits(value);
            let b = if b.is_nan() { 1.0 } else { b.clamp(0.0, 1.0) };
            [b; 3]
        }
        VolumeKind::Light => {
            let c = rgba(value);
            [c[0], c[1], c[2]]
        }
    }
}

/// Below this screen area a face is edge-on (`100c92f0` / `100c931c`; *observed*).
const EDGE_ON: f64 = 1e-20;

/// The screen corner the driver tests a depth plane at: 1024 pixels (*observed*, `10003080`).
const STEEP_AT: f64 = 1024.0;

type P3 = [f64; 3];

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: P3, b: P3) -> P3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The plane `[n, d]` with `n · p <= d` inside, scaled to a unit normal. `None` for a plane
/// with no normal.
fn unit(n: P3, d: f64) -> Option<[f32; 4]> {
    let len = dot(n, n).sqrt();
    (len > 0.0 && len.is_finite()).then(|| [(n[0] / len) as f32, (n[1] / len) as f32, (n[2] / len) as f32, (d / len) as f32])
}

/// An edge plane: a cut across the screen through `v1`, inside where `E >= 0`. A screen point
/// `(x, y)` at distance `z` is the camera-space point `(x z, -y z, z)`, so the cut is the plane
/// through the eye with `n = (Nx, -Ny, -(Nx x1 + Ny y1))` and `d = 0`.
fn edge(v: &[Vertex; 3]) -> Option<[f32; 4]> {
    let s = |v: &Vertex| [v.x as f64, v.y as f64, v.z as f64];
    let n = cross(sub(s(&v[1]), s(&v[0])), sub(s(&v[2]), s(&v[0])));
    let (x1, y1) = (v[1].x as f64, v[1].y as f64);
    unit([n[0], -n[1], -(n[0] * x1 + n[1] * y1)], 0.0)
}

/// True when the face's depth plane is too steep for the chip, so the driver makes it an edge
/// plane: `(|A| + |B|) 1024 + |(A + B) 1024 + C| > 1`, for depth `A x + B y + C`, depth being
/// `NEAR_Z / w` (*observed*, `10003080`).
fn steep(v: &[Vertex; 3], area: f64) -> bool {
    let (x, y) = (|i: usize| v[i].x as f64, |i: usize| v[i].y as f64);
    let d = |i: usize| NEAR_Z as f64 * v[i].inv_w as f64;
    let a = ((d(1) - d(0)) * (y(2) - y(0)) - (d(2) - d(0)) * (y(1) - y(0))) / area;
    let b = ((x(1) - x(0)) * (d(2) - d(0)) - (x(2) - x(0)) * (d(1) - d(0))) / area;
    let c = d(0) - a * x(0) - b * y(0);
    (a.abs() + b.abs()) * STEEP_AT + ((a + b) * STEEP_AT + c).abs() > 1.0
}

/// One face's plane in camera space, oriented so that `n · p <= d` is inside the volume.
/// `None` for a face the plane test cannot use (a vertex behind the eye or not a number).
pub fn face_plane(v: &[Vertex; 3]) -> Option<[f32; 4]> {
    if v.iter().any(|v| !(v.inv_w > 0.0 && v.inv_w.is_finite() && v.x.is_finite() && v.y.is_finite() && v.z.is_finite())) {
        return None;
    }
    let (x, y) = (|i: usize| v[i].x as f64, |i: usize| v[i].y as f64);
    let area = (x(1) - x(0)) * (y(2) - y(0)) - (y(1) - y(0)) * (x(2) - x(0));
    if area.abs() < EDGE_ON || steep(v, area) {
        return edge(v);
    }
    // The camera-space points, as `tri` places vertices.
    let p = |v: &Vertex| {
        let w = 1.0 / v.inv_w as f64;
        [v.x as f64 * w, -(v.y as f64) * w, w]
    };
    let (p0, p1, p2) = (p(&v[0]), p(&v[1]), p(&v[2]));
    let mut n = cross(sub(p1, p0), sub(p2, p0));
    let mut d = dot(n, p0);
    // A near face (list A) leaves the eye outside: 0 > d. A far face (list B) holds it: 0 <= d.
    let near = area <= 0.0;
    if (near && d > 0.0) || (!near && d < 0.0) {
        n = n.map(|c| -c);
        d = -d;
    }
    unit(n, d)
}

/// The planes of one volume, in the faces' order. A face naming a vertex past `verts`, or one
/// `face_plane` cannot use, gives no plane. Deviation: the driver reads past the array there,
/// and packs a plane for any face.
pub fn volume(faces: &[[u32; 3]], verts: &[Vertex]) -> Vec<[f32; 4]> {
    faces
        .iter()
        .filter_map(|f| {
            let v = [verts.get(f[0] as usize)?, verts.get(f[1] as usize)?, verts.get(f[2] as usize)?];
            face_plane(&[*v[0], *v[1], *v[2]])
        })
        .collect()
}

#[cfg(test)]
mod tests;
