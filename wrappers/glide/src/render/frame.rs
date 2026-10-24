//! The Voodoo's frame buffer at the output size: two colour buffers (front and back, swapped
//! by `grBufferSwap`) sharing one depth buffer, as Glide's double buffering does; the copy to
//! the window; and reads back at the game's own resolution.

use crate::gl::*;
use crate::render::shaders::Program;
use std::ptr::null;

pub struct Frame {
    /// The game's resolution, and the output (drawing) size.
    pub w: u32,
    pub h: u32,
    pub ow: u32,
    pub oh: u32,
    fbo: [u32; 2],
    color: [u32; 2],
    depth: u32,
    /// Index of the back buffer.
    pub back: usize,
    // A buffer at the game's resolution, for reads: depth and colour are copied down into it.
    small_fbo: u32,
    small_color: u32,
    small_depth: u32,
}

unsafe fn color_texture(gl: &Gl, w: u32, h: u32) -> u32 {
    unsafe {
        let mut t = 0;
        (gl.GenTextures)(1, &mut t);
        (gl.BindTexture)(TEXTURE_2D, t);
        (gl.TexImage2D)(TEXTURE_2D, 0, RGBA8 as i32, w as i32, h as i32, 0, RGBA, UNSIGNED_BYTE, null());
        (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR);
        (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR);
        (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE);
        (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE);
        t
    }
}

unsafe fn depth_buffer(gl: &Gl, w: u32, h: u32) -> u32 {
    unsafe {
        let mut r = 0;
        (gl.GenRenderbuffers)(1, &mut r);
        (gl.BindRenderbuffer)(RENDERBUFFER, r);
        (gl.RenderbufferStorage)(RENDERBUFFER, DEPTH_COMPONENT24, w as i32, h as i32);
        r
    }
}

unsafe fn framebuffer(gl: &Gl, color: u32, depth: u32) -> Result<u32, String> {
    unsafe {
        let mut f = 0;
        (gl.GenFramebuffers)(1, &mut f);
        (gl.BindFramebuffer)(FRAMEBUFFER, f);
        (gl.FramebufferTexture2D)(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, color, 0);
        (gl.FramebufferRenderbuffer)(FRAMEBUFFER, DEPTH_ATTACHMENT, RENDERBUFFER, depth);
        let s = (gl.CheckFramebufferStatus)(FRAMEBUFFER);
        if s != FRAMEBUFFER_COMPLETE {
            return Err(format!("frame buffer incomplete: {s:#x}"));
        }
        Ok(f)
    }
}

impl Frame {
    pub unsafe fn new(gl: &Gl, w: u32, h: u32, ow: u32, oh: u32) -> Result<Frame, String> {
        unsafe {
            let depth = depth_buffer(gl, ow, oh);
            let color = [color_texture(gl, ow, oh), color_texture(gl, ow, oh)];
            let fbo = [framebuffer(gl, color[0], depth)?, framebuffer(gl, color[1], depth)?];
            let small_depth = depth_buffer(gl, w, h);
            let small_color = color_texture(gl, w, h);
            let small_fbo = framebuffer(gl, small_color, small_depth)?;
            let f = Frame { w, h, ow, oh, fbo, color, depth, back: 0, small_fbo, small_color, small_depth };
            f.bind_back(gl);
            Ok(f)
        }
    }

    pub unsafe fn delete(&self, gl: &Gl) {
        // Renderbuffers and frame buffers are few and small; textures hold the memory.
        unsafe { (gl.DeleteTextures)(3, [self.color[0], self.color[1], self.small_color].as_ptr()) };
        let _ = (self.depth, self.small_depth);
    }

    pub unsafe fn bind_back(&self, gl: &Gl) {
        unsafe {
            (gl.BindFramebuffer)(FRAMEBUFFER, self.fbo[self.back]);
            (gl.Viewport)(0, 0, self.ow as i32, self.oh as i32);
        }
    }

    /// Draw the back buffer into the window (`cw × ch`, the picture kept 4:3 and centred) with
    /// `gamma`, show it, and swap the buffers.
    pub unsafe fn present(&mut self, gl: &Gl, p: &Program, gamma: f32, cw: i32, ch: i32, dc: crate::win32::HDC) {
        unsafe {
            (gl.BindFramebuffer)(FRAMEBUFFER, 0);
            (gl.Disable)(SCISSOR_TEST);
            (gl.Disable)(DEPTH_TEST);
            (gl.Disable)(BLEND);
            (gl.ClearColor)(0.0, 0.0, 0.0, 1.0);
            (gl.Viewport)(0, 0, cw, ch);
            let scale = (cw as f32 / self.ow as f32).min(ch as f32 / self.oh as f32);
            let (vw, vh) = ((self.ow as f32 * scale) as i32, (self.oh as f32 * scale) as i32);
            let (x0, y0) = ((cw - vw) / 2, (ch - vh) / 2);
            if (vw, vh) != (cw, ch) {
                (gl.Clear)(COLOR_BUFFER_BIT);
            }
            if gamma == 1.0 || gamma <= 0.0 {
                // No gamma: a plain copy, cheaper than a shader pass in software OpenGL.
                (gl.BindFramebuffer)(READ_FRAMEBUFFER, self.fbo[self.back]);
                let filter = if (vw, vh) == (self.ow as i32, self.oh as i32) { NEAREST } else { LINEAR };
                (gl.BlitFramebuffer)(0, 0, self.ow as i32, self.oh as i32, x0, y0, x0 + vw, y0 + vh, COLOR_BUFFER_BIT, filter as u32);
            } else {
                (gl.Viewport)(x0, y0, vw, vh);
                (gl.UseProgram)(p.id);
                (gl.ActiveTexture)(TEXTURE0);
                (gl.BindTexture)(TEXTURE_2D, self.color[self.back]);
                (gl.Uniform1i)(p.loc("uImg"), 0);
                (gl.Uniform1f)(p.loc("uGamma"), gamma);
                (gl.Uniform4f)(p.loc("uRect"), 0.0, 0.0, 1.0, 1.0);
                (gl.Uniform2f)(p.loc("uScreen"), 1.0, 1.0);
                (gl.DrawArrays)(0x0005, 0, 4);
            }
            crate::gl::check(gl, "in the frame");
            crate::win32::SwapBuffers(dc);
            self.back ^= 1;
            self.bind_back(gl);
        }
    }

    /// Copy one buffer down to the game's resolution and read it: 16-bit values, 1024 per
    /// row, rows top to bottom. Depth reads give Glide's values (0xffff where nothing was
    /// drawn since the clear); colour reads give RGB565.
    pub unsafe fn read(&self, gl: &Gl, buffer: u32, wbuffer: bool) -> Vec<u16> {
        use crate::consts::*;
        let (w, h) = (self.w as usize, self.h as usize);
        let mut out = vec![0u16; 1024 * h];
        unsafe {
            let src = if buffer == BUFFER_FRONT { self.fbo[self.back ^ 1] } else { self.fbo[self.back] };
            let depth = buffer == BUFFER_DEPTH || buffer == BUFFER_AUX;
            (gl.BindFramebuffer)(READ_FRAMEBUFFER, src);
            (gl.BindFramebuffer)(DRAW_FRAMEBUFFER, self.small_fbo);
            (gl.Disable)(SCISSOR_TEST);
            let mask = if depth { DEPTH_BUFFER_BIT } else { COLOR_BUFFER_BIT };
            (gl.BlitFramebuffer)(0, 0, self.ow as i32, self.oh as i32, 0, 0, w as i32, h as i32, mask, NEAREST as u32);
            (gl.BindFramebuffer)(FRAMEBUFFER, self.small_fbo);
            (gl.PixelStorei)(PACK_ALIGNMENT, 1);
            if depth {
                let mut d = vec![0f32; w * h];
                (gl.ReadPixels)(0, 0, w as i32, h as i32, DEPTH_COMPONENT, FLOAT, d.as_mut_ptr().cast());
                for y in 0..h {
                    for x in 0..w {
                        out[y * 1024 + x] = glide_depth(d[(h - 1 - y) * w + x], wbuffer);
                    }
                }
            } else {
                let mut c = vec![0u8; w * h * 4];
                (gl.ReadPixels)(0, 0, w as i32, h as i32, RGBA, UNSIGNED_BYTE, c.as_mut_ptr().cast());
                for y in 0..h {
                    for x in 0..w {
                        let p = &c[((h - 1 - y) * w + x) * 4..];
                        out[y * 1024 + x] = (p[0] as u16 >> 3) << 11 | (p[1] as u16 >> 2) << 5 | p[2] as u16 >> 3;
                    }
                }
            }
            self.bind_back(gl);
        }
        out
    }
}

/// An OpenGL depth (0–1, as the scene shader writes it) as the Voodoo's 16-bit value: in the
/// Z buffer `ooz`; in the W buffer the 4.12 floating point `w` (exponent in the top 4 bits).
/// The cleared value 1.0 is 0xffff in both.
fn glide_depth(d: f32, wbuffer: bool) -> u16 {
    if d >= 1.0 {
        return 0xffff;
    }
    if !wbuffer {
        return (d * 65535.0).round() as u16;
    }
    let w = 1.0 / (1.0 - d).max(1.0 / 65536.0);
    let e = (w.log2().floor() as i32).clamp(0, 15);
    let m = ((w / (1 << e) as f32 - 1.0) * 4096.0).clamp(0.0, 4095.0) as u16;
    ((e as u16) << 12 | m).min(0xfffe)
}
