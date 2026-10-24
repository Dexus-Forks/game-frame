//! Drawing calls: points, lines, triangles and polygons of `GrVertex`, the antialiased forms,
//! and the `gu` calls that clip first. Every vertex goes through `prepare` and every triangle
//! through `triangle`, which do what the Voodoo's triangle setup does before drawing: the
//! origin, culling, the delta-0 colour, a TMU's own *w* (`GR_HINT_STWHINT`), and the counts
//! `grTriStats` reports.
//!
//! Deviation: the `grAADraw*` calls draw without antialiased edges. The output is drawn at
//! a multiple of the game's resolution, which smooths edges more than the Voodoo's edge
//! blending did.

use super::shadow::{self, Shadow};
use crate::cmd::{Cmd, Vertex, push};
use crate::consts::*;
use crate::vertex::{area, clip, culled, fan};
use std::sync::atomic::{AtomicU32, Ordering};

/// Triangles given and triangles drawn (not culled), for `grTriStats`.
pub static TRIS: [AtomicU32; 2] = [AtomicU32::new(0), AtomicU32::new(0)];

unsafe fn read(v: *const Vertex) -> Option<Vertex> {
    (!v.is_null()).then(|| unsafe { v.read_unaligned() })
}

/// A vertex as the render thread draws it.
fn prepare(mut v: Vertex, s: &Shadow) -> Vertex {
    if s.origin == ORIGIN_LOWER_LEFT {
        v.y = super::sst::screen().1 as f32 - v.y;
    }
    if s.delta0 != 0 {
        [v.r, v.g, v.b] = [s.color4[1], s.color4[2], s.color4[3]];
    }
    // With its own w, TMU 0's s/w and t/w are over that w: put them over the vertex's.
    if s.stw_hint & STWHINT_W_DIFF_TMU0 != 0 && v.tmu[0][2] != 0.0 {
        let k = v.oow / v.tmu[0][2];
        v.tmu[0][0] *= k;
        v.tmu[0][1] *= k;
    }
    v
}

/// One triangle in the game's coordinates: culled by its area there, then recorded.
fn triangle(t: [Vertex; 3], s: &Shadow) {
    TRIS[0].fetch_add(1, Ordering::Relaxed);
    if culled(s.cull, area(&t[0], &t[1], &t[2])) {
        return;
    }
    TRIS[1].fetch_add(1, Ordering::Relaxed);
    push(Cmd::Triangle(t.map(|v| prepare(v, s))));
}

/// A convex polygon as a fan of triangles.
fn polygon(vs: &[Vertex], s: &Shadow) {
    super::lfb::finish_write();
    for [a, b, c] in fan(vs.len()) {
        triangle([vs[a], vs[b], vs[c]], s);
    }
}

/// A polygon clipped to the clip window first (in the game's coordinates, as it set it).
fn clipped(vs: &[Vertex], s: &Shadow) {
    polygon(&clip(vs, s.clip.map(|v| v as f32)), s);
}

unsafe fn list(n: i32, vlist: *const Vertex) -> Vec<Vertex> {
    if vlist.is_null() || n < 3 {
        return Vec::new();
    }
    (0..n as usize).map(|i| unsafe { vlist.add(i).read_unaligned() }).collect()
}

unsafe fn indexed(n: i32, ilist: *const i32, vlist: *const Vertex) -> Vec<Vertex> {
    if ilist.is_null() || vlist.is_null() || n < 3 {
        return Vec::new();
    }
    (0..n as usize).map(|i| unsafe { vlist.add(ilist.add(i).read_unaligned() as usize).read_unaligned() }).collect()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawTriangle(a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    let (Some(a), Some(b), Some(c)) = (unsafe { read(a) }, unsafe { read(b) }, unsafe { read(c) }) else { return };
    super::lfb::finish_write();
    triangle([a, b, c], &shadow::get());
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawLine(a: *const Vertex, b: *const Vertex) {
    let (Some(a), Some(b)) = (unsafe { read(a) }, unsafe { read(b) }) else { return };
    super::lfb::finish_write();
    let s = shadow::get();
    push(Cmd::Line([prepare(a, &s), prepare(b, &s)]));
}

/// One pixel: the one that holds `(x, y)`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawPoint(p: *const Vertex) {
    let Some(mut p) = (unsafe { read(p) }) else { return };
    super::lfb::finish_write();
    let s = shadow::get();
    p = prepare(p, &s);
    (p.x, p.y) = (p.x.floor() + 0.5, p.y.floor() + 0.5);
    push(Cmd::Point(p));
}

/// A convex polygon of `n` vertices picked from `vlist` by `ilist`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawPolygon(n: i32, ilist: *const i32, vlist: *const Vertex) {
    polygon(&unsafe { indexed(n, ilist, vlist) }, &shadow::get());
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawPolygonVertexList(n: i32, vlist: *const Vertex) {
    polygon(&unsafe { list(n, vlist) }, &shadow::get());
}

/// A planar polygon has the same colour and depth gradients over all of it. Drawn as a fan,
/// its parameters are interpolated in each triangle, which gives the same result when they
/// do lie in one plane.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawPlanarPolygon(n: i32, ilist: *const i32, vlist: *const Vertex) {
    unsafe { grDrawPolygon(n, ilist, vlist) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grDrawPlanarPolygonVertexList(n: i32, vlist: *const Vertex) {
    unsafe { grDrawPolygonVertexList(n, vlist) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grAADrawPoint(p: *const Vertex) {
    unsafe { grDrawPoint(p) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grAADrawLine(a: *const Vertex, b: *const Vertex) {
    unsafe { grDrawLine(a, b) };
}

/// The three flags say which edges to antialias.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn grAADrawTriangle(a: *const Vertex, b: *const Vertex, c: *const Vertex, _ab: u32, _bc: u32, _ca: u32) {
    unsafe { grDrawTriangle(a, b, c) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grAADrawPolygon(n: i32, ilist: *const i32, vlist: *const Vertex) {
    unsafe { grDrawPolygon(n, ilist, vlist) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn grAADrawPolygonVertexList(n: i32, vlist: *const Vertex) {
    unsafe { grDrawPolygonVertexList(n, vlist) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guDrawTriangleWithClip(a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    let (Some(a), Some(b), Some(c)) = (unsafe { read(a) }, unsafe { read(b) }, unsafe { read(c) }) else { return };
    clipped(&[a, b, c], &shadow::get());
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guAADrawTriangleWithClip(a: *const Vertex, b: *const Vertex, c: *const Vertex) {
    unsafe { guDrawTriangleWithClip(a, b, c) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn guDrawPolygonVertexListWithClip(n: i32, vlist: *const Vertex) {
    clipped(&unsafe { list(n, vlist) }, &shadow::get());
}
