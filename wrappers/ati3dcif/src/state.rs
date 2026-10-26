//! The context's render state, as `ATI3DCIF_ContextSetState` sets it (`C3D_ERSID` values,
//! `rage::RENDER_STATES`).

pub const PRIM_TYPE: u32 = 2;
pub const SOLID_CLR: u32 = 3;
pub const SHADE_MODE: u32 = 4;
pub const ALPHA_SRC: u32 = 11;
pub const ALPHA_DST: u32 = 12;
pub const SURF_DRAW_PTR: u32 = 13;
pub const SURF_DRAW_PITCH: u32 = 14;
pub const SURF_DRAW_PF: u32 = 15;

pub struct Context {
    /// Every state's last value (the word `C3D_PRSDATA` points at), by id.
    pub values: [u32; 40],
    /// How often each state was set, for the log.
    pub sets: [u32; 40],
}

impl Context {
    pub const fn new() -> Context {
        Context { values: [0; 40], sets: [0; 40] }
    }

    pub fn get(&self, id: u32) -> u32 {
        self.values.get(id as usize).copied().unwrap_or(0)
    }
}
