//! The six exported calls.

use crate::{OK, context, state};
use rage::render_state_name;
use shim::{log, log_once};
use std::sync::atomic::{AtomicU32, Ordering};

#[unsafe(no_mangle)]
pub extern "system" fn ATI3DCIF_Init() -> u32 {
    shim::call!("ATI3DCIF_Init");
    log!("ATI3DCIF_Init");
    OK
}

/// One context: the handle is never looked at again by the game except to pass it back.
#[unsafe(no_mangle)]
pub extern "system" fn ATI3DCIF_ContextCreate() -> u32 {
    shim::call!("ATI3DCIF_ContextCreate");
    log!("ATI3DCIF_ContextCreate");
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ATI3DCIF_ContextSetState(_ctx: u32, id: u32, data: *const u32) -> u32 {
    shim::call!("ATI3DCIF_ContextSetState", _ctx, id, Word(data));
    let v = if data.is_null() { 0 } else { unsafe { data.read_unaligned() } };
    let mut c = context();
    if let Some(slot) = c.values.get_mut(id as usize) {
        *slot = v;
        c.sets[id as usize] += 1;
        if c.sets[id as usize] <= 4 {
            log!("ContextSetState {} ({id}) = {v:#x}", render_state_name(id).unwrap_or("?"));
        }
    }
    OK
}

#[unsafe(no_mangle)]
pub extern "system" fn ATI3DCIF_RenderBegin(_ctx: u32) -> u32 {
    shim::call!("ATI3DCIF_RenderBegin", _ctx);
    OK
}

#[unsafe(no_mangle)]
pub extern "system" fn ATI3DCIF_RenderEnd() -> u32 {
    shim::call!("ATI3DCIF_RenderEnd");
    shim::calls::end_frame();
    OK
}

static DUMPS: AtomicU32 = AtomicU32::new(0);

/// For the call log (`WRAPPER_CALLS`): the word a state's pointer points to.
struct Word(*const u32);
impl shim::calls::Arg for Word {
    fn put(&self, out: &mut String) {
        match self.0.is_null() {
            true => out.push_str("null"),
            false => unsafe { self.0.read_unaligned() }.put(out),
        }
    }
}

/// For the call log: each `C3D_VTCF` of a primitive list as its ten floats
/// (x, y, z, s, t, w, r, g, b, a).
struct Vtcf(*const *const u32, u32);
impl shim::calls::Arg for Vtcf {
    fn put(&self, out: &mut String) {
        if self.0.is_null() {
            return out.push_str("null");
        }
        out.push('[');
        for i in 0..self.1 as usize {
            let v = unsafe { self.0.add(i).read_unaligned() };
            let fs: Vec<String> = (0..10).map(|j| format!("{:?}", f32::from_bits(unsafe { v.add(j).read_unaligned() }))).collect();
            out.push_str(&format!("{}{{{}}}", if i == 0 { "" } else { " " }, fs.join(" ")));
        }
        out.push(']');
    }
}

/// `vlist`: `n` pointers to vertices.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ATI3DCIF_RenderPrimList(vlist: *const *const u32, n: u32) -> u32 {
    shim::call!("ATI3DCIF_RenderPrimList", Vtcf(vlist, n), n);
    if vlist.is_null() || n == 0 {
        return OK;
    }
    let vs: Vec<*const u32> = (0..n as usize).map(|i| unsafe { vlist.add(i).read_unaligned() }).collect();
    if DUMPS.fetch_add(1, Ordering::Relaxed) < 6 {
        let c = context();
        log!(
            "RenderPrimList {n} prim {} shade {} draw {:#x} pitch {} pf {} alpha {}/{}",
            c.get(state::PRIM_TYPE), c.get(state::SHADE_MODE), c.get(state::SURF_DRAW_PTR),
            c.get(state::SURF_DRAW_PITCH), c.get(state::SURF_DRAW_PF), c.get(state::ALPHA_SRC), c.get(state::ALPHA_DST)
        );
        for v in &vs {
            let w: Vec<String> = (0..12).map(|i| {
                let x = unsafe { v.add(i).read_unaligned() };
                format!("{:.3}", f32::from_bits(x))
            }).collect();
            log!("  {:?}: {}", v, w.join(" "));
        }
    }
    let c = context();
    // Each combination of primitive, shading and blend once, with its first vertex.
    {
        static SEEN: std::sync::Mutex<Vec<[u32; 4]>> = std::sync::Mutex::new(Vec::new());
        let key = [c.get(state::PRIM_TYPE), c.get(state::SHADE_MODE), c.get(state::ALPHA_SRC), c.get(state::ALPHA_DST)];
        let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
        if !seen.contains(&key) {
            seen.push(key);
            let v = vs[0];
            let w: Vec<String> = (0..10).map(|i| format!("{:.1}", f32::from_bits(unsafe { v.add(i).read_unaligned() }))).collect();
            log!("new combination prim/shade/src/dst {key:?}, {n} vertices, first {}", w.join(" "));
        }
    }
    unsafe { crate::raster::prim(&c, &vs) };
    log_once!("first primitive drawn");
    OK
}
