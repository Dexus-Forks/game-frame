//! The two programs, in GLSL 1.20 so that an OpenGL 2.1 context (macOS's, under Wine) runs
//! them as well as Mesa's.
//!
//! `scene` projects camera-space vertices as SGL does (*inferred* from the camera rectangle
//! POD gives each viewport, `FUN_0049dbe0`): the camera's square window, at distance 1 /
//! zoom, covers the camera rectangle on the device, x to the right and y up. So a point
//! (x, y, z) lands at `(cx + sx x / z, cy - sy y / z)` device pixels, with `(cx, cy)` the
//! rectangle's centre and `sx`, `sy` zoom times its half sizes. Depth is `1 - 2 near / z`.
//!
//! `overlay` draws a picture over a rectangle given in normalised device coordinates;
//! `volume` adds a light volume's light.

use super::gl::*;

pub const SCENE_FLOATS: usize = 12;

const SCENE_VS: &str = r#"#version 120
attribute vec3 aPos;
attribute vec2 aUV;
attribute vec4 aMod;
attribute vec3 aAdd;
uniform vec4 uCam;
uniform vec2 uDev;
uniform float uNear;
varying vec2 vUV;
varying vec4 vMod;
varying vec3 vAdd;
varying float vZ;
void main() {
    float z = aPos.z;
    float sx = uCam.x * z + uCam.z * aPos.x;
    float sy = uCam.y * z - uCam.w * aPos.y;
    gl_Position = vec4(sx / uDev.x * 2.0 - z, z - sy / uDev.y * 2.0, z - 2.0 * uNear, z);
    vUV = aUV;
    vMod = aMod;
    vAdd = aAdd;
    vZ = z;
}
"#;

const SCENE_FS: &str = r#"#version 120
uniform sampler2D uTex;
uniform int uTextured;
uniform vec4 uFog;
uniform int uFogOn;
varying vec2 vUV;
varying vec4 vMod;
varying vec3 vAdd;
varying float vZ;
void main() {
    vec4 t = uTextured == 1 ? texture2D(uTex, vUV) : vec4(1.0);
    if (t.a < 0.5) discard;
    vec3 c = t.rgb * vMod.rgb + vAdd;
    if (uFogOn == 1) {
        float f = clamp(exp(-uFog.a * vZ), 0.0, 1.0);
        c = mix(uFog.rgb, c, f);
    }
    gl_FragColor = vec4(c, vMod.a);
}
"#;

const OVERLAY_VS: &str = r#"#version 120
attribute vec2 aPos;
attribute vec2 aUV;
varying vec2 vUV;
void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
    vUV = aUV;
}
"#;

const OVERLAY_FS: &str = r#"#version 120
uniform sampler2D uImg;
varying vec2 vUV;
void main() {
    gl_FragColor = texture2D(uImg, vUV);
}
"#;

/// The most planes a light volume may have here (POD's has 41).
pub const MAX_PLANES: usize = 64;

const VOLUME_VS: &str = r#"#version 120
attribute vec2 aPos;
void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

/// Each pixel's place in the camera's space, from the depth, tested against the volume's
/// planes, as the PowerVR chip tested planes per pixel.
const VOLUME_FS: &str = r#"#version 120
uniform sampler2D uDepth;
uniform vec2 uSize;
uniform vec4 uCam;
uniform vec2 uDev;
uniform float uNear;
uniform vec4 uPlanes[64];
uniform int uCount;
uniform vec3 uColor;
void main() {
    float d = texture2D(uDepth, gl_FragCoord.xy / uSize).r;
    if (d >= 1.0) discard;
    float z = 2.0 * uNear / (1.0 - (d * 2.0 - 1.0));
    float px = gl_FragCoord.x / uSize.x * uDev.x;
    float py = (1.0 - gl_FragCoord.y / uSize.y) * uDev.y;
    vec3 p = vec3((px - uCam.x) * z / uCam.z, -(py - uCam.y) * z / uCam.w, z);
    for (int i = 0; i < 64; i++) {
        if (i >= uCount) break;
        if (dot(uPlanes[i].xyz, p) > uPlanes[i].w) discard;
    }
    gl_FragColor = vec4(uColor, 1.0);
}
"#;

pub struct Program {
    pub id: u32,
    locs: Vec<(&'static str, i32)>,
}

impl Program {
    pub fn loc(&self, name: &str) -> i32 {
        self.locs.iter().find(|(n, _)| *n == name).map_or(-1, |(_, l)| *l)
    }
}

pub struct Programs {
    pub scene: Program,
    pub overlay: Program,
    /// A light volume's light, where the surface is inside its planes.
    pub volume: Program,
}

unsafe fn compile(gl: &Gl, kind: u32, src: &str) -> Result<u32, String> {
    unsafe {
        let s = (gl.CreateShader)(kind);
        let p = src.as_ptr();
        let n = src.len() as i32;
        (gl.ShaderSource)(s, 1, &p, &n);
        (gl.CompileShader)(s);
        let mut ok = 0;
        (gl.GetShaderiv)(s, COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let mut log = vec![0u8; 2048];
            let mut len = 0;
            (gl.GetShaderInfoLog)(s, 2048, &mut len, log.as_mut_ptr());
            return Err(format!("shader: {}", String::from_utf8_lossy(&log[..len.max(0) as usize])));
        }
        Ok(s)
    }
}

unsafe fn link(gl: &Gl, vs: &str, fs: &str, attrs: &[&str], uniforms: &[&'static str]) -> Result<Program, String> {
    unsafe {
        let p = (gl.CreateProgram)();
        (gl.AttachShader)(p, compile(gl, VERTEX_SHADER, vs)?);
        (gl.AttachShader)(p, compile(gl, FRAGMENT_SHADER, fs)?);
        for (i, a) in attrs.iter().enumerate() {
            let c = std::ffi::CString::new(*a).unwrap();
            (gl.BindAttribLocation)(p, i as u32, c.as_ptr().cast());
        }
        (gl.LinkProgram)(p);
        let mut ok = 0;
        (gl.GetProgramiv)(p, LINK_STATUS, &mut ok);
        if ok == 0 {
            let mut log = vec![0u8; 2048];
            let mut len = 0;
            (gl.GetProgramInfoLog)(p, 2048, &mut len, log.as_mut_ptr());
            return Err(format!("link: {}", String::from_utf8_lossy(&log[..len.max(0) as usize])));
        }
        let locs = uniforms
            .iter()
            .map(|u| {
                let c = std::ffi::CString::new(*u).unwrap();
                (*u, (gl.GetUniformLocation)(p, c.as_ptr().cast()))
            })
            .collect();
        Ok(Program { id: p, locs })
    }
}

pub unsafe fn build(gl: &Gl) -> Result<Programs, String> {
    unsafe {
        Ok(Programs {
            scene: link(gl, SCENE_VS, SCENE_FS, &["aPos", "aUV", "aMod", "aAdd"], &["uCam", "uDev", "uNear", "uTex", "uTextured", "uFog", "uFogOn"])?,
            overlay: link(gl, OVERLAY_VS, OVERLAY_FS, &["aPos", "aUV"], &["uImg"])?,
            volume: link(gl, VOLUME_VS, VOLUME_FS, &["aPos"], &["uDepth", "uSize", "uCam", "uDev", "uNear", "uPlanes", "uCount", "uColor"])?,
        })
    }
}
