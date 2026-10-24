//! SGL's immediate mode (`sgltri_*`). The game sends triangles and quads each frame, already
//! projected to the screen, as Direct3D's transform stage writes them. `sgltri_startofframe`
//! opens a frame. Each `sgltri_triangles` or `sgltri_quads` adds faces under its context.
//! `sgltri_render` draws them. This module turns the game's contexts and vertices into the
//! batches the render thread draws for the scene graph. Plain Rust, tested on the host.
//!
//! The details the header leaves open come from PowerVR's own implementation (SGLMID4.dll of
//! driver 3.1.0; *observed* in its code):
//!
//! - `SGLCONTEXT` is 0x90 bytes. `sgltri_triangles` copies 0x24 words of it (`FUN_1003daf0`).
//! - A face keeps its winding when twice its screen area, `x0 y1 - x1 y0 + x1 y2 - x2 y1 +
//!   x2 y0 - x0 y2` (y down), is at least 1/16. Otherwise it is culled when
//!   `bCullBackfacing` is set. Without that flag, a face whose area is below 1/16 the other
//!   way is dropped. Any other face is turned round (`FUN_1003f510`, the quad loop). So a
//!   face clockwise on the screen faces the camera. A face of less than 1/32 pixel is never
//!   drawn.
//! - Flat shading takes the first vertex's colour (`FUN_1003f510` reads `u32Colour` of the
//!   face's first vertex). `SGLTT_AVERAGECOLOUR` picks the routines that average them.
//! - `SGLTT_TEXTURE | SGLTT_DECAL` clears `SGLTT_HIGHLIGHT`, `SGLTT_GLOBALTRANS` and
//!   `SGLTT_VERTEXTRANS` (`& 0xfffffddb`).
//! - `SGLTT_GLOBALTRANS` with `u32GlobalTrans` below 0xf0 sets one of 16 levels,
//!   `15 - (u32GlobalTrans >> 4)`. A value of 0xf0 and above stays opaque.
//! - Fog: `bFogOn` is read per call. The colour and the density (0 to 31) come from the
//!   context given to `sgltri_render`, which writes the density into the chip's fog register
//!   unchanged (`sgltri_render`, `FUN_100019b0`). `rate` below says how that is read here.
//! - `sgltri_isrendercomplete` returns 0 when the chip is idle, 1 when busy with a timeout of
//!   0, and 2 when the timeout ran out.

use super::Colour;
use super::graph::Camera;
use super::walk::{Batch, Vert, View};

pub const GOURAUD: u32 = 0x1;
pub const TEXTURE: u32 = 0x2;
pub const HIGHLIGHT: u32 = 0x4;
pub const DECAL: u32 = 0x8;
pub const GLOBALTRANS: u32 = 0x20;
pub const WRAPU: u32 = 0x40;
pub const WRAPV: u32 = 0x80;
pub const FORCEOPAQUE: u32 = 0x100;
pub const VERTEXTRANS: u32 = 0x200;
pub const FACESIND3DFORMAT: u32 = 0x800;
pub const DISABLEZBUFFER: u32 = 0x2000;
pub const AVERAGECOLOUR: u32 = 0x4000;
pub const VERTEXFOG: u32 = 0x8000;

/// `sizeof(SGLCONTEXT)` and `sizeof(SGLVERTEX)` (sgl.h, packed).
pub const CONTEXT_SIZE: usize = 0x90;
pub const VERTEX_SIZE: usize = 32;

fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn float(b: &[u8], at: usize) -> f32 {
    f32::from_bits(word(b, at))
}

/// The fields of `SGLCONTEXT` this library reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub fog_on: bool,
    pub fog: Colour,
    pub fog_density: u32,
    pub cull: bool,
    pub flags: u32,
    pub texture: i32,
    pub background: Colour,
    /// `eShadowLightVolMode`: 0 none, 1 shadows, 2 light volumes.
    pub shadow_mode: u32,
    pub flip: (bool, bool),
    pub uv_times_inv_w: bool,
    pub global_trans: u32,
}

impl Context {
    /// From the game's `SGLCONTEXT`, at least `CONTEXT_SIZE` bytes.
    pub fn read(b: &[u8]) -> Context {
        Context {
            fog_on: word(b, 0x00) != 0,
            fog: [float(b, 0x04), float(b, 0x08), float(b, 0x0c)],
            fog_density: word(b, 0x10),
            cull: word(b, 0x14) != 0,
            flags: word(b, 0x18),
            texture: word(b, 0x1c) as i32,
            background: [float(b, 0x28), float(b, 0x2c), float(b, 0x30)],
            shadow_mode: word(b, 0x34),
            flip: (word(b, 0x3c) != 0, word(b, 0x40) != 0),
            uv_times_inv_w: word(b, 0x44) != 0,
            global_trans: word(b, 0x60),
        }
    }

    /// The flags as the triangle routines see them: a decal drops the highlight and both kinds
    /// of translucency.
    pub fn effective_flags(&self) -> u32 {
        if self.flags & (TEXTURE | DECAL) == TEXTURE | DECAL { self.flags & 0xfffffddb } else { self.flags }
    }
}

/// `SGLVERTEX`: a Direct3D `D3DTLVERTEX`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub inv_w: f32,
    /// `D3DCOLOR`: 0xAARRGGBB.
    pub colour: u32,
    pub specular: u32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn read(b: &[u8]) -> Vertex {
        Vertex {
            x: float(b, 0),
            y: float(b, 4),
            z: float(b, 8),
            inv_w: float(b, 12),
            colour: word(b, 16),
            specular: word(b, 20),
            u: float(b, 24),
            v: float(b, 28),
        }
    }
}

/// A `D3DCOLOR` as red, green, blue and alpha from 0 to 1.
pub fn rgba(c: u32) -> [f32; 4] {
    let f = |s: u32| ((c >> s) & 255) as f32 / 255.0;
    [f(16), f(8), f(0), f(24)]
}

/// The opacity `u32GlobalTrans` gives: one of 16 levels, opaque from 0xf0.
pub fn global_alpha(t: u32) -> f32 {
    if t >= 0xf0 { 1.0 } else { (t >> 4) as f32 / 15.0 }
}

/// Twice the face's signed area on the screen, positive for a face clockwise on the screen.
pub fn area2(a: &Vertex, b: &Vertex, c: &Vertex) -> f32 {
    a.x * b.y - b.x * a.y + b.x * c.y - c.x * b.y + c.x * a.y - a.x * c.y
}

/// The fog's rate, as the natural logarithm of what a surface keeps per unit of `w`, from the
/// chip's fog register and the device's smaller side. *Inferred*: for the scene graph, SGL
/// writes into the same register the exponent of `0.1 × foreground × -log2(1 - density)`
/// plus 22, less the doublings that take the device's smaller side to 2048 (`FUN_10006bf0`),
/// so the register is a power of two on that rate. Here it is read back the same way, with
/// each exponent taken as half way between its powers of two.
pub fn rate(register: u32, device_side: i32) -> f32 {
    let mut side = device_side.max(1);
    let mut shift = 0;
    while side < 2048 {
        side *= 2;
        shift += 1;
    }
    let log2_rate = 2f32.powi(register.min(31) as i32 - 22 + shift) / (2.0 * 0.1);
    log2_rate * std::f32::consts::LN_2
}

/// One frame of immediate-mode faces.
#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub background: Colour,
    pub batches: Vec<Batch>,
    /// Per vertex of each batch: the fog factor `SGLTT_VERTEXFOG` gives (1 keeps all the
    /// colour), folded in by `view` once the fog colour is known.
    pub vertex_fog: Vec<Vec<f32>>,
    /// Faces sent and faces drawn, for the log.
    pub sent: u32,
    pub kept: u32,
    pub shadows: u32,
    calls: u32,
}

impl Frame {
    pub fn start(&mut self, c: &Context) {
        *self = Frame { background: c.background, ..Frame::default() };
    }

    /// Add faces of 3 or 4 vertices (a quad is drawn as two triangles; its first three
    /// vertices decide its facing, as on the chip). `faces` index `verts`; a face with an
    /// index out of range is skipped. `translucent_map`: the context's texture is a
    /// translucent map.
    pub fn add<const N: usize>(&mut self, c: &Context, faces: &[[u32; N]], verts: &[Vertex], translucent_map: bool) {
        self.calls += 1;
        let flags = c.effective_flags();
        let textured = flags & TEXTURE != 0 && c.texture > 0;
        let mut out: Vec<Vert> = Vec::new();
        let mut fogs: Vec<f32> = Vec::new();
        for f in faces {
            self.sent += 1;
            let Some(vs) = f.iter().map(|&i| verts.get(i as usize).copied()).collect::<Option<Vec<_>>>() else { continue };
            if vs.iter().any(|v| !(v.inv_w > 0.0 && v.inv_w.is_finite() && v.x.is_finite() && v.y.is_finite())) {
                continue;
            }
            let a = area2(&vs[0], &vs[1], &vs[2]);
            if a < 1.0 / 16.0 && (c.cull || -a < 1.0 / 16.0) {
                continue;
            }
            self.kept += 1;
            let flat = if flags & AVERAGECOLOUR != 0 {
                let sum = vs.iter().map(|v| rgba(v.colour)).fold([0.0; 4], |s, x| [s[0] + x[0], s[1] + x[1], s[2] + x[2], s[3] + x[3]]);
                sum.map(|x| x / vs.len() as f32)
            } else {
                rgba(vs[0].colour)
            };
            let mut uvs: Vec<[f32; 2]> = vs.iter().map(|v| uv(c, v)).collect();
            if c.uv_times_inv_w {
                wrap(&mut uvs, flags);
            }
            let vert = |i: usize| -> (Vert, f32) {
                let v = &vs[i];
                let col = if flags & GOURAUD != 0 { rgba(v.colour) } else { flat };
                let mut alpha = 1.0;
                if flags & VERTEXTRANS != 0 {
                    alpha *= col[3];
                }
                if flags & GLOBALTRANS != 0 {
                    alpha *= global_alpha(c.global_trans);
                }
                if flags & FORCEOPAQUE != 0 {
                    alpha = 1.0;
                }
                let decal = textured && flags & DECAL != 0;
                let modulate = if decal { [1.0, 1.0, 1.0, alpha] } else { [col[0], col[1], col[2], alpha] };
                let spec = rgba(v.specular);
                let add = if flags & HIGHLIGHT != 0 { [spec[0], spec[1], spec[2]] } else { [0.0; 3] };
                let z = 1.0 / v.inv_w;
                let fog = if c.fog_on && flags & VERTEXFOG != 0 { spec[3] } else { 1.0 };
                (Vert { p: [v.x * z, -v.y * z, z], uv: uvs[i], modulate, add }, fog)
            };
            let order: &[usize] = if N == 4 { &[0, 1, 2, 0, 2, 3] } else { &[0, 1, 2] };
            for &i in order {
                let (v, f) = vert(i);
                out.push(v);
                fogs.push(f);
            }
        }
        if out.is_empty() {
            return;
        }
        let translucent = (textured && translucent_map) || out.iter().any(|v| v.modulate[3] < 1.0);
        let batch = Batch {
            texture: if textured { c.texture as u32 } else { 0 },
            // Deviation: translucent faces are drawn in the order sent, after the opaque ones;
            // the PCX2 sorted them itself.
            pass: if translucent { self.calls } else { 0 },
            translucent,
            fog: c.fog_on && flags & VERTEXFOG == 0,
            no_depth: flags & DISABLEZBUFFER != 0,
            verts: Vec::new(),
        };
        let last = self.batches.len().wrapping_sub(1);
        let same = self.batches.last().is_some_and(|b| {
            !translucent && b.texture == batch.texture && b.pass == 0 && !b.translucent && b.fog == batch.fog && b.no_depth == batch.no_depth
        });
        if same {
            self.batches[last].verts.extend(out);
            self.vertex_fog[last].extend(fogs);
        } else {
            self.batches.push(Batch { verts: out, ..batch });
            self.vertex_fog.push(fogs);
        }
    }

    /// The frame as a view for the render thread, with the context given to `sgltri_render`.
    /// The camera is placed so the shader's projection gives back each vertex's screen place:
    /// a camera rectangle of (-1, -1) to (1, 1) at zoom 1. Its foreground distance is the
    /// nearest vertex's `w`, so the depth buffer's range covers the frame.
    pub fn view(&self, c: &Context, device: (i32, i32)) -> View {
        let fog_valid = c.fog_density < 32 && c.fog.iter().all(|x| (0.0..=1.0).contains(x));
        let (fog_colour, register) = if fog_valid { (c.fog, c.fog_density) } else { ([0.0; 3], 0) };
        let mut batches = self.batches.clone();
        for (b, fogs) in batches.iter_mut().zip(&self.vertex_fog) {
            for (v, &f) in b.verts.iter_mut().zip(fogs) {
                if f < 1.0 {
                    for ((m, a), fc) in v.modulate.iter_mut().zip(&mut v.add).zip(fog_colour) {
                        *m *= f;
                        *a = *a * f + fc * (1.0 - f);
                    }
                }
            }
        }
        let near = batches.iter().flat_map(|b| b.verts.iter().map(|v| v.p[2])).fold(f32::INFINITY, f32::min);
        let near = if near.is_finite() { near * 0.99 } else { 1.0 };
        // The render thread's fog is SGL's scene-graph density: what is lost per unit.
        let keep = (-rate(register, device.0.min(device.1))).exp();
        let camera = Camera { zoom: 1.0, foreground: near, inv_background: 0.0, fog: Some((fog_colour, 1.0 - keep)), background: self.background };
        View { camera, batches, volumes: Vec::new() }
    }
}

/// The texture coordinates as OpenGL takes them (its interpolation divides by `w` itself).
fn uv(c: &Context, v: &Vertex) -> [f32; 2] {
    let (mut u, mut t) = if c.uv_times_inv_w { (v.u, v.v) } else { (v.u / v.inv_w, v.v / v.inv_w) };
    if c.flip.0 {
        u = 1.0 - u;
    }
    if c.flip.1 {
        t = 1.0 - t;
    }
    [u, t]
}

/// Direct3D's wrapping (`SGLTT_WRAPU`, `SGLTT_WRAPV`): a coordinate takes the shorter way
/// round the texture, so each vertex is moved by whole turns to within half of the first.
fn wrap(uvs: &mut [[f32; 2]], flags: u32) {
    for (axis, bit) in [(0, WRAPU), (1, WRAPV)] {
        if flags & bit == 0 {
            continue;
        }
        let first = uvs[0][axis];
        for uv in uvs.iter_mut().skip(1) {
            uv[axis] -= (uv[axis] - first).round();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, inv_w: f32, colour: u32) -> Vertex {
        Vertex { x, y, z: 0.5, inv_w, colour, specular: 0xff000000, u: 0.0, v: 0.0 }
    }

    fn ctx(flags: u32) -> Context {
        Context { flags, ..Context::default() }
    }

    #[test]
    fn reads_the_context_at_its_offsets() {
        let mut b = vec![0u8; CONTEXT_SIZE];
        b[0x00] = 1;
        b[0x04..0x08].copy_from_slice(&0.5f32.to_le_bytes());
        b[0x10] = 12;
        b[0x14] = 1;
        b[0x18..0x1c].copy_from_slice(&(GOURAUD | TEXTURE).to_le_bytes());
        b[0x1c] = 7;
        b[0x2c..0x30].copy_from_slice(&0.25f32.to_le_bytes());
        b[0x34] = 2;
        b[0x40] = 1;
        b[0x44] = 1;
        b[0x60] = 0x80;
        let c = Context::read(&b);
        assert!(c.fog_on && c.cull && c.uv_times_inv_w);
        assert_eq!((c.fog[0], c.fog_density, c.flags, c.texture), (0.5, 12, GOURAUD | TEXTURE, 7));
        assert_eq!((c.background[1], c.shadow_mode, c.flip, c.global_trans), (0.25, 2, (false, true), 0x80));
    }

    #[test]
    fn reads_a_vertex() {
        let mut b = [0u8; VERTEX_SIZE];
        for (i, f) in [10.0f32, 20.0, 0.5, 2.0].iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
        }
        b[16..20].copy_from_slice(&0x80ff0000u32.to_le_bytes());
        b[24..28].copy_from_slice(&0.75f32.to_le_bytes());
        let v = Vertex::read(&b);
        assert_eq!((v.x, v.y, v.inv_w, v.colour, v.u), (10.0, 20.0, 2.0, 0x80ff0000, 0.75));
        assert_eq!(rgba(v.colour), [1.0, 0.0, 0.0, 128.0 / 255.0]);
    }

    #[test]
    fn culls_and_drops_faces_as_the_chip_does() {
        let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0), v(0.1, 0.0, 1.0, 0)];
        let mut f = Frame::default();
        // Clockwise on the screen (y down): kept either way.
        f.add(&Context { cull: true, ..ctx(0) }, &[[0, 1, 2]], &vs, false);
        // Counter-clockwise: culled, or kept turned round.
        f.add(&Context { cull: true, ..ctx(0) }, &[[0, 2, 1]], &vs, false);
        f.add(&ctx(0), &[[0, 2, 1]], &vs, false);
        // Less than 1/32 of a pixel: dropped.
        f.add(&ctx(0), &[[0, 3, 0]], &vs, false);
        // An index past the vertices: skipped.
        f.add(&ctx(0), &[[0, 1, 9]], &vs, false);
        assert_eq!((f.sent, f.kept), (5, 2));
    }

    #[test]
    fn places_vertices_so_the_projection_gives_them_back() {
        let mut f = Frame::default();
        f.add(&ctx(0), &[[0, 1, 2]], &[v(100.0, 50.0, 0.5, 0), v(200.0, 50.0, 0.25, 0), v(100.0, 150.0, 0.5, 0)], false);
        let view = f.view(&ctx(0), (640, 480));
        let p = view.batches[0].verts[1].p;
        // The shader puts (x, y, z) at (x / z, -y / z) with this camera.
        assert_eq!(p, [800.0, -200.0, 4.0]);
        assert_eq!(view.camera.zoom, 1.0);
        assert!((view.camera.foreground - 2.0 * 0.99).abs() < 1e-6);
    }

    #[test]
    fn shades_flat_from_the_first_vertex_or_the_average() {
        let vs = [v(0.0, 0.0, 1.0, 0xffff0000), v(10.0, 0.0, 1.0, 0xff0000ff), v(0.0, 10.0, 1.0, 0xff00ff00)];
        let mut f = Frame::default();
        f.add(&ctx(0), &[[0, 1, 2]], &vs, false);
        f.add(&ctx(GOURAUD), &[[0, 1, 2]], &vs, false);
        f.add(&ctx(AVERAGECOLOUR), &[[0, 1, 2]], &vs, false);
        let all: Vec<&Vert> = f.batches.iter().flat_map(|b| &b.verts).collect();
        assert_eq!(all[1].modulate, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(all[4].modulate, [0.0, 0.0, 1.0, 1.0]);
        let third = 1.0 / 3.0;
        assert!(all[7].modulate.iter().zip([third, third, third, 1.0]).all(|(a, b)| (a - b).abs() < 1e-6));
    }

    #[test]
    fn translucency_and_decals() {
        let vs = [v(0.0, 0.0, 1.0, 0x80ffffff), v(10.0, 0.0, 1.0, 0x80ffffff), v(0.0, 10.0, 1.0, 0x80ffffff)];
        let mut f = Frame::default();
        f.add(&ctx(VERTEXTRANS), &[[0, 1, 2]], &vs, false);
        f.add(&Context { global_trans: 0x7f, ..ctx(GLOBALTRANS) }, &[[0, 1, 2]], &vs, false);
        f.add(&Context { global_trans: 0xf0, ..ctx(GLOBALTRANS) }, &[[0, 1, 2]], &vs, false);
        f.add(&Context { texture: 3, ..ctx(TEXTURE | DECAL | VERTEXTRANS | HIGHLIGHT) }, &[[0, 1, 2]], &vs, false);
        let b = &f.batches;
        assert!(b[0].translucent && (b[0].verts[0].modulate[3] - 128.0 / 255.0).abs() < 1e-6);
        assert!(b[1].translucent && (b[1].verts[0].modulate[3] - 7.0 / 15.0).abs() < 1e-6);
        assert!(b[1].pass > b[0].pass);
        assert!(!b[2].translucent);
        assert_eq!(b[3].verts[0].modulate, [1.0; 4]);
        assert_eq!(b[3].texture, 3);
    }

    #[test]
    fn opaque_calls_of_one_state_share_a_batch() {
        let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0), v(10.0, 10.0, 1.0, 0)];
        let mut f = Frame::default();
        f.add(&ctx(0), &[[0, 1, 2]], &vs, false);
        f.add(&ctx(0), &[[0, 1, 3, 2]], &vs, false);
        f.add(&ctx(DISABLEZBUFFER), &[[0, 1, 2]], &vs, false);
        assert_eq!(f.batches.len(), 2);
        assert_eq!(f.batches[0].verts.len(), 9);
        assert!(f.batches[1].no_depth);
    }

    #[test]
    fn texture_coordinates_and_wrapping() {
        let mut a = v(0.0, 0.0, 0.5, 0);
        (a.u, a.v) = (0.25, 0.5);
        assert_eq!(uv(&ctx(0), &a), [0.5, 1.0]);
        assert_eq!(uv(&Context { uv_times_inv_w: true, flip: (true, false), ..ctx(0) }, &a), [0.75, 0.5]);
        let mut uvs = [[0.9, 0.1], [0.1, 0.9], [0.5, 0.5]];
        wrap(&mut uvs, WRAPU);
        assert!((uvs[1][0] - 1.1).abs() < 1e-6 && uvs[1][1] == 0.9);
    }

    #[test]
    fn vertex_fog_folds_into_the_colours() {
        let mut vs = [v(0.0, 0.0, 1.0, 0xffffffff), v(10.0, 0.0, 1.0, 0xffffffff), v(0.0, 10.0, 1.0, 0xffffffff)];
        vs[0].specular = 0x40000000;
        let mut f = Frame::default();
        let c = Context { fog_on: true, fog: [0.0, 0.0, 1.0], ..ctx(GOURAUD | VERTEXFOG) };
        f.add(&c, &[[0, 1, 2]], &vs, false);
        let view = f.view(&c, (640, 480));
        let b = &view.batches[0];
        assert!(!b.fog);
        let k = 64.0 / 255.0;
        assert!((b.verts[0].modulate[0] - k).abs() < 1e-6 && (b.verts[0].add[2] - (1.0 - k)).abs() < 1e-6);
        assert_eq!(b.verts[1].modulate[0], 1.0);
    }

    #[test]
    fn fog_rate_doubles_per_step_of_the_register() {
        let r = rate(10, 480);
        assert!((rate(11, 480) / r - 2.0).abs() < 1e-5);
        // 480 doubles 3 times to reach 2048, 1024 once.
        assert!((rate(10, 1024) / r - 0.25).abs() < 1e-5);
        let mut f = Frame::default();
        f.add(&Context { fog_on: true, ..ctx(0) }, &[[0, 1, 2]], &[v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)], false);
        assert!(f.batches[0].fog);
        // A density out of range turns the fog off, as sgltri_render does.
        let view = f.view(&Context { fog_density: 40, ..ctx(0) }, (640, 480));
        assert!(view.camera.fog.unwrap().1 < 1e-5);
    }
}
