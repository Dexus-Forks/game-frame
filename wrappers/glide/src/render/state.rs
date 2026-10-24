//! The Glide render state as the render thread keeps it, and its translation to OpenGL state
//! and the scene shader's uniforms.

use crate::cmd::Source;
use crate::consts::*;
use crate::gl::*;
use crate::render::shaders::Program;
use crate::render::textures::Textures;

pub struct State {
    pub cc: [u32; 5],
    pub ac: [u32; 5],
    pub constant: u32,
    pub blend: [u32; 4],
    pub depth_mode: u32,
    pub depth_func: u32,
    pub depth_mask: bool,
    pub chroma: bool,
    pub key: u32,
    pub fog_mode: u32,
    pub fog_color: u32,
    pub fog_table: [u8; 64],
    pub gamma: f32,
    pub clip: [u32; 4],
    pub tex_fn: u32,
    pub filter: (u32, u32),
    pub clamp: (u32, u32),
    pub source: Option<Source>,
    /// `GrCmpFnc_t` and the reference value.
    pub alpha_test: [u32; 2],
    pub color_mask: (bool, bool),
    pub depth_bias: i32,
    /// `grTexMipMapMode`: the mode and whether to blend between levels.
    pub mipmap: (u32, bool),
    pub lod_bias: f32,
}

impl State {
    /// Glide's state after `grSstOpen`, as far as POD does not set it itself.
    pub fn new(w: u32, h: u32) -> State {
        State {
            cc: [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
            ac: [FUNC_LOCAL, FACTOR_ZERO, LOCAL_ITERATED, OTHER_CONSTANT, 0],
            constant: 0xffff_ffff,
            blend: [BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO],
            depth_mode: DEPTHBUFFER_DISABLE,
            depth_func: 1,
            depth_mask: false,
            chroma: false,
            key: 0,
            fog_mode: 0,
            fog_color: 0,
            fog_table: [0; 64],
            gamma: 1.0,
            clip: [0, 0, w, h],
            tex_fn: 1,
            filter: (FILTER_BILINEAR, FILTER_BILINEAR),
            clamp: (0, 0),
            source: None,
            alpha_test: [7, 0],
            color_mask: (true, true),
            depth_bias: 0,
            mipmap: (MIPMAP_DISABLE, false),
            lod_bias: 0.0,
        }
    }

    fn uses_texture(&self) -> bool {
        let tex_factor = |c: &[u32; 5]| c[1] == FACTOR_TEXTURE_ALPHA || c[1] == 0xc;
        self.cc[3] == OTHER_TEXTURE || self.ac[3] == OTHER_TEXTURE || tex_factor(&self.cc) || tex_factor(&self.ac)
    }
}

fn rgb(c: u32) -> [f32; 3] {
    [((c >> 16) & 0xff) as f32 / 255.0, ((c >> 8) & 0xff) as f32 / 255.0, (c & 0xff) as f32 / 255.0]
}

/// `GR_BLEND_*` to OpenGL. Source and destination differ at 2, 6 and 15.
fn blend_factor(f: u32, dst: bool) -> u32 {
    match f {
        BLEND_ZERO => 0,
        BLEND_SRC_ALPHA => 0x302,
        BLEND_SRC_COLOR => if dst { 0x300 } else { 0x306 },
        BLEND_DST_ALPHA => 0x304,
        BLEND_ONE => 1,
        BLEND_ONE_MINUS_SRC_ALPHA => 0x303,
        BLEND_ONE_MINUS_SRC_COLOR => if dst { 0x301 } else { 0x307 },
        BLEND_ONE_MINUS_DST_ALPHA => 0x305,
        // Deviation: GR_BLEND_PREFOG_COLOR (destination) has no OpenGL factor; zero.
        BLEND_ALPHA_SATURATE => if dst { 0 } else { 0x308 },
        _ => 1,
    }
}

/// Blending as Glide's `grAlphaBlendFunction` sets it.
pub unsafe fn blend(gl: &Gl, b: [u32; 4]) {
    unsafe {
        if b == [BLEND_ONE, BLEND_ZERO, BLEND_ONE, BLEND_ZERO] || b[..2] == [BLEND_ONE, BLEND_ZERO] && b[2..] == [0, 0] {
            (gl.Disable)(BLEND);
        } else {
            (gl.Enable)(BLEND);
            (gl.BlendFuncSeparate)(blend_factor(b[0], false), blend_factor(b[1], true), blend_factor(b[2], false), blend_factor(b[3], true));
        }
    }
}

/// The clip window in output pixels (OpenGL counts rows from the bottom).
pub unsafe fn scissor(gl: &Gl, clip: [u32; 4], w: u32, h: u32, ow: u32, oh: u32) {
    let (sx, sy) = (ow as f32 / w as f32, oh as f32 / h as f32);
    let [x0, y0, x1, y1] = clip.map(|v| v as f32);
    unsafe {
        (gl.Enable)(SCISSOR_TEST);
        (gl.Scissor)(
            (x0 * sx).round() as i32,
            ((h as f32 - y1) * sy).round() as i32,
            ((x1 - x0) * sx).round() as i32,
            ((y1 - y0) * sy).round() as i32,
        );
    }
}

pub unsafe fn depth(gl: &Gl, s: &State) {
    unsafe {
        if s.depth_mode == DEPTHBUFFER_DISABLE {
            (gl.Disable)(DEPTH_TEST);
        } else {
            (gl.Enable)(DEPTH_TEST);
            (gl.DepthFunc)(NEVER + (s.depth_func & 7));
        }
        (gl.DepthMask)((s.depth_mask && s.depth_mode != DEPTHBUFFER_DISABLE) as u8);
    }
}

/// Everything a triangle or line needs: program, uniforms, blending, depth, clip, texture.
pub unsafe fn apply(gl: &Gl, p: &Program, s: &State, tex: &mut Textures, w: u32, h: u32, ow: u32, oh: u32) {
    unsafe {
        (gl.UseProgram)(p.id);
        (gl.Uniform2f)(p.loc("uScreen"), w as f32, h as f32);
        (gl.Uniform1i)(p.loc("uDepthMode"), s.depth_mode as i32);
        (gl.Uniform1f)(p.loc("uDepthBias"), s.depth_bias as f32);
        (gl.Uniform2i)(p.loc("uAlphaTest"), s.alpha_test[0] as i32, s.alpha_test[1] as i32);
        let (m, a) = (s.color_mask.0 as u8, s.color_mask.1 as u8);
        (gl.ColorMask)(m, m, m, a);
        (gl.Uniform4i)(p.loc("uCC"), s.cc[0] as i32, s.cc[1] as i32, s.cc[2] as i32, s.cc[3] as i32);
        (gl.Uniform1i)(p.loc("uCCInvert"), (s.cc[4] != 0) as i32);
        (gl.Uniform4i)(p.loc("uAC"), s.ac[0] as i32, s.ac[1] as i32, s.ac[2] as i32, s.ac[3] as i32);
        (gl.Uniform1i)(p.loc("uACInvert"), (s.ac[4] != 0) as i32);
        let [r, g, b] = rgb(s.constant);
        (gl.Uniform4f)(p.loc("uConst"), r, g, b, (s.constant >> 24) as f32 / 255.0);
        (gl.Uniform1i)(p.loc("uChroma"), s.chroma as i32);
        let [r, g, b] = rgb(s.key);
        (gl.Uniform3f)(p.loc("uKey"), r, g, b);
        let fog = if s.fog_mode & 0xff == FOG_WITH_TABLE { 2 } else { (s.fog_mode & 0xff == 1) as i32 };
        (gl.Uniform1i)(p.loc("uFog"), fog);
        if fog == 2 {
            let t: Vec<f32> = s.fog_table.iter().map(|v| *v as f32).collect();
            (gl.Uniform1fv)(p.loc("uFogTable"), 64, t.as_ptr());
        }
        let [r, g, b] = rgb(s.fog_color);
        (gl.Uniform3f)(p.loc("uFogColor"), r, g, b);
        blend(gl, s.blend);
        depth(gl, s);
        scissor(gl, s.clip, w, h, ow, oh);
        (gl.ActiveTexture)(TEXTURE0);
        (gl.Uniform1i)(p.loc("uTex"), 0);
        let mut tex_fn = s.tex_fn;
        let bound = match s.source {
            Some(src) if s.uses_texture() => tex.bind(gl, src),
            _ => None,
        };
        match bound {
            Some((tw, th)) => {
                let long = tw.max(th) as f32;
                (gl.Uniform2f)(p.loc("uSTScale"), long / (256.0 * tw as f32), long / (256.0 * th as f32));
                let f = |m: u32| if m == FILTER_BILINEAR { LINEAR } else { NEAREST };
                let c = |m: u32| if m == CLAMP_CLAMP { CLAMP_TO_EDGE } else { REPEAT };
                // With mipmaps: GL_{NEAREST,LINEAR}_MIPMAP_{NEAREST,LINEAR}, 0x2700 to 0x2703.
                // Deviation: GR_MIPMAP_NEAREST_DITHER is drawn without the dither.
                let min = match s.mipmap {
                    (MIPMAP_DISABLE, _) => f(s.filter.0),
                    (_, blend) => NEAREST_MIPMAP_NEAREST + (f(s.filter.0) - NEAREST) + 2 * blend as i32,
                };
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MIN_FILTER, min);
                (gl.TexParameterf)(TEXTURE_2D, TEXTURE_LOD_BIAS, s.lod_bias);
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_MAG_FILTER, f(s.filter.1));
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_S, c(s.clamp.0));
                (gl.TexParameteri)(TEXTURE_2D, TEXTURE_WRAP_T, c(s.clamp.1));
            }
            None => tex_fn = 10,
        }
        (gl.Uniform1i)(p.loc("uTexFn"), tex_fn as i32);
    }
}
