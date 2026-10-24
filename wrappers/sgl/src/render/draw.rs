//! What the render thread does with each command. A frame's views are drawn into an
//! off-screen buffer of the device's size times the scale, which is copied into the window
//! where the game's picture puts the device; then the game's picture goes on top, except
//! where it still holds the marker.

use super::gl::*;
use super::shaders::{MAX_PLANES, Programs, SCENE_FLOATS};
use super::win32::*;
use super::{Cmd, Present, ViewDraw};
use crate::scene::graph::Texture;
use std::collections::HashMap;
use std::ptr::null_mut;

struct Fbo {
    id: u32,
    color: u32,
    /// The depth, as a texture: the light volume pass reads it.
    depth: u32,
    /// The same colour with no depth, to draw light volumes into while reading the depth.
    light: u32,
    w: i32,
    h: i32,
}

struct Gpu {
    gl: Gl,
    programs: Programs,
    dc: HDC,
    hwnd: HWND,
    vbo: u32,
    overlay_tex: u32,
    overlay_size: (u32, u32),
    fbo: Option<Fbo>,
}

pub struct Renderer {
    gpu: Option<Gpu>,
    failed: bool,
    textures: HashMap<u32, (Texture, u32)>,
    frame: Vec<ViewDraw>,
    /// Pictures shown since the last frame came.
    age: u32,
    shown: u32,
    /// Every picture shown, for the captures' names.
    pictures: u32,
    since: std::time::Instant,
    scratch: Vec<f32>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            gpu: None,
            failed: false,
            textures: HashMap::new(),
            frame: Vec::new(),
            age: 0,
            shown: 0,
            pictures: 0,
            since: std::time::Instant::now(),
            scratch: Vec::new(),
        }
    }

    pub fn exec(&mut self, c: Cmd) {
        match c {
            Cmd::Texture(id, t) => {
                let old = self.textures.insert(id, (t, 0));
                self.free(old.map_or(0, |o| o.1));
            }
            Cmd::DeleteTexture(id) => {
                let old = self.textures.remove(&id);
                self.free(old.map_or(0, |o| o.1));
            }
            Cmd::Frame(v) => {
                self.frame = v;
                self.age = 0;
            }
            Cmd::Present(p) => unsafe { self.present(p) },
        }
    }

    fn free(&self, tex: u32) {
        if let (Some(g), true) = (&self.gpu, tex != 0) {
            unsafe { (g.gl.DeleteTextures)(1, &tex) };
        }
    }

    unsafe fn open(&mut self, hwnd: HWND) -> Result<(), String> {
        unsafe {
            if let Some(g) = &mut self.gpu {
                if g.hwnd != hwnd {
                    let dc = pixel_format(hwnd)?;
                    if wglMakeCurrent(dc, wglGetCurrentContext()) == 0 {
                        return Err("cannot move the context to the new window".into());
                    }
                    (g.dc, g.hwnd) = (dc, hwnd);
                }
                return Ok(());
            }
            let dc = pixel_format(hwnd)?;
            let rc = wglCreateContext(dc);
            if rc.is_null() || wglMakeCurrent(dc, rc) == 0 {
                return Err("wglCreateContext failed".into());
            }
            let gl = Gl::load()?;
            let s = |n| std::ffi::CStr::from_ptr((gl.GetString)(n).cast()).to_string_lossy().into_owned();
            shim::log!("OpenGL {} on {}", s(VERSION), s(RENDERER));
            let programs = super::shaders::build(&gl)?;
            let mut ids = [0u32; 2];
            (gl.GenBuffers)(1, &mut ids[0]);
            (gl.GenTextures)(1, &mut ids[1]);
            (gl.Disable)(CULL_FACE);
            self.gpu = Some(Gpu { gl, programs, dc, hwnd, vbo: ids[0], overlay_tex: ids[1], overlay_size: (0, 0), fbo: None });
        }
        Ok(())
    }

    unsafe fn present(&mut self, p: Present) {
        if self.failed {
            return;
        }
        let hwnd = if p.hwnd != 0 { p.hwnd as HWND } else { find_window() };
        if hwnd.is_null() {
            return;
        }
        if let Err(e) = unsafe { self.open(hwnd) } {
            shim::log!("OpenGL: {e}; nothing will be drawn");
            self.failed = true;
            return;
        }
        let (cw, ch) = client(hwnd);
        let dev = self.frame.first().map_or((640, 480), |v| v.device);
        let (w, h) = if p.rgba.is_some() { (p.w as i32, p.h as i32) } else { dev };
        if cw <= 0 || ch <= 0 || w <= 0 || h <= 0 {
            return;
        }
        // The game's picture fitted to the window, kept in shape.
        let k = (cw as f32 / w as f32).min(ch as f32 / h as f32);
        let (vw, vh) = ((w as f32 * k) as i32, (h as f32 * k) as i32);
        let (ox, oy) = ((cw - vw) / 2, (ch - vh) / 2);
        unsafe {
            let g = self.gpu.as_ref().unwrap();
            let gl = &g.gl;
            (gl.BindFramebuffer)(FRAMEBUFFER, 0);
            (gl.Disable)(SCISSOR_TEST);
            (gl.Viewport)(0, 0, cw, ch);
            (gl.ClearColor)(0.0, 0.0, 0.0, 1.0);
            (gl.Clear)(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }
        // Deviation: a frame is shown under at most three of the game's pictures, so a menu
        // after a race does not show the race's last frame through marker-coloured pixels.
        if !self.frame.is_empty() && self.age < 3 {
            let s = super::scale().unwrap_or(k);
            let (fw, fh) = (((dev.0 as f32) * s).round() as i32, ((dev.1 as f32) * s).round() as i32);
            if unsafe { self.views(fw, fh) } {
                let g = self.gpu.as_ref().unwrap();
                let (dw, dh) = ((dev.0 as f32 * k) as i32, (dev.1 as f32 * k) as i32);
                let fbo = g.fbo.as_ref().unwrap();
                unsafe {
                    (g.gl.BindFramebuffer)(READ_FRAMEBUFFER, fbo.id);
                    (g.gl.BindFramebuffer)(DRAW_FRAMEBUFFER, 0);
                    (g.gl.BlitFramebuffer)(0, 0, fw, fh, ox, ch - oy - dh, ox + dw, ch - oy, COLOR_BUFFER_BIT, LINEAR as u32);
                    (g.gl.BindFramebuffer)(FRAMEBUFFER, 0);
                }
            }
        }
        self.age += 1;
        if let Some(rgba) = &p.rgba {
            unsafe { self.overlay(rgba, p.w, p.h, (ox, ch - oy - vh, vw, vh), (cw, ch)) };
        }
        unsafe {
            let g = self.gpu.as_ref().unwrap();
            self.pictures += 1;
            if let Some((dir, every)) = shots() {
                if self.pictures % every == 0 {
                    shot(&g.gl, cw, ch, &format!("{dir}\\sgl-{:06}.bmp", self.pictures));
                }
            }
            super::gl::check(&g.gl, "in the frame");
            SwapBuffers(g.dc);
        }
        self.shown += 1;
        let t = self.since.elapsed().as_secs_f32();
        if t >= 10.0 {
            shim::log!("{:.1} pictures a second, {} x {} window", self.shown as f32 / t, cw, ch);
            self.shown = 0;
            self.since = std::time::Instant::now();
        }
    }

    /// Make (or resize) the off-screen buffer.
    unsafe fn fbo(&mut self, w: i32, h: i32) -> bool {
        let g = self.gpu.as_mut().unwrap();
        let gl = &g.gl;
        if g.fbo.as_ref().is_some_and(|f| (f.w, f.h) == (w, h)) {
            return true;
        }
        unsafe {
            if let Some(f) = g.fbo.take() {
                (gl.DeleteTextures)(1, &f.color);
                (gl.DeleteTextures)(1, &f.depth);
            }
            let mut f = Fbo { id: 0, color: 0, depth: 0, light: 0, w, h };
            (gl.GenFramebuffers)(1, &mut f.id);
            (gl.GenFramebuffers)(1, &mut f.light);
            (gl.GenTextures)(1, &mut f.color);
            (gl.GenTextures)(1, &mut f.depth);
            (gl.BindTexture)(TEXTURE_2D, f.color);
            (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, w, h, 0, RGBA, UNSIGNED_BYTE, std::ptr::null());
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR);
            (gl.BindTexture)(TEXTURE_2D, f.depth);
            (gl.TexImage2D)(TEXTURE_2D, 0, DEPTH_COMPONENT24 as i32, w, h, 0, DEPTH_COMPONENT, UNSIGNED_INT, std::ptr::null());
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST);
            (gl.BindFramebuffer)(FRAMEBUFFER, f.light);
            (gl.FramebufferTexture2D)(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, f.color, 0);
            (gl.BindFramebuffer)(FRAMEBUFFER, f.id);
            (gl.FramebufferTexture2D)(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, f.color, 0);
            (gl.FramebufferTexture2D)(FRAMEBUFFER, DEPTH_ATTACHMENT, TEXTURE_2D, f.depth, 0);
            let st = (gl.CheckFramebufferStatus)(FRAMEBUFFER);
            if st != FRAMEBUFFER_COMPLETE {
                shim::log!("off-screen buffer {w}x{h} incomplete ({st:#x})");
                (gl.BindFramebuffer)(FRAMEBUFFER, 0);
                return false;
            }
            shim::log!("3D frame {w}x{h}");
            g.fbo = Some(f);
        }
        true
    }

    /// Upload a texture the first time it is drawn.
    unsafe fn texture(&mut self, id: u32) -> u32 {
        let Some(g) = &self.gpu else { return 0 };
        let Some((t, gl_id)) = self.textures.get_mut(&id) else { return 0 };
        if *gl_id == 0 && !t.rgba.is_empty() {
            let gl = &g.gl;
            unsafe {
                (gl.GenTextures)(1, gl_id);
                (gl.BindTexture)(TEXTURE_2D, *gl_id);
                (gl.PixelStorei)(UNPACK_ALIGNMENT, 1);
                (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, t.w as i32, t.h as i32, 0, RGBA, UNSIGNED_BYTE, t.rgba.as_ptr().cast());
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT);
            }
        }
        *gl_id
    }

    /// Draw the frame's views into the off-screen buffer.
    unsafe fn views(&mut self, fw: i32, fh: i32) -> bool {
        if !unsafe { self.fbo(fw, fh) } {
            return false;
        }
        let frame = std::mem::take(&mut self.frame);
        for vd in &frame {
            unsafe { self.view(vd, fw, fh) };
        }
        self.frame = frame;
        true
    }

    unsafe fn view(&mut self, vd: &ViewDraw, fw: i32, fh: i32) {
        let (sx, sy) = (fw as f32 / vd.device.0 as f32, fh as f32 / vd.device.1 as f32);
        let cam = &vd.view.camera;
        let [l, t, r, b] = vd.cam;
        let mut order: Vec<usize> = (0..vd.view.batches.len()).collect();
        // Opaque first, then the translucent passes in order.
        order.sort_by_key(|&i| (vd.view.batches[i].translucent, vd.view.batches[i].pass));
        let textures: Vec<u32> = order.iter().map(|&i| unsafe { self.texture(vd.view.batches[i].texture) }).collect();
        let g = self.gpu.as_ref().unwrap();
        let (gl, p) = (&g.gl, &g.programs.scene);
        unsafe {
            (gl.BindFramebuffer)(FRAMEBUFFER, g.fbo.as_ref().unwrap().id);
            (gl.Viewport)(0, 0, fw, fh);
            (gl.UseProgram)(p.id);
            (gl.Uniform4f)(p.loc("uCam"), (l + r) / 2.0, (t + b) / 2.0, cam.zoom * (r - l) / 2.0, cam.zoom * (b - t) / 2.0);
            (gl.Uniform2f)(p.loc("uDev"), vd.device.0 as f32, vd.device.1 as f32);
            (gl.Uniform1f)(p.loc("uNear"), cam.foreground.max(1e-3));
            (gl.Uniform1i)(p.loc("uTex"), 0);
            let fog = cam.fog.unwrap_or(([0.0; 3], 0.0));
            (gl.Uniform4f)(p.loc("uFog"), fog.0[0], fog.0[1], fog.0[2], fog_rate(fog.1) * fog_scale());
            (gl.ActiveTexture)(TEXTURE0);
            (gl.Enable)(SCISSOR_TEST);
            (gl.BindBuffer)(ARRAY_BUFFER, g.vbo);
            let stride = (SCENE_FLOATS * 4) as i32;
            for (i, (size, off)) in [(3, 0), (2, 3), (4, 5), (3, 9)].into_iter().enumerate() {
                (gl.EnableVertexAttribArray)(i as u32);
                (gl.VertexAttribPointer)(i as u32, size, FLOAT, 0, stride, (off * 4) as *const _);
            }
        }
        for rect in &vd.visible {
            let x0 = (rect[0] as f32 * sx).round() as i32;
            let x1 = ((rect[2] + 1) as f32 * sx).round() as i32;
            let y0 = ((rect[1]) as f32 * sy).round() as i32;
            let y1 = ((rect[3] + 1) as f32 * sy).round() as i32;
            let rect_px = (x0, fh - y1, x1 - x0, y1 - y0);
            unsafe {
                (gl.Scissor)(x0, fh - y1, x1 - x0, y1 - y0);
                let c = cam.background;
                (gl.ClearColor)(c[0], c[1], c[2], 1.0);
                (gl.ClearDepth)(1.0);
                (gl.DepthMask)(1);
                (gl.Clear)(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                (gl.Enable)(DEPTH_TEST);
                (gl.DepthFunc)(0x0203); // LEQUAL
            }
            let mut volumes_done = vd.view.volumes.is_empty();
            for (&bi, &tex) in order.iter().zip(&textures) {
                let b = &vd.view.batches[bi];
                if b.translucent && !volumes_done {
                    unsafe { self.volumes(vd, rect_px, (SCENE_FLOATS * 4) as i32) };
                    volumes_done = true;
                }
                let data = &mut self.scratch;
                data.clear();
                for v in &b.verts {
                    data.extend_from_slice(&[v.p[0], v.p[1], v.p[2], v.uv[0], v.uv[1], v.modulate[0], v.modulate[1], v.modulate[2], v.modulate[3], v.add[0], v.add[1], v.add[2]]);
                }
                unsafe {
                    if b.translucent {
                        (gl.Enable)(BLEND);
                        (gl.BlendFuncSeparate)(0x0302, 0x0303, 1, 0x0303); // SRC_ALPHA, ONE_MINUS_SRC_ALPHA
                        (gl.DepthMask)(0);
                    } else {
                        (gl.Disable)(BLEND);
                        (gl.DepthMask)(1);
                    }
                    if b.no_depth {
                        (gl.Disable)(DEPTH_TEST);
                        (gl.DepthMask)(0);
                    } else {
                        (gl.Enable)(DEPTH_TEST);
                    }
                    (gl.BindTexture)(TEXTURE_2D, tex);
                    (gl.Uniform1i)(p.loc("uTextured"), (tex != 0) as i32);
                    (gl.Uniform1i)(p.loc("uFogOn"), (b.fog && cam.fog.is_some()) as i32);
                    (gl.BufferData)(ARRAY_BUFFER, (data.len() * 4) as isize, data.as_ptr().cast(), STREAM_DRAW);
                    (gl.DrawArrays)(TRIANGLES, 0, b.verts.len() as i32);
                }
            }
            if !volumes_done {
                unsafe { self.volumes(vd, rect_px, (SCENE_FLOATS * 4) as i32) };
            }
        }
        unsafe {
            (gl.Disable)(SCISSOR_TEST);
            (gl.Disable)(DEPTH_TEST);
            (gl.Disable)(BLEND);
            (gl.DepthMask)(1);
            (gl.BindFramebuffer)(FRAMEBUFFER, 0);
        }
    }

    /// Light volumes (sgl.h: a light volume's colour is added wherever it meets a surface). Each
    /// pixel's surface, rebuilt from the depth, is tested against the volume's planes, and the
    /// colour added where it is inside all of them. Drawn into a frame buffer holding only the
    /// colour, so the depth texture can be read; then the scene's state is put back.
    unsafe fn volumes(&self, vd: &ViewDraw, (x, y, w, h): (i32, i32, i32, i32), stride: i32) {
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
            (gl.BlendFuncSeparate)(1, 1, 0, 1); // ONE, ONE
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
                if v.planes.is_empty() || v.colour == [0.0; 3] {
                    continue;
                }
                // Deviation: a light volume of more than MAX_PLANES planes keeps the first ones.
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

    /// The game's picture over the rectangle (x, y, w, h) of the window, in OpenGL's
    /// bottom-up pixels.
    unsafe fn overlay(&mut self, rgba: &[u8], w: u32, h: u32, (x, y, vw, vh): (i32, i32, i32, i32), (cw, ch): (i32, i32)) {
        let g = self.gpu.as_mut().unwrap();
        let gl = &g.gl;
        let p = &g.programs.overlay;
        unsafe {
            (gl.UseProgram)(p.id);
            (gl.ActiveTexture)(TEXTURE0);
            (gl.BindTexture)(TEXTURE_2D, g.overlay_tex);
            (gl.PixelStorei)(UNPACK_ALIGNMENT, 1);
            if g.overlay_size == (w, h) {
                (gl.TexSubImage2D)(TEXTURE_2D, 0, 0, 0, w as i32, h as i32, RGBA, UNSIGNED_BYTE, rgba.as_ptr().cast());
            } else {
                (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, w as i32, h as i32, 0, RGBA, UNSIGNED_BYTE, rgba.as_ptr().cast());
                g.overlay_size = (w, h);
            }
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE);
            (gl.Uniform1i)(p.loc("uImg"), 0);
            (gl.Viewport)(0, 0, cw, ch);
            (gl.Enable)(BLEND);
            (gl.BlendFuncSeparate)(0x0302, 0x0303, 1, 0x0303);
            let nx = |px: i32| px as f32 / cw as f32 * 2.0 - 1.0;
            let ny = |py: i32| py as f32 / ch as f32 * 2.0 - 1.0;
            let (x0, x1, y0, y1) = (nx(x), nx(x + vw), ny(y), ny(y + vh));
            // Two triangles; the picture's first row at the top.
            let quad: [f32; 24] = [x0, y0, 0.0, 1.0, x1, y0, 1.0, 1.0, x1, y1, 1.0, 0.0, x0, y0, 0.0, 1.0, x1, y1, 1.0, 0.0, x0, y1, 0.0, 0.0];
            (gl.BindBuffer)(ARRAY_BUFFER, g.vbo);
            (gl.BufferData)(ARRAY_BUFFER, (quad.len() * 4) as isize, quad.as_ptr().cast(), STREAM_DRAW);
            for i in 0..4 {
                (gl.EnableVertexAttribArray)(i);
            }
            (gl.VertexAttribPointer)(0, 2, FLOAT, 0, 16, std::ptr::null());
            (gl.VertexAttribPointer)(1, 2, FLOAT, 0, 16, 8 as *const _);
            // Attributes 2 and 3 belong to the scene program.
            (gl.DisableVertexAttribArray)(2);
            (gl.DisableVertexAttribArray)(3);
            (gl.DrawArrays)(TRIANGLES, 0, 6);
            (gl.Disable)(BLEND);
        }
    }
}

/// `SGL_SHOT_DIR` (a Windows folder) and `SGL_SHOT_EVERY` (default 100): save every Nth
/// picture shown there as a BMP, for checks and the diary.
fn shots() -> Option<(String, u32)> {
    static S: std::sync::OnceLock<Option<(String, u32)>> = std::sync::OnceLock::new();
    S.get_or_init(|| {
        let dir = std::env::var("SGL_SHOT_DIR").ok().filter(|d| !d.is_empty())?;
        let every = std::env::var("SGL_SHOT_EVERY").ok().and_then(|v| v.parse().ok()).filter(|n| *n > 0).unwrap_or(100);
        Some((dir, every))
    })
    .clone()
}

/// The window's back buffer as a 24-bit BMP (OpenGL's rows are bottom-up, as a BMP's are).
unsafe fn shot(gl: &Gl, w: i32, h: i32, path: &str) {
    let mut px = vec![0u8; (w * h * 4) as usize];
    unsafe {
        (gl.PixelStorei)(PACK_ALIGNMENT, 1);
        (gl.ReadPixels)(0, 0, w, h, RGBA, UNSIGNED_BYTE, px.as_mut_ptr().cast());
    }
    let row = ((w * 3 + 3) & !3) as usize;
    let size = 54 + row * h as usize;
    let mut f = Vec::with_capacity(size);
    f.extend_from_slice(b"BM");
    for v in [size as u32, 0, 54, 40, w as u32, h as u32] {
        f.extend_from_slice(&v.to_le_bytes());
    }
    f.extend_from_slice(&1u16.to_le_bytes());
    f.extend_from_slice(&24u16.to_le_bytes());
    f.extend_from_slice(&[0u8; 24]);
    for y in 0..h as usize {
        let start = f.len();
        for x in 0..w as usize {
            let p = &px[(y * w as usize + x) * 4..];
            f.extend_from_slice(&[p[2], p[1], p[0]]);
        }
        f.resize(start + row, 0);
    }
    match std::fs::write(path, f) {
        Ok(()) => shim::log!("saved {path}"),
        Err(e) => shim::log!("cannot save {path}: {e}"),
    }
}

/// The fog's rate per unit of depth from SGL's density, as PowerVR's driver works it out
/// (SGLMID4.dll `sgl_set_fog`: `-log2(1 - density)`, folded into the chip's depth scale with
/// the camera's foreground distance, `FUN_10006bf0`): a surface keeps `(1 - density)` of its
/// colour per unit (*observed*; that the chip applies it as `2^(-rate z)` is *inferred*).
/// Here the shader's `exp`, so the rate in natural logarithms.
fn fog_rate(density: f32) -> f32 {
    let keep = (1.0 - density).clamp(0.0, 1.0);
    if keep <= 1e-20 { 1000.0 } else { -keep.ln() }
}

/// `SGL_FOG`: a factor on the fog rate, for comparing pictures (default 1; the
/// rate itself comes from PowerVR's driver).
fn fog_scale() -> f32 {
    static S: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *S.get_or_init(|| std::env::var("SGL_FOG").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0))
}

unsafe extern "system" fn find_proc(w: HWND, out: isize) -> BOOL {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(w, &mut pid) };
    if pid != unsafe { GetCurrentProcessId() } {
        return 1;
    }
    let mut name = [0u8; 64];
    let n = unsafe { GetClassNameA(w, name.as_mut_ptr(), 64) }.max(0) as usize;
    let game = match shim::config::window_class() {
        Some(class) => &name[..n] == class,
        None => (unsafe { IsWindowVisible(w) }) != 0,
    };
    if game {
        unsafe { *(out as *mut HWND) = w };
        return 0;
    }
    1
}

/// The game's window: of class `WRAPPER_WINDOW_CLASS`, or without it the first visible
/// top-level window of the process.
fn find_window() -> HWND {
    let mut found: HWND = null_mut();
    unsafe { EnumWindows(find_proc, &mut found as *mut _ as isize) };
    found
}

fn client(hwnd: HWND) -> (i32, i32) {
    let mut r = RECT::default();
    unsafe { GetClientRect(hwnd, &mut r) };
    (r.right - r.left, r.bottom - r.top)
}

/// Give a window's device context the pixel format OpenGL draws with.
fn pixel_format(hwnd: HWND) -> Result<HDC, String> {
    unsafe {
        let dc = GetDC(hwnd);
        let pfd = PIXELFORMATDESCRIPTOR {
            size: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            version: 1,
            flags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            color_bits: 24,
            alpha_bits: 8,
            depth_bits: 24,
            ..Default::default()
        };
        let f = ChoosePixelFormat(dc, &pfd);
        if f == 0 || SetPixelFormat(dc, f, &pfd) == 0 {
            return Err(format!("no pixel format (ChoosePixelFormat {f})"));
        }
        Ok(dc)
    }
}
