//! OpenGL entry points, loaded at run time through `wglGetProcAddress` (and `opengl32.dll`'s
//! own exports for the OpenGL 1.1 ones). Only what the renderer uses.

#![allow(non_snake_case, clippy::too_many_arguments)]

use crate::win32::{GetProcAddress, LoadLibraryA, wglGetProcAddress};
use std::ffi::c_void;

pub const TEXTURE_2D: u32 = 0x0de1;
pub const BLEND: u32 = 0x0be2;
pub const DEPTH_TEST: u32 = 0x0b71;
pub const SCISSOR_TEST: u32 = 0x0c11;
pub const DEPTH_CLAMP: u32 = 0x864f;
pub const CULL_FACE: u32 = 0x0b44;
pub const TRIANGLES: u32 = 0x0004;
pub const LINES: u32 = 0x0001;
pub const POINTS: u32 = 0x0000;
pub const COLOR_BUFFER_BIT: u32 = 0x4000;
pub const DEPTH_BUFFER_BIT: u32 = 0x0100;
pub const UNSIGNED_BYTE: u32 = 0x1401;
pub const FLOAT: u32 = 0x1406;
pub const RGBA: u32 = 0x1908;
pub const RGBA8: u32 = 0x8058;
pub const DEPTH_COMPONENT: u32 = 0x1902;
pub const DEPTH_COMPONENT24: u32 = 0x81a6;
pub const TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const TEXTURE_WRAP_S: u32 = 0x2802;
pub const TEXTURE_WRAP_T: u32 = 0x2803;
pub const NEAREST: i32 = 0x2600;
pub const LINEAR: i32 = 0x2601;
pub const NEAREST_MIPMAP_NEAREST: i32 = 0x2700;
pub const TEXTURE_BASE_LEVEL: u32 = 0x813c;
pub const TEXTURE_MAX_LEVEL: u32 = 0x813d;
pub const TEXTURE_LOD_BIAS: u32 = 0x8501;
pub const REPEAT: i32 = 0x2901;
pub const CLAMP_TO_EDGE: i32 = 0x812f;
pub const ARRAY_BUFFER: u32 = 0x8892;
pub const STREAM_DRAW: u32 = 0x88e0;
pub const FRAMEBUFFER: u32 = 0x8d40;
pub const READ_FRAMEBUFFER: u32 = 0x8ca8;
pub const DRAW_FRAMEBUFFER: u32 = 0x8ca9;
pub const RENDERBUFFER: u32 = 0x8d41;
pub const COLOR_ATTACHMENT0: u32 = 0x8ce0;
pub const DEPTH_ATTACHMENT: u32 = 0x8d00;
pub const FRAMEBUFFER_COMPLETE: u32 = 0x8cd5;
pub const VERTEX_SHADER: u32 = 0x8b31;
pub const FRAGMENT_SHADER: u32 = 0x8b30;
pub const COMPILE_STATUS: u32 = 0x8b81;
pub const LINK_STATUS: u32 = 0x8b82;
pub const TEXTURE0: u32 = 0x84c0;
pub const PACK_ALIGNMENT: u32 = 0x0d05;
pub const UNPACK_ALIGNMENT: u32 = 0x0cf5;
pub const NEVER: u32 = 0x0200;
pub const VERSION: u32 = 0x1f02;
pub const RENDERER: u32 = 0x1f01;

macro_rules! gl_functions {
    ($($name:ident : fn($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;)*) => {
        /// The loaded function table.
        pub struct Gl {
            $(pub $name: unsafe extern "system" fn($($ty),*) $(-> $ret)?,)*
        }

        impl Gl {
            /// Load every function; call with a context current. Names that are missing are
            /// reported all at once.
            pub unsafe fn load() -> Result<Gl, String> {
                let lib = unsafe { LoadLibraryA(c"opengl32.dll".as_ptr().cast()) };
                let mut missing = Vec::new();
                let mut find = |name: &str| -> *const c_void {
                    let c = std::ffi::CString::new(name).unwrap();
                    let mut p = unsafe { wglGetProcAddress(c.as_ptr().cast()) };
                    if (p as usize) < 4 || p as isize == -1 {
                        p = unsafe { GetProcAddress(lib, c.as_ptr().cast()) };
                    }
                    if p.is_null() {
                        missing.push(name.to_string());
                    }
                    p
                };
                let table = ($(find(concat!("gl", stringify!($name))),)*);
                if !missing.is_empty() {
                    return Err(format!("OpenGL functions missing: {}", missing.join(", ")));
                }
                #[allow(non_snake_case)]
                let ($($name,)*) = table;
                Ok(Gl { $($name: unsafe { std::mem::transmute::<*const c_void, unsafe extern "system" fn($($ty),*) $(-> $ret)?>($name) },)* })
            }
        }
    };
}

gl_functions! {
    Enable: fn(cap: u32);
    Disable: fn(cap: u32);
    BlendFuncSeparate: fn(a: u32, b: u32, c: u32, d: u32);
    DepthFunc: fn(f: u32);
    DepthMask: fn(m: u8);
    Viewport: fn(x: i32, y: i32, w: i32, h: i32);
    Scissor: fn(x: i32, y: i32, w: i32, h: i32);
    Clear: fn(mask: u32);
    ClearColor: fn(r: f32, g: f32, b: f32, a: f32);
    ClearDepth: fn(d: f64);
    LineWidth: fn(w: f32);
    PointSize: fn(s: f32);
    ColorMask: fn(r: u8, g: u8, b: u8, a: u8);
    TexParameterf: fn(target: u32, name: u32, v: f32);
    GenTextures: fn(n: i32, t: *mut u32);
    DeleteTextures: fn(n: i32, t: *const u32);
    BindTexture: fn(target: u32, t: u32);
    TexImage2D: fn(target: u32, level: i32, internal: i32, w: i32, h: i32, border: i32, format: u32, ty: u32, data: *const c_void);
    TexParameteri: fn(target: u32, name: u32, v: i32);
    TexSubImage2D: fn(target: u32, level: i32, x: i32, y: i32, w: i32, h: i32, format: u32, ty: u32, data: *const c_void);
    ReadPixels: fn(x: i32, y: i32, w: i32, h: i32, format: u32, ty: u32, data: *mut c_void);
    PixelStorei: fn(name: u32, v: i32);
    DrawArrays: fn(mode: u32, first: i32, count: i32);
    GetString: fn(name: u32) -> *const u8;
    GetError: fn() -> u32;
    ActiveTexture: fn(t: u32);
    GenBuffers: fn(n: i32, b: *mut u32);
    BindBuffer: fn(target: u32, b: u32);
    BufferData: fn(target: u32, size: isize, data: *const c_void, usage: u32);
    VertexAttribPointer: fn(i: u32, size: i32, ty: u32, norm: u8, stride: i32, offset: *const c_void);
    EnableVertexAttribArray: fn(i: u32);
    CreateShader: fn(kind: u32) -> u32;
    ShaderSource: fn(s: u32, n: i32, src: *const *const u8, len: *const i32);
    CompileShader: fn(s: u32);
    GetShaderiv: fn(s: u32, name: u32, v: *mut i32);
    GetShaderInfoLog: fn(s: u32, max: i32, len: *mut i32, log: *mut u8);
    CreateProgram: fn() -> u32;
    AttachShader: fn(p: u32, s: u32);
    BindAttribLocation: fn(p: u32, i: u32, name: *const u8);
    LinkProgram: fn(p: u32);
    GetProgramiv: fn(p: u32, name: u32, v: *mut i32);
    GetProgramInfoLog: fn(p: u32, max: i32, len: *mut i32, log: *mut u8);
    UseProgram: fn(p: u32);
    GetUniformLocation: fn(p: u32, name: *const u8) -> i32;
    Uniform1i: fn(l: i32, v: i32);
    Uniform1f: fn(l: i32, v: f32);
    Uniform2f: fn(l: i32, a: f32, b: f32);
    Uniform3f: fn(l: i32, a: f32, b: f32, c: f32);
    Uniform4f: fn(l: i32, a: f32, b: f32, c: f32, d: f32);
    Uniform2i: fn(l: i32, a: i32, b: i32);
    Uniform4i: fn(l: i32, a: i32, b: i32, c: i32, d: i32);
    Uniform1fv: fn(l: i32, n: i32, v: *const f32);
    GenFramebuffers: fn(n: i32, f: *mut u32);
    BindFramebuffer: fn(target: u32, f: u32);
    FramebufferTexture2D: fn(target: u32, attach: u32, textarget: u32, t: u32, level: i32);
    GenRenderbuffers: fn(n: i32, r: *mut u32);
    BindRenderbuffer: fn(target: u32, r: u32);
    RenderbufferStorage: fn(target: u32, format: u32, w: i32, h: i32);
    FramebufferRenderbuffer: fn(target: u32, attach: u32, rbtarget: u32, r: u32);
    CheckFramebufferStatus: fn(target: u32) -> u32;
    BlitFramebuffer: fn(sx0: i32, sy0: i32, sx1: i32, sy1: i32, dx0: i32, dy0: i32, dx1: i32, dy1: i32, mask: u32, filter: u32);
}

/// In a debug build, log every OpenGL error raised since the last check, named by `when`.
/// A release build does nothing.
pub unsafe fn check(gl: &Gl, when: &str) {
    if cfg!(debug_assertions) {
        for _ in 0..8 {
            let e = unsafe { (gl.GetError)() };
            if e == 0 {
                break;
            }
            shim::log!("OpenGL error {e:#x} {when}");
        }
    }
}

/// Make and bind one vertex array object, which a core context needs before it draws. Loaded
/// apart from the table, as an OpenGL 2.1 context has no such call.
pub unsafe fn bind_vertex_array() {
    unsafe {
        let gen_ = wglGetProcAddress(c"glGenVertexArrays".as_ptr().cast());
        let bind = wglGetProcAddress(c"glBindVertexArray".as_ptr().cast());
        if (gen_ as usize) < 4 || (bind as usize) < 4 {
            shim::log!("no glGenVertexArrays in a core context");
            return;
        }
        let gen_: unsafe extern "system" fn(i32, *mut u32) = std::mem::transmute(gen_);
        let bind: unsafe extern "system" fn(u32) = std::mem::transmute(bind);
        let mut vao = 0;
        gen_(1, &mut vao);
        bind(vao);
    }
}
