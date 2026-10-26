//! The render states, as `D3DOP_STATERENDER` sets them.

use crate::consts::*;

/// Every render state by its `D3DRENDERSTATETYPE` number (0 to 95).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct States(pub [u32; 96]);

impl Default for States {
    /// The values a new device starts with. Deviation: the documentation of DirectX 3 does
    /// not list every default; these are the values the later documentation gives, and a game
    /// that sets each state it uses (as the SDK samples do) does not depend on them.
    fn default() -> States {
        let mut s = [0u32; 96];
        s[rs::TEXTUREADDRESS as usize] = ADDRESS_WRAP;
        s[rs::FILLMODE as usize] = FILL_SOLID;
        s[rs::SHADEMODE as usize] = SHADE_GOURAUD;
        s[rs::ZWRITEENABLE as usize] = 1;
        s[rs::LASTPIXEL as usize] = 1;
        s[rs::TEXTUREMAG as usize] = FILTER_NEAREST;
        s[rs::TEXTUREMIN as usize] = FILTER_NEAREST;
        s[rs::SRCBLEND as usize] = BLEND_ONE;
        s[rs::DESTBLEND as usize] = BLEND_ZERO;
        s[rs::TEXTUREMAPBLEND as usize] = TBLEND_MODULATE;
        s[rs::CULLMODE as usize] = CULL_CCW;
        s[rs::ZFUNC as usize] = CMP_LESSEQUAL;
        s[rs::ALPHAFUNC as usize] = CMP_ALWAYS;
        s[rs::PLANEMASK as usize] = u32::MAX;
        States(s)
    }
}

impl States {
    /// Sets one state; a number past the table is ignored.
    pub fn set(&mut self, t: u32, v: u32) {
        if let Some(s) = self.0.get_mut(t as usize) {
            *s = v;
        }
    }

    pub fn get(&self, t: u32) -> u32 {
        self.0.get(t as usize).copied().unwrap_or(0)
    }

    pub fn on(&self, t: u32) -> bool {
        self.get(t) != 0
    }

    /// Is the stipple pattern's bit at pixel `(x, y)` set? Row `y mod 32` is
    /// `STIPPLEPATTERN` *nn*, bit `x mod 32` counted from the low bit (*inferred*: the
    /// documentation gives the 32 × 32 pattern but not its bit order).
    pub fn stipple(&self, x: usize, y: usize) -> bool {
        self.get(rs::STIPPLEPATTERN00 + (y % 32) as u32) >> (x % 32) & 1 != 0
    }
}

/// A `D3DCMPFUNC` test of `new` against `old`.
pub fn compare(func: u32, new: f32, old: f32) -> bool {
    match func {
        CMP_NEVER => false,
        CMP_LESS => new < old,
        CMP_EQUAL => new == old,
        CMP_LESSEQUAL => new <= old,
        CMP_GREATER => new > old,
        CMP_NOTEQUAL => new != old,
        CMP_GREATEREQUAL => new >= old,
        _ => true,
    }
}
