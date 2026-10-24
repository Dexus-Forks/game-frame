//! The light and shadow volume pass of the render thread: each pixel's surface tested against
//! a volume's planes (`walk::Volume`).

use super::super::gl::*;
use super::super::shaders::MAX_PLANES;
use super::super::ViewDraw;
use super::Renderer;
use crate::scene::walk::VolumeKind;

impl Renderer {
    /// Light and shadow volumes (sgl.h: a light volume's colour is added wherever it meets a
    /// surface; a shadow darkens it to its brightness). Each pixel's surface, rebuilt from the
    /// depth, is tested against the volume's planes. Where it is inside all of them, a light
    /// volume adds its colour (blend ONE, ONE) and a shadow volume multiplies by its
    /// brightness (blend ZERO, SRC_COLOR). Drawn into a frame buffer holding only the colour,
    /// so the depth texture can be read; then the scene's state is put back.
    pub(super) unsafe fn volumes(&self, vd: &ViewDraw, (x, y, w, h): (i32, i32, i32, i32), stride: i32) {
        let g = self.gpu.as_ref().unwrap();
        let gl = &g.gl;
        let fbo = g.fbo.as_ref().unwrap();
        let p = &g.programs.volume;
        let cam = &vd.view.camera;
        let [l, t, r, b] = vd.cam;
        let quad: [f32; 12] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        unsafe {
            (gl.BindFramebuffer)(FRAMEBUFFER, fbo.light);
            (gl.Scissor)(x, y, w, h);
            (gl.Disable)(DEPTH_TEST);
            (gl.Enable)(BLEND);
            (gl.UseProgram)(p.id);
            (gl.ActiveTexture)(TEXTURE0);
            (gl.BindTexture)(TEXTURE_2D, fbo.depth);
            (gl.Uniform1i)(p.loc("uDepth"), 0);
            (gl.Uniform2f)(p.loc("uSize"), fbo.w as f32, fbo.h as f32);
            (gl.Uniform4f)(p.loc("uCam"), (l + r) / 2.0, (t + b) / 2.0, cam.zoom * (r - l) / 2.0, cam.zoom * (b - t) / 2.0);
            (gl.Uniform2f)(p.loc("uDev"), vd.device.0 as f32, vd.device.1 as f32);
            (gl.Uniform1f)(p.loc("uNear"), cam.foreground.max(1e-3));
            for i in 1..4 {
                (gl.DisableVertexAttribArray)(i);
            }
            (gl.EnableVertexAttribArray)(0);
            (gl.BufferData)(ARRAY_BUFFER, (quad.len() * 4) as isize, quad.as_ptr().cast(), STREAM_DRAW);
            (gl.VertexAttribPointer)(0, 2, FLOAT, 0, 8, std::ptr::null());
            for v in &vd.view.volumes {
                let nothing = match v.kind {
                    VolumeKind::Light => [0.0; 3],
                    VolumeKind::Shadow => [1.0; 3],
                };
                if v.planes.is_empty() || v.colour == nothing {
                    continue;
                }
                match v.kind {
                    VolumeKind::Light => (gl.BlendFuncSeparate)(1, 1, 0, 1),          // ONE, ONE
                    VolumeKind::Shadow => (gl.BlendFuncSeparate)(0, 0x0300, 0, 1), // ZERO, SRC_COLOR
                }
                // Deviation: a volume of more than MAX_PLANES planes keeps the first ones (a
                // `sgltri_shadow` volume may have 100).
                let n = v.planes.len().min(MAX_PLANES);
                let flat: Vec<f32> = v.planes[..n].iter().flatten().copied().collect();
                (gl.Uniform4fv)(p.loc("uPlanes"), n as i32, flat.as_ptr());
                (gl.Uniform1i)(p.loc("uCount"), n as i32);
                (gl.Uniform3f)(p.loc("uColor"), v.colour[0], v.colour[1], v.colour[2]);
                (gl.DrawArrays)(TRIANGLES, 0, 6);
            }
            // Back to the scene's frame buffer and program for the translucent batches.
            (gl.BindFramebuffer)(FRAMEBUFFER, fbo.id);
            (gl.Disable)(BLEND);
            (gl.Enable)(DEPTH_TEST);
            (gl.DepthFunc)(0x0203);
            (gl.UseProgram)(g.programs.scene.id);
            for (i, (size, off)) in [(3, 0), (2, 3), (4, 5), (3, 9)].into_iter().enumerate() {
                (gl.EnableVertexAttribArray)(i as u32);
                (gl.VertexAttribPointer)(i as u32, size, FLOAT, 0, stride, (off * 4) as *const _);
            }
        }
    }
}
