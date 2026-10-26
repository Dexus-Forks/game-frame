//! The PCX2's per-pixel rules, as SGL 3.1 drives the chip, for a view of the scene graph
//! (`scene::walk::View`): triangles in camera space with their colours.
//!
//! The PowerVR chips do not draw triangles in order into a depth buffer: each region's
//! surfaces are sorted per pixel (deferred, the "infinite planes" of VideoLogic's design), so
//! the nearest opaque surface wins whatever order the game built the scene in. Here that is a
//! per-pixel 1/z compare that keeps the nearer surface; translucent passes follow, each blended
//! over what is in front of them. Then (sgl.h, the driver SGLMID4.dll, *observed* unless
//! marked):
//!
//! - projection: the camera rectangle's centre, and its half sizes times the camera's zoom
//!   (`FUN_10006bf0`, constants 0.5): a point `(x, y, z)` lands at `(cx + sx x / z,
//!   cy − sy y / z)`, y up;
//! - the near plane at the camera's foreground distance; nothing beyond `1 / inv_background`;
//! - a texel times the colour the texture effects send to it, plus the terms they leave out
//!   (`FUN_1001c170`; the sums are worked per vertex by `walk`);
//! - fog: a surface keeps `(1 − density)` of its colour per unit of depth, the rest the fog
//!   colour (`sgl_set_fog`, `FUN_10006bf0`; that the chip applies it as a power of the depth
//!   is *inferred*);
//! - light volumes add their light to the opaque surfaces inside them (sgl.h);
//! - a 565 frame buffer.

mod tri;

use crate::scene::graph::Texture;
use crate::scene::walk::{Batch, View, VolumeKind};
use std::collections::HashMap;

/// A viewport: its device pixels (inclusive corners) and its camera rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub rect: [i32; 4],
    pub cam: [f32; 4],
}

/// The colour buffer (RGB565) and the per-pixel depth (1/z; 0 for nothing drawn).
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub colour: Vec<u16>,
    pub depth: Vec<f32>,
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Frame {
        Frame { width, height, colour: vec![0; width * height], depth: vec![0.0; width * height] }
    }

    /// RGBA bytes, each channel's top bits repeated into its low ones.
    pub fn picture(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.colour.len() * 4);
        for &c in &self.colour {
            let (r, g, b) = ((c >> 11) as u8 & 31, (c >> 5) as u8 & 63, c as u8 & 31);
            out.extend_from_slice(&[r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2, 255]);
        }
        out
    }
}

/// A colour to 565. Deviation: rounded to the nearest level, without the chip's dither (the
/// PCX2's output dither is not taken from the driver).
pub fn to_565(c: [f32; 3]) -> u16 {
    let q = |v: f32, max: f32| (v.clamp(0.0, 1.0) * max + 0.5) as u16;
    q(c[0], 31.0) << 11 | q(c[1], 63.0) << 5 | q(c[2], 31.0)
}

/// The projection and the pixels one render covers, in output pixels.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Proj {
    pub cx: f32,
    pub cy: f32,
    pub kx: f32,
    pub ky: f32,
    pub near: f32,
    pub far: f32,
    /// x0, y0 inclusive; x1, y1 exclusive.
    pub clip: [usize; 4],
}

/// `sgl_render` of `view` into the viewport: clear to the camera's background, draw the
/// opaque surfaces, the light volumes, then the translucent passes in order.
pub fn render(frame: &mut Frame, view: &View, vp: &Viewport, device: (usize, usize), textures: &HashMap<u32, Texture>) {
    let (sx, sy) = (frame.width as f32 / device.0 as f32, frame.height as f32 / device.1 as f32);
    let cam = &view.camera;
    let [l, t, r, b] = vp.cam;
    let px = |v: f32, s: f32, max: usize| ((v * s).round().max(0.0) as usize).min(max);
    let clip = [
        px(vp.rect[0] as f32, sx, frame.width),
        px(vp.rect[1] as f32, sy, frame.height),
        px((vp.rect[2] + 1) as f32, sx, frame.width),
        px((vp.rect[3] + 1) as f32, sy, frame.height),
    ];
    let proj = Proj {
        cx: (l + r) / 2.0 * sx,
        cy: (t + b) / 2.0 * sy,
        kx: cam.zoom * (r - l) / 2.0 * sx,
        ky: cam.zoom * (b - t) / 2.0 * sy,
        near: cam.foreground.max(1e-4),
        far: if cam.inv_background > 0.0 { 1.0 / cam.inv_background } else { f32::INFINITY },
        clip,
    };
    // Each pixel's colour before it goes to 565, and the fog's keep rate.
    let (w, h) = (clip[2].saturating_sub(clip[0]), clip[3].saturating_sub(clip[1]));
    let mut acc = vec![cam.background; w * h];
    for y in clip[1]..clip[3] {
        frame.depth[y * frame.width + clip[0]..y * frame.width + clip[2]].fill(0.0);
    }
    let fog = cam.fog.map(|(c, d)| (c, (1.0 - d).clamp(0.0, 1.0)));
    let mut order: Vec<&Batch> = view.batches.iter().collect();
    order.sort_by_key(|b| (b.translucent, b.pass));
    let mut volumes_done = false;
    for b in order {
        if b.translucent && !volumes_done {
            volumes(frame, &mut acc, view, &proj);
            volumes_done = true;
        }
        let tex = textures.get(&b.texture);
        let fog = if b.fog { fog } else { None };
        for t in b.verts.chunks_exact(3) {
            tri::draw(frame, &mut acc, &proj, [t[0], t[1], t[2]], tex, fog, b.translucent, b.no_depth);
        }
    }
    if !volumes_done {
        volumes(frame, &mut acc, view, &proj);
    }
    for y in clip[1]..clip[3] {
        for x in clip[0]..clip[2] {
            frame.colour[y * frame.width + x] = to_565(acc[(y - clip[1]) * w + x - clip[0]]);
        }
    }
}

/// Light and shadow volumes on the opaque surfaces: each pixel's point in the camera's space,
/// back from its 1/z, tested against every plane of each volume.
/// Deviation: the volume's light goes onto the fogged colour, and a surface inside it is lit
/// by its full colour; how strongly the PCX2 lit what a light volume meets is not known (POD's
/// docs/hardware/3d-cards.md, "Open").
fn volumes(frame: &Frame, acc: &mut [[f32; 3]], view: &View, p: &Proj) {
    if view.volumes.is_empty() {
        return;
    }
    let w = p.clip[2] - p.clip[0];
    for y in p.clip[1]..p.clip[3] {
        for x in p.clip[0]..p.clip[2] {
            let iz = frame.depth[y * frame.width + x];
            if iz <= 0.0 {
                continue;
            }
            let z = 1.0 / iz;
            let q = [(x as f32 + 0.5 - p.cx) / p.kx * z, -(y as f32 + 0.5 - p.cy) / p.ky * z, z];
            let c = &mut acc[(y - p.clip[1]) * w + x - p.clip[0]];
            for v in &view.volumes {
                if v.planes.iter().all(|pl| pl[0] * q[0] + pl[1] * q[1] + pl[2] * q[2] <= pl[3] + 1e-4) {
                    for k in 0..3 {
                        c[k] = match v.kind {
                            VolumeKind::Light => (c[k] + v.colour[k]).min(1.0),
                            VolumeKind::Shadow => c[k] * v.colour[k],
                        };
                    }
                }
            }
        }
    }
}
