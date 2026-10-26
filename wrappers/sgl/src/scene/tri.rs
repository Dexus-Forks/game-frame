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
//! - Texture filtering and mipmaps are not chosen per call (`FUN_1003daf0`). No immediate-mode
//!   code reads `SGLTT_BILINEAR` (0x80000): bilinear filtering is one global setting,
//!   `BilinearFilteringMode` in sglhw.ini, written to the chip at start-up (`FUN_10001e50`).
//!   There is no per-call mipmap flag: a texture is mipmapped when it was made with a
//!   mipmapped map type. `SGLTT_MIPMAPOFFSET` (0x400) copies `n32MipmapOffset`, which only
//!   the packer for mipmapped textures reads (`FUN_10048d60`), and which stays set until the
//!   next call that sets it. Deviation: this library ignores all three and filters every
//!   texture (`render`).
//! - Wrapping (`SGLTT_WRAPU`, `SGLTT_WRAPV`) moves vertices 1 and 2 by one whole turn at most
//!   toward vertex 0 (`wrap`; *measured*).
//! - Shadow and light-volume modes (`eShadowLightVolMode` at +0x34). Each `sgltri_triangles`
//!   or `sgltri_quads` call copies the brightness or light colour at +0x38 into its faces.
//!   A Gouraud face with a translucent map is drawn flat in those modes
//!   (`shadow_drops_gouraud`). `sgltri_shadow` makes one convex volume from its faces' planes
//!   (`scene::shadow`). The view carries the volumes; the render thread tests each pixel
//!   against their planes after the opaque faces, multiplying by a shadow's brightness or
//!   adding a light volume's colour. Deviation: the driver applies the volumes after every
//!   pass, translucent ones too (`FUN_100384b0`); here they are applied once, after the opaque
//!   faces and before the translucent ones.
//! - `sgltri_isrendercomplete` returns 0 when the chip is idle, 1 when busy with a timeout of
//!   0, and 2 when the timeout ran out.

use super::Colour;
use super::graph::Camera;
use super::shadow;
use super::walk::{Batch, Vert, View, Volume, VolumeKind};

mod input;
pub use input::*;

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

/// The chip's fog constant `c`. Deviation: the driver does not hold it. SGL bounds it to
/// 0.5..1, because it drops a mantissa when it sets the register for the scene graph
/// (`FUN_10006bf0`; *inferred*). The chip's fog is a base-2 curve from a table of 128 steps
/// (`FUN_10001e50` uploads it from `100c2528`; *measured*), which points to a power of two
/// (*inferred*, weak). 1.0 is a person's choice in that range; the chip's own value needs a
/// picture from a PCX2. `SGL_FOG` scales the result.
pub const FOG_CONSTANT: f32 = 1.0;

/// The fog's rate, as the natural logarithm of what a surface loses per unit of `w`, from the
/// fog register (0 to 31). For the scene graph SGL writes the register so that a surface keeps
/// `2^(-c × 2^(R - 22) / depth)` (*inferred* from `sgl_set_fog` and `FUN_10006bf0`). The
/// immediate mode's depth is `NEAR_Z / w` with no other scale, so the rate per unit of `w` is
/// `ln 2 × c × 2^(R - 22) / NEAR_Z`.
pub fn rate(register: u32) -> f32 {
    std::f32::consts::LN_2 * FOG_CONSTANT * 2f32.powi(register.min(31) as i32 - 22) / NEAR_Z
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
    /// The volumes of `sgltri_shadow`, in the order sent: their kind, their planes and the
    /// context's +0x38 at that call.
    volumes: Vec<(VolumeKind, Vec<[f32; 4]>, u32)>,
    /// The last +0x38 a `sgltri_triangles` or `sgltri_quads` call gave in shadow mode (1) and in
    /// light-volume mode (2).
    shade: [Option<u32>; 2],
}

/// True when a face loses its Gouraud shading: in shadow or light-volume mode, a Gouraud face
/// with a translucent map and neither `SGLTT_GLOBALTRANS` nor `SGLTT_VERTEXTRANS`
/// (`(flags & 0x221) == 1`). The driver's tables for those modes have no smooth translucent
/// format, so it picks the flat translucent one (`FUN_1003daf0`, tables `100c1b28` and
/// `100c1d28`; *observed*). `flags` are the effective flags.
pub fn shadow_drops_gouraud(c: &Context, flags: u32, translucent_map: bool) -> bool {
    c.shadow_mode != 0 && flags & TEXTURE != 0 && translucent_map && flags & (GOURAUD | GLOBALTRANS | VERTEXTRANS) == GOURAUD
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
        if c.shadow_mode != 0 {
            // Each call copies +0x38 into its faces (`10042650`; *observed*).
            self.shade[(c.shadow_mode != 1) as usize] = Some(c.shadow_value);
        }
        let mut flags = c.effective_flags();
        if shadow_drops_gouraud(c, flags, translucent_map) {
            flags &= !GOURAUD;
        }
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

    /// One `sgltri_shadow` volume, after the argument checks (`shadow::shadow_args`): the
    /// planes of its faces, kept for `view`. Deviation: the bounding box is not used; the
    /// driver files the volume only into the screen regions it covers (`10003080`).
    pub fn shadow(&mut self, c: &Context, faces: &[[u32; 3]], verts: &[Vertex]) {
        self.shadows += faces.len() as u32;
        let planes = shadow::volume(faces, verts);
        if !planes.is_empty() {
            self.volumes.push((shadow::kind(c.shadow_mode), planes, c.shadow_value));
        }
    }

    /// The volumes for the view. Deviation: one brightness and one light colour for the frame,
    /// the last a `sgltri_triangles` or `sgltri_quads` call gave in that mode (or, with none,
    /// the volume's own call's). The driver packs them into each face as it is sent
    /// (`1003c6c0`, `1003c4c0`), so each face keeps its own; and faces sent outside shadow
    /// mode are not darkened (*inferred*), where here everything inside is.
    fn volumes(&self) -> Vec<Volume> {
        self.volumes
            .iter()
            .map(|(kind, planes, own)| {
                let value = self.shade[(*kind == VolumeKind::Light) as usize].unwrap_or(*own);
                Volume { planes: planes.clone(), colour: shadow::volume_colour(*kind, value), kind: *kind }
            })
            .collect()
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
        View { camera, batches, volumes: self.volumes(), skipped: 0 }
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

/// Direct3D's wrapping (`SGLTT_WRAPU`, `SGLTT_WRAPV`), as the driver does it: for vertices 1
/// and 2, `d` is the coordinate less vertex 0's, as an `f32`. When `|d| >= 0.5` the coordinate
/// moves one step toward vertex 0: -1 when `d` is positive, +1 when it is negative, however
/// large `d` is. Vertex 0 never moves, and U and V are independent (Original: `10036b00`
/// (WRAPU), `10036930` (WRAPV), `100366d0` (both), steps at `100c9360`; *measured*,
/// `sgl2-measure.md`). A quad's fourth vertex is left as sent: the routines take three, and
/// the quad path's wrapping was not measured (*inferred*).
/// Deviation: the driver keeps `u ± 1` at x87 precision through its multiply by `1/w`; here
/// it is an `f32`, up to 1.2e-7 off.
fn wrap(uvs: &mut [[f32; 2]], flags: u32) {
    for (axis, bit) in [(0, WRAPU), (1, WRAPV)] {
        if flags & bit == 0 {
            continue;
        }
        let first = uvs[0][axis];
        for uv in uvs.iter_mut().skip(1).take(2) {
            let d = uv[axis] - first;
            if d.abs() >= 0.5 {
                uv[axis] += if d.is_sign_positive() { -1.0 } else { 1.0 };
            }
        }
    }
}

#[cfg(test)]
mod tests;
