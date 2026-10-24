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
//! - Translucency is one 4-bit level per face (`trans_level`). The driver sorts translucent
//!   faces by pass, not by face: each call makes one pass for the faces kept as sent and one
//!   for those turned round. It draws the passes farthest first, and the faces in a pass in
//!   the order sent (`Frame::view`). `SGLTT_NEWPASSPERTRI` and `SGLTT_FORCEOPAQUE` are never
//!   tested in the immediate mode.
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
pub const VERTEXTRANS: u32 = 0x200;
pub const FACESIND3DFORMAT: u32 = 0x800;
pub const DISABLEZBUFFER: u32 = 0x2000;
pub const AVERAGECOLOUR: u32 = 0x4000;
/// Never read by SGLMID4.dll (*observed*), nor here.
pub const VERTEXFOG: u32 = 0x8000;
pub const OPAQUE: u32 = 0x40000;

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

/// A translucent face's level: 0 opaque to 15 the most see-through, one value per face. A face
/// is translucent with `SGLTT_GLOBALTRANS` and `u32GlobalTrans` below 0xf0 (level
/// `15 - (gt >> 4)`), or with `SGLTT_VERTEXTRANS`, whatever its alphas. Vertex alpha is the
/// sum of the first three vertices' alpha bytes, scaled by 170, or by `gt × 85 >> 7` with
/// global translucency too, and cut to 4 bits (*observed*: `FUN_1003daf0`, the alpha packers
/// `100420a0` and `10041d80`). `None`: an opaque face.
pub fn trans_level(flags: u32, gt: u32, vs: &[Vertex]) -> Option<u32> {
    let global = flags & GLOBALTRANS != 0 && gt < 0xf0;
    if flags & VERTEXTRANS != 0 {
        let sum: u32 = vs.iter().take(3).map(|v| v.colour >> 24).sum();
        let k = if global { gt * 85 >> 7 } else { 170 };
        Some(15 - ((sum * k >> 13) & 15))
    } else if global {
        Some(15 - (gt >> 4))
    } else {
        None
    }
}

/// The opacity of a level. *Inferred*: level 15 is fully clear; the chip's own scale is not in
/// the driver.
pub fn level_alpha(level: u32) -> f32 {
    (15 - level.min(15)) as f32 / 15.0
}

/// Twice the face's signed area on the screen, positive for a face clockwise on the screen.
pub fn area2(a: &Vertex, b: &Vertex, c: &Vertex) -> f32 {
    a.x * b.y - b.x * a.y + b.x * c.y - c.x * b.y + c.x * a.y - a.x * c.y
}

/// SGL's "Near Z Clip" (sgl.ini, default 0.01): the chip's depth is `NEAR_Z / w`
/// (`FUN_1003f510`, `FUN_1003fb58`; *observed*).
pub const NEAR_Z: f32 = 0.01;

/// The chip's fog constant. Deviation: *inferred* to lie between 0.5 and 1, because SGL drops
/// a mantissa when it sets the register for the scene graph (`FUN_10006bf0`); the middle is
/// taken. `SGL_FOG` scales the result.
pub const FOG_CONSTANT: f32 = 0.75;

/// The fog's rate, as the natural logarithm of what a surface loses per unit of `w`, from the
/// fog register (0 to 31). For the scene graph SGL writes the register so that a surface keeps
/// `2^(-c × 2^(R - 22) / depth)` (*inferred* from `sgl_set_fog` and `FUN_10006bf0`). The
/// immediate mode's depth is `NEAR_Z / w` with no other scale, so the rate per unit of `w` is
/// `ln 2 × c × 2^(R - 22) / NEAR_Z`.
pub fn rate(register: u32) -> f32 {
    std::f32::consts::LN_2 * FOG_CONSTANT * 2f32.powi(register.min(31) as i32 - 22) / NEAR_Z
}

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
        return Some(super::ERR_BAD_PARAMETER);
    }
    if mode == Some(0) {
        return Some(0);
    }
    if mode.is_none() || !pointers || n < 0 {
        return Some(super::ERR_BAD_PARAMETER);
    }
    None
}

/// How a batch takes part in the passes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pass {
    Opaque,
    /// `SGLTT_OPAQUE`: translucent faces in one pass after the opaque ones, in the order sent.
    Pseudo,
    /// A sorted translucent pass: one per call and facing. `key` is the largest, over its faces,
    /// of the `1/w` of each face's farthest vertex; `seq` the order sent.
    Sorted { key: f32, seq: u32 },
}

/// One frame of immediate-mode faces.
#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub background: Colour,
    pub batches: Vec<Batch>,
    /// The pass of each batch.
    pub passes: Vec<Pass>,
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
    /// vertices decide its facing and its alpha, as on the chip). `faces` index `verts`; a face
    /// with an index out of range is skipped. `translucent_map`: the context's texture is a
    /// translucent map.
    pub fn add<const N: usize>(&mut self, c: &Context, faces: &[[u32; N]], verts: &[Vertex], translucent_map: bool) {
        self.calls += 1;
        let flags = c.effective_flags();
        let textured = flags & TEXTURE != 0 && c.texture > 0;
        // The faces kept as sent and those turned round, with each set's key.
        let mut out: [Vec<Vert>; 2] = [Vec::new(), Vec::new()];
        let mut keys = [f32::NEG_INFINITY; 2];
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
            let turned = (a < 1.0 / 16.0) as usize;
            self.kept += 1;
            keys[turned] = keys[turned].max(vs[..3].iter().map(|v| v.inv_w).fold(f32::INFINITY, f32::min));
            let flat = if flags & AVERAGECOLOUR != 0 {
                let sum = vs.iter().map(|v| rgba(v.colour)).fold([0.0; 4], |s, x| [s[0] + x[0], s[1] + x[1], s[2] + x[2], s[3] + x[3]]);
                sum.map(|x| x / vs.len() as f32)
            } else {
                rgba(vs[0].colour)
            };
            let alpha = trans_level(flags, c.global_trans, &vs).map_or(1.0, level_alpha);
            let mut uvs: Vec<[f32; 2]> = vs.iter().map(|v| uv(c, v)).collect();
            if c.uv_times_inv_w {
                wrap(&mut uvs, flags);
            }
            let vert = |i: usize| -> Vert {
                let v = &vs[i];
                let col = if flags & GOURAUD != 0 { rgba(v.colour) } else { flat };
                let decal = textured && flags & DECAL != 0;
                let modulate = if decal { [1.0, 1.0, 1.0, alpha] } else { [col[0], col[1], col[2], alpha] };
                let spec = rgba(v.specular);
                let add = if flags & HIGHLIGHT != 0 { [spec[0], spec[1], spec[2]] } else { [0.0; 3] };
                let z = 1.0 / v.inv_w;
                Vert { p: [v.x * z, -v.y * z, z], uv: uvs[i], modulate, add }
            };
            let order: &[usize] = if N == 4 { &[0, 1, 2, 0, 2, 3] } else { &[0, 1, 2] };
            out[turned].extend(order.iter().map(|&i| vert(i)));
        }
        let translucent = trans_level(flags, c.global_trans, &[]).is_some() || (textured && translucent_map);
        let base = Batch {
            texture: if textured { c.texture as u32 } else { 0 },
            pass: 0,
            translucent,
            // `SGLTT_VERTEXFOG` and the specular alpha are never read (*observed*): `bFogOn`
            // alone decides.
            fog: c.fog_on,
            no_depth: flags & DISABLEZBUFFER != 0,
            verts: Vec::new(),
        };
        if !translucent || flags & OPAQUE != 0 {
            let pass = if translucent { Pass::Pseudo } else { Pass::Opaque };
            let verts: Vec<Vert> = out.into_iter().flatten().collect();
            if verts.is_empty() {
                return;
            }
            let same = self.batches.last().zip(self.passes.last()).is_some_and(|(b, &p)| {
                p == pass && b.texture == base.texture && b.fog == base.fog && b.no_depth == base.no_depth
            });
            if same {
                self.batches.last_mut().unwrap().verts.extend(verts);
            } else {
                self.batches.push(Batch { verts, ..base });
                self.passes.push(pass);
            }
            return;
        }
        for (turned, verts) in out.into_iter().enumerate() {
            if !verts.is_empty() {
                self.batches.push(Batch { verts, ..base.clone() });
                self.passes.push(Pass::Sorted { key: keys[turned], seq: self.calls * 2 + turned as u32 });
            }
        }
    }

    /// The frame as a view for the render thread, with the context given to `sgltri_render`.
    /// The camera is placed so the shader's projection gives back each vertex's screen place:
    /// a camera rectangle of (-1, -1) to (1, 1) at zoom 1. Its foreground distance is the
    /// nearest vertex's `w`, so the depth buffer's range covers the frame.
    ///
    /// The passes are numbered for the renderer, which draws them in that order: the opaque
    /// faces, then the `SGLTT_OPAQUE` pass, then the sorted passes farthest first (smallest
    /// key). A later pass goes before an earlier one of the same key (`FUN_100384b0`,
    /// `FUN_1003a2bc`; *observed*). Deviation: the driver sorts the passes per 32 × 32 region,
    /// with each region's own keys; here they are sorted once for the screen.
    pub fn view(&self, c: &Context) -> View {
        let fog_valid = c.fog_density < 32 && c.fog.iter().all(|x| (0.0..=1.0).contains(x));
        let (fog_colour, register) = if fog_valid { (c.fog, c.fog_density) } else { ([0.0; 3], 0) };
        let mut sorted: Vec<(f32, u32, usize)> = self
            .passes
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match *p {
                Pass::Sorted { key, seq } => Some((key, seq, i)),
                _ => None,
            })
            .collect();
        sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
        let mut batches = self.batches.clone();
        for (b, p) in batches.iter_mut().zip(&self.passes) {
            b.pass = (*p == Pass::Pseudo) as u32;
        }
        for (rank, &(_, _, i)) in sorted.iter().enumerate() {
            batches[i].pass = 2 + rank as u32;
        }
        let near = batches.iter().flat_map(|b| b.verts.iter().map(|v| v.p[2])).fold(f32::INFINITY, f32::min);
        let near = if near.is_finite() { near * 0.99 } else { 1.0 };
        // The render thread's fog is SGL's scene-graph density: what is lost per unit of `w`.
        let lost = -(-(rate(register) as f64)).exp_m1() as f32;
        let camera = Camera { zoom: 1.0, foreground: near, inv_background: 0.0, fog: Some((fog_colour, lost)), background: self.background };
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
        let view = f.view(&ctx(0));
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
    fn one_translucency_level_per_face() {
        let vs = [v(0.0, 0.0, 1.0, 0x80ffffff), v(10.0, 0.0, 1.0, 0x80ffffff), v(0.0, 10.0, 1.0, 0x80ffffff)];
        let full = [v(0.0, 0.0, 1.0, 0xffffffff); 3];
        // Global alone: 15 - (gt >> 4); opaque from 0xf0.
        assert_eq!(trans_level(GLOBALTRANS, 0x80, &[]), Some(7));
        assert_eq!(trans_level(GLOBALTRANS, 0xf0, &[]), None);
        // Vertex alpha: 3 × 0x80 × 170 >> 13 is 7, so level 8.
        assert_eq!(trans_level(VERTEXTRANS, 0, &vs), Some(8));
        // Full alpha is level 0, but still a translucent face.
        assert_eq!(trans_level(VERTEXTRANS, 0, &full), Some(0));
        // Both: gt = 0x80 scales by 85, so full alpha gives level 8, one more than global alone.
        assert_eq!(trans_level(VERTEXTRANS | GLOBALTRANS, 0x80, &full), Some(8));
        assert_eq!(level_alpha(0), 1.0);
        assert_eq!(level_alpha(15), 0.0);
    }

    #[test]
    fn translucency_and_decals() {
        let vs = [v(0.0, 0.0, 1.0, 0x80ffffff), v(10.0, 0.0, 1.0, 0xffffffff), v(0.0, 10.0, 1.0, 0x00ffffff)];
        let mut f = Frame::default();
        f.add(&ctx(VERTEXTRANS | GOURAUD), &[[0, 1, 2]], &vs, false);
        f.add(&Context { global_trans: 0xf0, ..ctx(GLOBALTRANS) }, &[[0, 1, 2]], &vs, false);
        f.add(&Context { texture: 3, ..ctx(TEXTURE | DECAL | VERTEXTRANS | HIGHLIGHT) }, &[[0, 1, 2]], &vs, false);
        f.add(&Context { texture: 3, ..ctx(TEXTURE) }, &[[0, 1, 2]], &vs, true);
        let b = &f.batches;
        // One alpha for the whole face, from the sum of its alphas.
        assert!(b[0].translucent && b[0].verts.iter().all(|v| v.modulate[3] == b[0].verts[0].modulate[3]));
        assert!(!b[1].translucent);
        assert_eq!((b[2].verts[0].modulate, b[2].texture, b[2].translucent), ([1.0; 4], 3, false));
        assert!(b[3].translucent);
    }

    #[test]
    fn translucent_passes_split_by_facing_and_sort_farthest_first() {
        let near = [v(0.0, 0.0, 0.5, 0), v(10.0, 0.0, 0.5, 0), v(0.0, 10.0, 0.5, 0)];
        let far = [v(0.0, 0.0, 0.1, 0), v(10.0, 0.0, 0.1, 0), v(0.0, 10.0, 0.2, 0)];
        let c = Context { global_trans: 0x80, ..ctx(GLOBALTRANS) };
        let mut f = Frame::default();
        // One call with a face kept and a face turned round: two passes.
        f.add(&c, &[[0, 1, 2], [0, 2, 1]], &near, false);
        f.add(&c, &[[0, 1, 2]], &far, false);
        f.add(&ctx(0), &[[0, 1, 2]], &far, false);
        f.add(&Context { global_trans: 0x80, ..ctx(GLOBALTRANS | OPAQUE) }, &[[0, 1, 2]], &near, false);
        assert_eq!(f.batches.len(), 5);
        let view = f.view(&ctx(0));
        let pass: Vec<u32> = view.batches.iter().map(|b| b.pass).collect();
        // The far call (key 0.1) first, then the near call's two facings, the later one first;
        // the opaque batch at 0 and the SGLTT_OPAQUE batch at 1.
        assert_eq!(pass, [4, 3, 2, 0, 1]);
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
    fn fog_ignores_the_vertex_fog_flag() {
        let vs = [v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)];
        let mut f = Frame::default();
        f.add(&Context { fog_on: true, ..ctx(VERTEXFOG) }, &[[0, 1, 2]], &vs, false);
        f.add(&ctx(VERTEXFOG), &[[0, 1, 2]], &vs, false);
        assert!(f.batches[0].fog && !f.batches[1].fog);
    }

    #[test]
    fn fog_rate_doubles_per_step_of_the_register() {
        let r = rate(10);
        assert!((rate(11) / r - 2.0).abs() < 1e-5);
        // Register 12 halves the colour about every 13.65 units of w (0.01 × 1024 / 0.75).
        assert!((std::f32::consts::LN_2 / rate(12) - 13.653).abs() < 0.01);
        let mut f = Frame::default();
        f.add(&Context { fog_on: true, ..ctx(0) }, &[[0, 1, 2]], &[v(0.0, 0.0, 1.0, 0), v(10.0, 0.0, 1.0, 0), v(0.0, 10.0, 1.0, 0)], false);
        // A density out of range writes register 0, as sgltri_render does: a faint fog.
        let lost = f.view(&Context { fog_density: 40, ..ctx(0) }).camera.fog.unwrap().1;
        assert!((lost - rate(0)).abs() / rate(0) < 1e-3);
    }

    #[test]
    fn shadow_arguments_in_the_drivers_order() {
        assert_eq!(shadow_args(0, None, false), Some(0));
        assert_eq!(shadow_args(101, Some(1), true), Some(-252));
        assert_eq!(shadow_args(5, Some(0), false), Some(0));
        assert_eq!(shadow_args(5, None, true), Some(-252));
        assert_eq!(shadow_args(5, Some(1), false), Some(-252));
        assert_eq!(shadow_args(-1, Some(2), true), Some(-252));
        assert_eq!(shadow_args(100, Some(2), true), None);
    }
}
