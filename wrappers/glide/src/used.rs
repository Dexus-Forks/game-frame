//! Which fields of a `GrVertex` the Voodoo reads under the current state, so the call log
//! (`callargs`) writes only those. A game leaves the fields it does not need as they were:
//! POD's race leaves old values in `z`, `ooz` and TMU 1 (*measured*, its 3dfx build). The card
//! never reads them, and a port's log would hold different leftovers, so writing them would
//! make two equal frames differ.
//!
//! The rules follow the pixel pipeline as the scene shader draws it (`render/shaders.rs`):
//! - `x`, `y`: always. `z`: never (Glide 2's `glide.h`: "Z is ignored").
//! - `r`, `g`, `b`: when the colour combine reads the iterated colour as its local or other
//!   colour, and the colour is not the delta-0 constant (`GR_COLORCOMBINE_*_DELTA0`).
//! - `a`: when either combine reads the iterated alpha as its local or other alpha, or fog
//!   follows the iterated alpha (`GR_FOG_WITH_ITERATED_ALPHA`).
//! - `ooz`: with a Z buffer. `oow`: with a W buffer, with table fog, or with a texture.
//! - TMU 0's `sow`, `tow`: with a texture. Its own `oow`: with a texture and
//!   `GR_STWHINT_W_DIFF_TMU0`.
//! - TMU 1's: with a texture and its `GR_STWHINT_*_DIFF_TMU1` hint (*inferred*: the wrapper
//!   draws with one TMU and reads none of them, but a game that sets the hint meant them).
//!
//! A source or factor counts as read when the combine function uses it: `GR_COMBINE_FUNCTION_ZERO`
//! reads nothing, `LOCAL` the local colour only. The chroma key compares the other colour, so
//! with the key on it is read whatever the function.

/// The state that decides which fields are read, as the game set it.
#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    /// `grColorCombine` and `grAlphaCombine`: function, factor, local, other, invert.
    pub cc: [u32; 5],
    pub ac: [u32; 5],
    pub chroma: bool,
    /// The colour is `grConstantColorValue4`'s, not the vertices' (`*_DELTA0`).
    pub delta0: bool,
    pub depth_mode: u32,
    pub fog_mode: u32,
    pub stw_hint: u32,
}

// GrCombineFunction_t, GrCombineFactor_t, GrCombineLocal_t, GrCombineOther_t.
const FUNC_ZERO: u32 = 0x0;
const FUNC_LOCAL_ALPHA: u32 = 0x2;
const FUNC_SCALE_OTHER: u32 = 0x3;
const FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA: u32 = 0x5;
const FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA: u32 = 0x8;
const FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA: u32 = 0x10;
const FACTOR_LOCAL: u32 = 0x1;
const FACTOR_OTHER_ALPHA: u32 = 0x2;
const FACTOR_LOCAL_ALPHA: u32 = 0x3;
const FACTOR_TEXTURE_ALPHA: u32 = 0x4;
const FACTOR_ONE_MINUS_LOCAL: u32 = 0x9;
const FACTOR_ONE_MINUS_OTHER_ALPHA: u32 = 0xa;
const FACTOR_ONE_MINUS_LOCAL_ALPHA: u32 = 0xb;
const FACTOR_ONE_MINUS_TEXTURE_ALPHA: u32 = 0xc;
const ITERATED: u32 = 0x0;
const OTHER_TEXTURE: u32 = 0x1;
// GrDepthBufferMode_t, GrFogMode_t, GrSTWHint_t.
const DEPTH_ZBUFFER: u32 = 0x1;
const DEPTH_WBUFFER: u32 = 0x2;
const DEPTH_ZBUFFER_COMPARE_TO_BIAS: u32 = 0x3;
const DEPTH_WBUFFER_COMPARE_TO_BIAS: u32 = 0x4;
const FOG_WITH_ITERATED_ALPHA: u32 = 0x1;
const FOG_WITH_TABLE: u32 = 0x2;
const STWHINT_W_DIFF_TMU0: u32 = 0x2;
const STWHINT_W_DIFF_TMU1: u32 = 0x8;
const STWHINT_ST_DIFF_TMU1: u32 = 0x10;

/// The fields read, in `GrVertex` order. `tmu` is TMU 0's and TMU 1's `[st, w]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fields {
    pub rgb: bool,
    pub ooz: bool,
    pub a: bool,
    pub oow: bool,
    pub tmu: [[bool; 2]; 2],
}

/// What one combine unit reads: its local value, its local alpha (the alpha unit's local), its
/// other value, the other alpha, and the texture alpha.
struct Combine {
    local: bool,
    local_alpha: bool,
    other: bool,
    other_alpha: bool,
    texture_alpha: bool,
}

fn combine([func, factor, ..]: [u32; 5]) -> Combine {
    // The factor only counts when the function scales by it.
    let f = |a: u32, b: u32| func >= FUNC_SCALE_OTHER && (factor == a || factor == b);
    Combine {
        // Every function but ZERO, LOCAL_ALPHA, SCALE_OTHER and SCALE_OTHER_ADD_LOCAL_ALPHA.
        local: !matches!(func, FUNC_ZERO | FUNC_LOCAL_ALPHA | FUNC_SCALE_OTHER | FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA)
            || f(FACTOR_LOCAL, FACTOR_ONE_MINUS_LOCAL),
        local_alpha: matches!(func, FUNC_LOCAL_ALPHA | FUNC_SCALE_OTHER_ADD_LOCAL_ALPHA | FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA
                                  | FUNC_SCALE_MINUS_LOCAL_ADD_LOCAL_ALPHA)
            || f(FACTOR_LOCAL_ALPHA, FACTOR_ONE_MINUS_LOCAL_ALPHA),
        other: (FUNC_SCALE_OTHER..=FUNC_SCALE_OTHER_MINUS_LOCAL_ADD_LOCAL_ALPHA).contains(&func),
        other_alpha: f(FACTOR_OTHER_ALPHA, FACTOR_ONE_MINUS_OTHER_ALPHA),
        texture_alpha: f(FACTOR_TEXTURE_ALPHA, FACTOR_ONE_MINUS_TEXTURE_ALPHA),
    }
}

/// The fields of a vertex the Voodoo reads under `s`.
pub fn used(s: &State) -> Fields {
    let (c, a) = (combine(s.cc), combine(s.ac));
    // For the alpha unit, its "local" is the local alpha and its "other" the other alpha.
    let local_alpha = a.local || a.local_alpha || c.local_alpha;
    let other_alpha = a.other || a.other_alpha || c.other_alpha;
    let other_colour = c.other || s.chroma;
    let rgb = !s.delta0 && (c.local && s.cc[2] == ITERATED || other_colour && s.cc[3] == ITERATED);
    let alpha = local_alpha && s.ac[2] == ITERATED || other_alpha && s.ac[3] == ITERATED || s.fog_mode & 0xff == FOG_WITH_ITERATED_ALPHA;
    let texture = other_colour && s.cc[3] == OTHER_TEXTURE
        || other_alpha && s.ac[3] == OTHER_TEXTURE
        || c.texture_alpha
        || a.texture_alpha;
    let depth = s.depth_mode;
    Fields {
        rgb,
        ooz: depth == DEPTH_ZBUFFER || depth == DEPTH_ZBUFFER_COMPARE_TO_BIAS,
        a: alpha,
        oow: depth == DEPTH_WBUFFER || depth == DEPTH_WBUFFER_COMPARE_TO_BIAS || s.fog_mode & 0xff == FOG_WITH_TABLE || texture,
        tmu: [
            [texture, texture && s.stw_hint & STWHINT_W_DIFF_TMU0 != 0],
            [texture && s.stw_hint & STWHINT_ST_DIFF_TMU1 != 0, texture && s.stw_hint & STWHINT_W_DIFF_TMU1 != 0],
        ],
    }
}

/// A vertex as the call log writes it: the fields read, by name, floats in their shortest
/// round-trip form. `{x 1.0 y 2.0 rgb 0.0 0.0 0.0 a 255.0 oow 0.5 tmu0 s 1.0 t 2.0}`.
pub fn write(v: &crate::vertex::Vertex, f: Fields, out: &mut String) {
    use std::fmt::Write as _;
    let _ = write!(out, "{{x {:?} y {:?}", v.x, v.y);
    if f.rgb {
        let _ = write!(out, " rgb {:?} {:?} {:?}", v.r, v.g, v.b);
    }
    if f.ooz {
        let _ = write!(out, " ooz {:?}", v.ooz);
    }
    if f.a {
        let _ = write!(out, " a {:?}", v.a);
    }
    if f.oow {
        let _ = write!(out, " oow {:?}", v.oow);
    }
    for (k, [st, w]) in f.tmu.into_iter().enumerate() {
        let [s, t, q] = v.tmu[k];
        if st {
            let _ = write!(out, " tmu{k} s {s:?} t {t:?}");
        }
        if w {
            let _ = write!(out, " tmu{k} w {q:?}");
        }
    }
    out.push('}');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertex::Vertex;

    const FUNC_LOCAL: u32 = 0x1;

    /// POD's race state: colour `SCALE_OTHER_ADD_LOCAL` by the local alpha, local iterated,
    /// other the texture; alpha the iterated alpha; a W buffer.
    fn race() -> State {
        State { cc: [0x4, 0x3, ITERATED, OTHER_TEXTURE, 0], ac: [FUNC_LOCAL, 0, ITERATED, 2, 0], depth_mode: DEPTH_WBUFFER, ..State::default() }
    }

    fn text(v: &Vertex, s: &State) -> String {
        let mut out = String::new();
        write(v, used(s), &mut out);
        out
    }

    #[test]
    fn leftovers_in_unread_fields_are_not_written() {
        // A vertex from POD's race, whose z, ooz and TMU 1 hold whatever was there before.
        let v = Vertex { x: 0.0, y: 7.0, z: 3.0550943e-37, r: 0.0, g: 0.0, b: 0.0, ooz: 4e-45, a: 255.0, oow: 0.05449183,
                         tmu: [[2.3064828, 6.920462, 234.0], [337.0, 0.15041369, 0.0]] };
        assert_eq!(text(&v, &race()), "{x 0.0 y 7.0 rgb 0.0 0.0 0.0 a 255.0 oow 0.05449183 tmu0 s 2.3064828 t 6.920462}");
        // Different leftovers write the same text.
        let w = Vertex { z: 1.0, ooz: 2.0, tmu: [[2.3064828, 6.920462, 9.0], [1.0, 2.0, 3.0]], ..v };
        assert_eq!(text(&w, &race()), text(&v, &race()));
    }

    #[test]
    fn each_field_is_written_when_the_state_reads_it() {
        let v = Vertex { x: 1.0, y: 2.0, z: 3.0, r: 4.0, g: 5.0, b: 6.0, ooz: 7.0, a: 8.0, oow: 9.0, tmu: [[10.0, 11.0, 12.0], [13.0, 14.0, 15.0]] };
        // Glide's state after grSstOpen: colour and alpha the local iterated value, no depth.
        let open = State { cc: [FUNC_LOCAL, 0, ITERATED, 2, 0], ac: [FUNC_LOCAL, 0, ITERATED, 2, 0], ..State::default() };
        assert_eq!(text(&v, &open), "{x 1.0 y 2.0 rgb 4.0 5.0 6.0 a 8.0}");
        // Flat constant colour and alpha, a Z buffer.
        let flat = State { cc: [FUNC_LOCAL, 0, 1, 2, 0], ac: [FUNC_LOCAL, 0, 1, 2, 0], depth_mode: DEPTH_ZBUFFER, ..State::default() };
        assert_eq!(text(&v, &flat), "{x 1.0 y 2.0 ooz 7.0}");
        // The delta-0 colour replaces the vertices'.
        assert_eq!(text(&v, &State { delta0: true, ..open }), "{x 1.0 y 2.0 a 8.0}");
        // Decal texture with both TMUs' own w and TMU 1's s and t, and table fog.
        let tex = State { cc: [FUNC_SCALE_OTHER, 0x8, 1, OTHER_TEXTURE, 0], ac: [FUNC_LOCAL, 0, 1, 2, 0], fog_mode: FOG_WITH_TABLE,
                          stw_hint: STWHINT_W_DIFF_TMU0 | STWHINT_ST_DIFF_TMU1 | STWHINT_W_DIFF_TMU1, ..State::default() };
        assert_eq!(text(&v, &tex), "{x 1.0 y 2.0 oow 9.0 tmu0 s 10.0 t 11.0 tmu0 w 12.0 tmu1 s 13.0 t 14.0 tmu1 w 15.0}");
        // Fog by the iterated alpha reads it, though the combines do not.
        assert_eq!(text(&v, &State { fog_mode: FOG_WITH_ITERATED_ALPHA, ..flat }), "{x 1.0 y 2.0 ooz 7.0 a 8.0}");
        // The chroma key compares the other colour: iterated, so the vertex colour is read.
        let keyed = State { cc: [FUNC_LOCAL, 0, 1, ITERATED, 0], ac: [FUNC_ZERO, 0, 1, 2, 0], chroma: true, ..State::default() };
        assert_eq!(text(&v, &keyed), "{x 1.0 y 2.0 rgb 4.0 5.0 6.0}");
        assert_eq!(text(&v, &State { chroma: false, ..keyed }), "{x 1.0 y 2.0}");
    }
}
