//! The GLSL programs: the Voodoo pixel pipeline (combiners, chroma key, table fog) for
//! triangles and lines, the LFB picture, and the final copy to the window with gamma.

use crate::gl::*;

/// Vertex attributes of the scene program: x, y, ooz, oow, r, g, b, a, sow, tow.
pub const FLOATS: usize = 10;

const SCENE_VS: &str = r#"#version 130
in vec4 aPos;   // x, y, ooz, oow
in vec4 aColor; // r, g, b, a: 0-255
in vec2 aST;    // sow, tow
uniform vec2 uScreen;
uniform int uDepthMode;
uniform float uDepthBias; // grDepthBiasLevel, in 16-bit depth units
uniform vec2 uSTScale;
smooth out vec2 vST;
noperspective out vec4 vColor;
noperspective out float vOow;
void main() {
    float oow = max(aPos.w, 1e-9);
    float w = 1.0 / oow;
    vec2 ndc = vec2(aPos.x / uScreen.x * 2.0 - 1.0, 1.0 - aPos.y / uScreen.y * 2.0);
    // Depth rises with the value the Voodoo stores, so Glide's compare functions map to
    // OpenGL's as they are: W buffer 1/oow (here 1 - oow, linear on screen), Z buffer ooz.
    // Deviation: in the W buffer the bias is added to this linear depth. The Voodoo added
    // it to its floating point w.
    float d = (uDepthMode == 2 ? 1.0 - oow : aPos.z / 65535.0) + uDepthBias / 65535.0;
    gl_Position = vec4(ndc * w, (2.0 * d - 1.0) * w, w);
    vST = aST * w * uSTScale;
    vColor = aColor / 255.0;
    vOow = oow;
}
"#;

const SCENE_FS: &str = r#"#version 130
uniform sampler2D uTex;
uniform int uTexFn;      // GR_TEXTURECOMBINE_*: 0 zero, 10 one, else decal
uniform ivec4 uCC;       // colour: function, factor, local, other
uniform int uCCInvert;
uniform ivec4 uAC;       // alpha: function, factor, local, other
uniform int uACInvert;
uniform vec4 uConst;
uniform int uChroma;
uniform vec3 uKey;
uniform int uFog;        // 0 off, 1 iterated alpha, 2 table
uniform float uFogTable[64];
uniform vec3 uFogColor;
uniform ivec2 uAlphaTest; // GrCmpFnc_t (7 always), reference 0-255
smooth in vec2 vST;
noperspective in vec4 vColor;
noperspective in float vOow;
out vec4 oColor;

vec3 cfactor(int f, vec3 cl, float al, float ao, float at) {
    if (f == 1) return cl;
    if (f == 2) return vec3(ao);
    if (f == 3) return vec3(al);
    if (f == 4) return vec3(at);
    if (f == 8) return vec3(1.0);
    if (f == 9) return 1.0 - cl;
    if (f == 10) return vec3(1.0 - ao);
    if (f == 11) return vec3(1.0 - al);
    if (f == 12) return vec3(1.0 - at);
    return vec3(0.0);
}
float afactor(int f, float al, float ao, float at) {
    if (f == 1 || f == 3) return al;
    if (f == 2) return ao;
    if (f == 4) return at;
    if (f == 8) return 1.0;
    if (f == 9 || f == 11) return 1.0 - al;
    if (f == 10) return 1.0 - ao;
    if (f == 12) return 1.0 - at;
    return 0.0;
}
vec3 cfunc(int fn, vec3 f, vec3 cl, vec3 co, float al) {
    if (fn == 1) return cl;
    if (fn == 2) return vec3(al);
    if (fn == 3) return f * co;
    if (fn == 4) return f * co + cl;
    if (fn == 5) return f * co + al;
    if (fn == 6) return f * (co - cl);
    if (fn == 7) return f * (co - cl) + cl;
    if (fn == 8) return f * (co - cl) + al;
    if (fn == 9) return cl - f * cl;
    if (fn == 16) return al - f * cl;
    return vec3(0.0);
}
float afunc(int fn, float f, float al, float ao) {
    if (fn == 1 || fn == 2) return al;
    if (fn == 3) return f * ao;
    if (fn == 4 || fn == 5) return f * ao + al;
    if (fn == 6) return f * (ao - al);
    if (fn == 7 || fn == 8) return f * (ao - al) + al;
    if (fn == 9 || fn == 16) return al - f * al;
    return 0.0;
}
float fogTable(float oow) {
    // The table's entry i is for w = 2^(3 + i/4) / (8 - i%4) (guFogTableIndexToW); the
    // Voodoo blends between neighbouring entries.
    float w = 1.0 / max(oow, 1e-9);
    if (w <= 1.0) return uFogTable[0];
    float e = floor(log2(w));
    float fr = w / exp2(e);
    float b0, b1; int m;
    if (fr < 8.0 / 7.0) { m = 0; b0 = 1.0; b1 = 8.0 / 7.0; }
    else if (fr < 4.0 / 3.0) { m = 1; b0 = 8.0 / 7.0; b1 = 4.0 / 3.0; }
    else if (fr < 1.6) { m = 2; b0 = 4.0 / 3.0; b1 = 1.6; }
    else { m = 3; b0 = 1.6; b1 = 2.0; }
    int i = int(e) * 4 + m;
    if (i >= 63) return uFogTable[63];
    return mix(uFogTable[i], uFogTable[i + 1], (fr - b0) / (b1 - b0));
}
void main() {
    vec4 tex = texture(uTex, vST);
    if (uTexFn == 0) tex = vec4(0.0);
    if (uTexFn == 10) tex = vec4(1.0);
    float al = uAC.z == 1 ? uConst.a : vColor.a;
    float ao = uAC.w == 0 ? vColor.a : (uAC.w == 1 ? tex.a : uConst.a);
    vec3 cl = uCC.z == 1 ? uConst.rgb : vColor.rgb;
    vec3 co = uCC.w == 0 ? vColor.rgb : (uCC.w == 1 ? tex.rgb : uConst.rgb);
    if (uChroma != 0 && all(lessThan(abs(co - uKey), vec3(0.5 / 255.0)))) discard;
    vec3 c = clamp(cfunc(uCC.x, cfactor(uCC.y, cl, al, ao, tex.a), cl, co, al), 0.0, 1.0);
    if (uCCInvert != 0) c = 1.0 - c;
    float a = clamp(afunc(uAC.x, afactor(uAC.y, al, ao, tex.a), al, ao), 0.0, 1.0);
    if (uACInvert != 0) a = 1.0 - a;
    if (uAlphaTest.x != 7) {
        int ai = int(a * 255.0 + 0.5), r = uAlphaTest.y, f = uAlphaTest.x;
        bool pass = f == 1 ? ai < r : f == 2 ? ai == r : f == 3 ? ai <= r : f == 4 ? ai > r : f == 5 ? ai != r : f == 6 ? ai >= r : false;
        if (!pass) discard;
    }
    if (uFog == 2) c = mix(c, uFogColor, fogTable(vOow) / 255.0);
    else if (uFog == 1) c = mix(c, uFogColor, vColor.a);
    oColor = vec4(c, a);
}
"#;

/// A rectangle from `gl_VertexID` 0–3 (triangle strip) over `uRect` (x0, y0, x1, y1 in the
/// game's pixels), with texture coordinates 0–1 top to bottom.
const QUAD_VS: &str = r#"#version 130
uniform vec4 uRect;
uniform vec2 uScreen;
out vec2 vUV;
void main() {
    vec2 c = vec2(gl_VertexID & 1, gl_VertexID >> 1);
    vec2 p = mix(uRect.xy, uRect.zw, c);
    vUV = c;
    gl_Position = vec4(p.x / uScreen.x * 2.0 - 1.0, 1.0 - p.y / uScreen.y * 2.0, 0.0, 1.0);
}
"#;

const LFB_FS: &str = r#"#version 130
uniform sampler2D uImg;
uniform int uChroma;
uniform vec3 uKey;
in vec2 vUV;
out vec4 oColor;
void main() {
    vec4 c = texture(uImg, vUV);
    if (c.a == 0.0) discard;
    if (uChroma != 0 && all(lessThan(abs(c.rgb - uKey), vec3(0.5 / 255.0)))) discard;
    oColor = c;
}
"#;

/// The back buffer to the window: the frame buffer texture is upside down (OpenGL's rows run
/// bottom to top), and the Voodoo's gamma table applies on the way out.
const PRESENT_FS: &str = r#"#version 130
uniform sampler2D uImg;
uniform float uGamma;
in vec2 vUV;
out vec4 oColor;
void main() {
    vec3 c = texture(uImg, vec2(vUV.x, 1.0 - vUV.y)).rgb;
    oColor = vec4(pow(c, vec3(1.0 / uGamma)), 1.0);
}
"#;

unsafe fn shader(gl: &Gl, kind: u32, src: &str) -> Result<u32, String> {
    unsafe {
        let s = (gl.CreateShader)(kind);
        let p = src.as_ptr();
        let n = src.len() as i32;
        (gl.ShaderSource)(s, 1, &p, &n);
        (gl.CompileShader)(s);
        let mut ok = 0;
        (gl.GetShaderiv)(s, COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let mut log = vec![0u8; 4096];
            let mut len = 0;
            (gl.GetShaderInfoLog)(s, 4096, &mut len, log.as_mut_ptr());
            return Err(String::from_utf8_lossy(&log[..len.max(0) as usize]).into_owned());
        }
        Ok(s)
    }
}

unsafe fn program(gl: &Gl, vs: &str, fs: &str, attribs: &[&std::ffi::CStr]) -> Result<u32, String> {
    unsafe {
        let p = (gl.CreateProgram)();
        (gl.AttachShader)(p, shader(gl, VERTEX_SHADER, vs)?);
        (gl.AttachShader)(p, shader(gl, FRAGMENT_SHADER, fs)?);
        for (i, a) in attribs.iter().enumerate() {
            (gl.BindAttribLocation)(p, i as u32, a.as_ptr().cast());
        }
        (gl.LinkProgram)(p);
        let mut ok = 0;
        (gl.GetProgramiv)(p, LINK_STATUS, &mut ok);
        if ok == 0 {
            let mut log = vec![0u8; 4096];
            let mut len = 0;
            (gl.GetProgramInfoLog)(p, 4096, &mut len, log.as_mut_ptr());
            return Err(String::from_utf8_lossy(&log[..len.max(0) as usize]).into_owned());
        }
        Ok(p)
    }
}

/// A program and the locations of its uniforms, looked up by name once.
pub struct Program {
    pub id: u32,
    names: Vec<(&'static str, i32)>,
}

impl Program {
    pub fn loc(&self, name: &str) -> i32 {
        self.names.iter().find(|(n, _)| *n == name).map_or(-1, |(_, l)| *l)
    }
}

unsafe fn with_uniforms(gl: &Gl, id: u32, names: &[&'static str]) -> Program {
    let names = names
        .iter()
        .map(|n| {
            let c = std::ffi::CString::new(*n).unwrap();
            (*n, unsafe { (gl.GetUniformLocation)(id, c.as_ptr().cast()) })
        })
        .collect();
    Program { id, names }
}

pub struct Programs {
    pub scene: Program,
    pub lfb: Program,
    pub present: Program,
}

pub unsafe fn build(gl: &Gl) -> Result<Programs, String> {
    unsafe {
        let scene = program(gl, SCENE_VS, SCENE_FS, &[c"aPos", c"aColor", c"aST"])?;
        let lfb = program(gl, QUAD_VS, LFB_FS, &[])?;
        let present = program(gl, QUAD_VS, PRESENT_FS, &[])?;
        Ok(Programs {
            scene: with_uniforms(gl, scene, &[
                "uScreen", "uDepthMode", "uDepthBias", "uAlphaTest", "uSTScale", "uTex", "uTexFn", "uCC", "uCCInvert", "uAC", "uACInvert",
                "uConst", "uChroma", "uKey", "uFog", "uFogTable", "uFogColor",
            ]),
            lfb: with_uniforms(gl, lfb, &["uRect", "uScreen", "uImg", "uChroma", "uKey"]),
            present: with_uniforms(gl, present, &["uRect", "uScreen", "uImg", "uGamma"]),
        })
    }
}
