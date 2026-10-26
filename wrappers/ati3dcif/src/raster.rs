//! Drawing one `ATI3DCIF_RenderPrimList` call into the game's surface, with GameFrame's model
//! of the chip (`gameframe_rage::draw`).
//!
//! The surface is the DirectDraw back buffer (`SURF_DRAW_PTR`), `SURF_DRAW_PITCH` pixels to a
//! row; its height is taken as three quarters of the pitch (640 × 480).

use crate::state::*;
use rage::{Target, Vtcf, render_prim_list};

pub unsafe fn prim(c: &Context, vs: &[*const u32]) {
    let ptr = c.get(SURF_DRAW_PTR) as *mut u8;
    let pitch = c.get(SURF_DRAW_PITCH) as usize;
    if ptr.is_null() || pitch == 0 {
        return;
    }
    let mut rage = rage::Context::default();
    rage.values.copy_from_slice(&c.values);
    let format = rage.format();
    let (w, h) = (pitch, pitch * 3 / 4);
    let data = unsafe { std::slice::from_raw_parts_mut(ptr, pitch * 2 * h) };
    let mut t = Target::new(data, pitch * 2, w, h, format);
    let f = |v: *const u32, i: usize| f32::from_bits(unsafe { v.add(i).read_unaligned() });
    let verts: Vec<Vtcf> = vs.iter().map(|&v| Vtcf::from_words(std::array::from_fn(|i| f(v, i)))).collect();
    if let Err(gap) = render_prim_list(&rage, &mut t, &verts) {
        shim::log::gap(gap.what, format_args!("{}", gap.value));
    }
}
