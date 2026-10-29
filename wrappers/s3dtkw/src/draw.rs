//! Drawing one `S3DTK_TriangleSet` call into the game's surface, with GameFrame's model of
//! the chip (`gameframe_virge::draw`).
//!
//! The draw surface is an `S3DTK_SURFACE` (`sfOffset`, `sfWidth`, `sfHeight`, `sfFormat`); POD
//! gives `S3DTK_VIDEORGB15` (4), the only 16-bit format the ViRGE's 3D engine draws, so the
//! DirectDraw surfaces of this build are 5-5-5 (`DDRAW_PIXELS=555`).

use crate::table::State;
use virge::draw::{Vertex32, key, surface_format, triangle_set};
use virge::Target;

pub unsafe fn triangles(s: &State, list: *const u32, n: u32, kind: u32) {
    let surf = s.values[key::DRAWSURFACE] as *const u32;
    if surf.is_null() || list.is_null() {
        return;
    }
    let word = |i: usize| unsafe { surf.add(i).read_unaligned() };
    let (ptr, w, h, sf_format) = (word(0) as *mut u8, word(1) as usize, word(2) as usize, word(3));
    let Some(format) = surface_format(sf_format) else {
        shim::gap!("surface format", "{sf_format:#x} (24-bit) is not drawn");
        return;
    };
    let pitch = w * format.bytes();
    let data = unsafe { std::slice::from_raw_parts_mut(ptr, pitch * h) };
    let mut t = Target::new(data, pitch, w, h, format);
    let verts: Vec<Vertex32> = (0..n as usize)
        .map(|i| {
            let v = unsafe { list.add(i).read_unaligned() } as *const [u8; 32];
            let bytes = unsafe { v.read_unaligned() };
            Vertex32::from_bytes(&bytes)
        })
        .collect();
    let mut state = virge::State::default();
    state.values.copy_from_slice(&s.values[..24]);
    for gap in triangle_set(&state, &mut t, &verts, kind) {
        shim::log::gap(gap.what, format_args!("{:#x}", gap.value));
    }
}
