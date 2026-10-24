//! The render thread: owns the OpenGL context and replays the recorded Glide commands in
//! order. Triangles and lines are gathered while the state stays the same and drawn together.

mod frame;
mod shaders;
mod state;
mod textures;

use crate::cmd::{Cmd, LfbImage, Vertex};
use crate::consts::DEPTHBUFFER_WBUFFER;
use crate::gl::*;
use crate::log::log;
use crate::win32::{HDC, HWND};
use frame::Frame;
use shaders::{FLOATS, Programs};
use state::State;
use std::sync::mpsc::Receiver;
use textures::Textures;

struct Gpu {
    gl: Gl,
    programs: Programs,
    dc: HDC,
    hwnd: HWND,
    frame: Frame,
    vbo: u32,
    lfb_tex: u32,
    /// The LFB texture's size, to update it in place while it stays the same.
    lfb_size: (u32, u32),
}

struct Renderer {
    gpu: Option<Gpu>,
    state: State,
    textures: Textures,
    dirty: bool,
    batch: Vec<f32>,
    prim: u32,
    failed: bool,
    frames: u32,
    since: std::time::Instant,
    /// Time spent replaying since `since`, against the time waiting for the game.
    busy: std::time::Duration,
    /// Time per kind of command since `since`: draws, LFB pictures, clears, swaps, the rest.
    spent: [std::time::Duration; 5],
}

pub fn run(commands: Receiver<Vec<Cmd>>) {
    let mut r = Renderer {
        gpu: None,
        state: State::new(640, 480),
        textures: Textures::new(),
        dirty: true,
        batch: Vec::with_capacity(1 << 16),
        prim: TRIANGLES,
        failed: false,
        frames: 0,
        since: std::time::Instant::now(),
        busy: std::time::Duration::ZERO,
        spent: [std::time::Duration::ZERO; 5],
    };
    for batch in commands {
        let t = std::time::Instant::now();
        for c in batch {
            let kind = match c {
                Cmd::Triangle(_) | Cmd::Line(_) | Cmd::Point(_) => 0,
                Cmd::Lfb(_) => 1,
                Cmd::Clear { .. } => 2,
                Cmd::Swap => 3,
                _ => 4,
            };
            let t = std::time::Instant::now();
            unsafe { r.exec(c) };
            r.spent[kind] += t.elapsed();
        }
        r.busy += t.elapsed();
    }
}

impl Renderer {
    unsafe fn open(&mut self, w: u32, h: u32, (ow, oh): (u32, u32), hwnd: HWND) -> Result<(), String> {
        unsafe {
            if let Some(g) = &mut self.gpu {
                if g.hwnd != hwnd {
                    // Another window (the game's, once it exists): the same context on it.
                    let dc = crate::window::pixel_format(hwnd)?;
                    if crate::win32::wglMakeCurrent(dc, crate::win32::wglGetCurrentContext()) == 0 {
                        return Err("cannot move the context to the new window".into());
                    }
                    (g.dc, g.hwnd) = (dc, hwnd);
                }
                if (g.frame.w, g.frame.h, g.frame.ow, g.frame.oh) != (w, h, ow, oh) {
                    g.frame.delete(&g.gl);
                    g.frame = Frame::new(&g.gl, w, h, ow, oh)?;
                }
                return Ok(());
            }
            let dc = crate::window::context(hwnd)?;
            let gl = Gl::load()?;
            let s = |n| std::ffi::CStr::from_ptr((gl.GetString)(n).cast()).to_string_lossy().into_owned();
            log!("OpenGL {} on {}; drawing {w}x{h} at {ow}x{oh}", s(VERSION), s(RENDERER));
            let programs = shaders::build(&gl)?;
            let frame = Frame::new(&gl, w, h, ow, oh)?;
            let mut ids = [0u32; 2];
            (gl.GenBuffers)(1, &mut ids[0]);
            (gl.GenTextures)(1, &mut ids[1]);
            // A core context draws nothing without a vertex array object bound.
            if crate::window::CORE.load(std::sync::atomic::Ordering::Relaxed) {
                crate::gl::bind_vertex_array();
            }
            (gl.Enable)(DEPTH_CLAMP);
            (gl.Disable)(CULL_FACE);
            self.gpu = Some(Gpu { gl, programs, dc, hwnd, frame, vbo: ids[0], lfb_tex: ids[1], lfb_size: (0, 0) });
        }
        Ok(())
    }

    /// Draw what is gathered.
    unsafe fn draw(&mut self) {
        let n = self.batch.len() / FLOATS;
        let Some(g) = &self.gpu else {
            self.batch.clear();
            return;
        };
        if n == 0 {
            return;
        }
        unsafe {
            let gl = &g.gl;
            (gl.BindBuffer)(ARRAY_BUFFER, g.vbo);
            (gl.BufferData)(ARRAY_BUFFER, (self.batch.len() * 4) as isize, self.batch.as_ptr().cast(), STREAM_DRAW);
            let stride = (FLOATS * 4) as i32;
            for (i, (size, off)) in [(4, 0), (4, 4), (2, 8)].into_iter().enumerate() {
                (gl.EnableVertexAttribArray)(i as u32);
                (gl.VertexAttribPointer)(i as u32, size, FLOAT, 0, stride, (off * 4) as *const _);
            }
            let scale = (g.frame.ow as f32 / g.frame.w as f32).max(1.0);
            // A core context takes no width above 1.
            if self.prim == LINES && !crate::window::CORE.load(std::sync::atomic::Ordering::Relaxed) {
                (gl.LineWidth)(scale);
            }
            if self.prim == POINTS {
                (gl.PointSize)(scale);
            }
            (gl.DrawArrays)(self.prim, 0, n as i32);
        }
        self.batch.clear();
    }

    /// A state change: draw what was gathered under the old state first.
    unsafe fn change(&mut self) {
        unsafe { self.draw() };
        self.dirty = true;
    }

    unsafe fn vertices(&mut self, prim: u32, vs: &[Vertex]) {
        if self.gpu.is_none() {
            return;
        }
        unsafe {
            if self.prim != prim {
                self.draw();
                self.prim = prim;
            }
            if self.dirty {
                self.draw();
                let g = self.gpu.as_ref().unwrap();
                let f = &g.frame;
                state::apply(&g.gl, &g.programs.scene, &self.state, &mut self.textures, f.w, f.h, f.ow, f.oh);
                self.dirty = false;
            }
        }
        for v in vs {
            let t = v.tmu[0];
            self.batch.extend_from_slice(&[v.x, v.y, v.ooz, v.oow, v.r, v.g, v.b, v.a, t[0], t[1]]);
        }
    }

    unsafe fn lfb(&mut self, img: &LfbImage) {
        let Some(g) = &mut self.gpu else { return };
        let same = g.lfb_size == (img.w, img.h);
        g.lfb_size = (img.w, img.h);
        let (gl, p) = (&g.gl, &g.programs.lfb);
        let filter = if crate::config::get().lfb_linear { LINEAR } else { NEAREST };
        unsafe {
            (gl.UseProgram)(p.id);
            (gl.ActiveTexture)(TEXTURE0);
            (gl.BindTexture)(TEXTURE_2D, g.lfb_tex);
            (gl.PixelStorei)(UNPACK_ALIGNMENT, 1);
            if same {
                (gl.TexSubImage2D)(TEXTURE_2D, 0, 0, 0, img.w as i32, img.h as i32, RGBA, UNSIGNED_BYTE, img.rgba.as_ptr().cast());
            } else {
                (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, img.w as i32, img.h as i32, 0, RGBA, UNSIGNED_BYTE, img.rgba.as_ptr().cast());
            }
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, filter);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, filter);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE);
            (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE);
            (gl.Uniform1i)(p.loc("uImg"), 0);
            (gl.Uniform2f)(p.loc("uScreen"), g.frame.w as f32, g.frame.h as f32);
            let (x, y) = (img.x as f32, img.y as f32);
            (gl.Uniform4f)(p.loc("uRect"), x, y, x + img.w as f32, y + img.h as f32);
            // Deviation: LFB writes through the pixel pipeline are blended and chroma-keyed,
            // but neither depth-tested nor fogged.
            (gl.Uniform1i)(p.loc("uChroma"), (img.pipeline && self.state.chroma) as i32);
            let k = self.state.key;
            (gl.Uniform3f)(p.loc("uKey"), ((k >> 16) & 0xff) as f32 / 255.0, ((k >> 8) & 0xff) as f32 / 255.0, (k & 0xff) as f32 / 255.0);
            if img.pipeline {
                state::blend(gl, self.state.blend);
            } else {
                (gl.Disable)(BLEND);
            }
            (gl.Disable)(SCISSOR_TEST);
            (gl.Disable)(DEPTH_TEST);
            (gl.DepthMask)(0);
            (gl.ColorMask)(1, 1, 1, 1);
            (gl.DrawArrays)(0x0005, 0, 4);
        }
        self.dirty = true;
    }

    /// Draw into the front buffer or the back one. Going back to the back buffer shows what
    /// was drawn into the front one, as the Voodoo showed it while it was drawn.
    unsafe fn render_buffer(&mut self, front: bool) {
        let Some(g) = &mut self.gpu else { return };
        unsafe {
            if g.frame.front && !front {
                let (cw, ch) = crate::window::client(g.hwnd);
                let (cw, ch) = if cw > 0 && ch > 0 { (cw, ch) } else { (g.frame.ow as i32, g.frame.oh as i32) };
                g.frame.show(&g.gl, &g.programs.present, self.state.gamma, cw, ch, g.dc, g.frame.back ^ 1);
            }
            g.frame.front = front;
            g.frame.bind_back(&g.gl);
        }
        self.dirty = true;
    }

    unsafe fn clear(&mut self, color: u32, alpha: u32, depth: u32) {
        let Some(g) = &self.gpu else { return };
        let (gl, f, s) = (&g.gl, &g.frame, &self.state);
        unsafe {
            state::scissor(gl, s.clip, f.w, f.h, f.ow, f.oh);
            let c = |sh: u32| ((color >> sh) & 0xff) as f32 / 255.0;
            (gl.ClearColor)(c(16), c(8), c(0), alpha as f32 / 255.0);
            (gl.ClearDepth)(depth as f64 / 65535.0);
            (gl.DepthMask)(s.depth_mask as u8);
            let (rgb, a) = (s.color_mask.0 as u8, s.color_mask.1 as u8);
            (gl.ColorMask)(rgb, rgb, rgb, a);
            (gl.Clear)(COLOR_BUFFER_BIT | if s.depth_mask { DEPTH_BUFFER_BIT } else { 0 });
        }
        self.dirty = true;
    }

    unsafe fn exec(&mut self, c: Cmd) {
        use Cmd::*;
        if !matches!(c, Triangle(_) | Line(_) | Point(_)) {
            unsafe { self.change() };
        }
        let s2 = &mut self.state;
        match c {
            Open { width, height, out, hwnd } => {
                if self.failed {
                    return;
                }
                if let Err(e) = unsafe { self.open(width, height, out, hwnd as HWND) } {
                    log!("cannot draw: {e}");
                    self.failed = true;
                }
                self.state.clip = [0, 0, width, height];
            }
            ColorCombine(v) => s2.cc = v,
            AlphaCombine(v) => s2.ac = v,
            ConstantColor(v) => s2.constant = v,
            AlphaBlend(v) => s2.blend = v,
            DepthMode(v) => s2.depth_mode = v,
            DepthFunc(v) => s2.depth_func = v,
            DepthMask(v) => s2.depth_mask = v,
            ChromaMode(v) => s2.chroma = v,
            ChromaValue(v) => s2.key = v,
            FogMode(v) => s2.fog_mode = v,
            FogColor(v) => s2.fog_color = v,
            FogTable(t) => s2.fog_table = *t,
            Gamma(v) => s2.gamma = v,
            Clip(v) => s2.clip = v,
            TexCombine(v) => s2.tex_fn = v,
            TexFilter(a, b) => s2.filter = (a, b),
            TexClamp(a, b) => s2.clamp = (a, b),
            TexSource(src) => s2.source = Some(src),
            Download { start, data } => self.textures.download(start, &data),
            Palette { first, words } => self.textures.palette(first, &words),
            Ncc(which, table) => self.textures.ncc(which, &*table),
            MipMap(mode, blend) => s2.mipmap = (mode, blend),
            LodBias(v) => s2.lod_bias = v,
            AlphaTest(v) => s2.alpha_test = v,
            ColorMask(rgb, a) => s2.color_mask = (rgb, a),
            DepthBias(v) => s2.depth_bias = v,
            RenderBuffer(b) => unsafe { self.render_buffer(b == crate::consts::BUFFER_FRONT) },
            Triangle(v) => unsafe { self.vertices(TRIANGLES, &v) },
            Line(v) => unsafe { self.vertices(LINES, &v) },
            Point(v) => unsafe { self.vertices(POINTS, &[v]) },
            Clear { color, alpha, depth } => unsafe { self.clear(color, alpha, depth) },
            Lfb(img) => unsafe { self.lfb(&img) },
            Swap => {
                if let Some(g) = &mut self.gpu {
                    let (cw, ch) = crate::window::client(g.hwnd);
                    let (cw, ch) = if cw > 0 && ch > 0 { (cw, ch) } else { (g.frame.ow as i32, g.frame.oh as i32) };
                    unsafe { g.frame.present(&g.gl, &g.programs.present, self.state.gamma, cw, ch, g.dc) };
                }
                self.frames += 1;
                if self.frames % 100 == 0 {
                    let all = self.since.elapsed().as_secs_f32();
                    let ms = self.spent.map(|d| (d.as_secs_f32() * 10.0) as u32);
                    log!(
                        "{} frames, {:.1} per second, drawing {:.0}% of the time; ms per frame: triangles {}, lfb {}, clear {}, swap {}, other {}",
                        self.frames, 100.0 / all, 100.0 * self.busy.as_secs_f32() / all, ms[0], ms[1], ms[2], ms[3], ms[4]
                    );
                    self.spent = [std::time::Duration::ZERO; 5];
                    self.since = std::time::Instant::now();
                    self.busy = std::time::Duration::ZERO;
                }
            }
            Read(r) => {
                let data = match &self.gpu {
                    Some(g) => unsafe { g.frame.read(&g.gl, r.buffer, self.state.depth_mode == DEPTHBUFFER_WBUFFER) },
                    None => vec![0xffff; 1024 * 480],
                };
                let _ = r.reply.send(data);
            }
        }
    }
}
