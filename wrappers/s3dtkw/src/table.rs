//! `S3DTK_FUNCTIONLIST`: the table the game draws through. Its first argument is always the
//! table itself. POD calls slots 0 (`SetState`), 1 (`GetState`) and 2 (`TriangleSet`). Each
//! returns 1 on success: the game stops with "S3DTK Error : SetState" on anything else.

use gpu_1997::s3::state_name;
use shim::log;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

type Fn4 = unsafe extern "C" fn(*mut Table, u32, u32, u32) -> i32;

#[repr(C)]
pub struct Table {
    slots: [Fn4; 16],
}

pub struct State {
    /// Every state's last value, by `S3DTK_STATEKEY`.
    pub values: [u32; 32],
    sets: [u32; 32],
}

pub static STATE: Mutex<State> = Mutex::new(State { values: [0; 32], sets: [0; 32] });

unsafe extern "C" fn set_state(_t: *mut Table, key: u32, value: u32, _: u32) -> i32 {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(v) = s.values.get_mut(key as usize) {
        *v = value;
        s.sets[key as usize] += 1;
        if s.sets[key as usize] <= 4 {
            log!("SetState {} ({key}) = {value:#x}", state_name(key).unwrap_or("?"));
        }
    }
    1
}

/// The game asks only for `S3DTK_VIDEOMEMORYADDRESS` (2), and takes 1 as success
/// (`0x4a5fb1`). Video memory is said to start at 0, so the offsets the game derives from
/// linear addresses (`sfOffset`) are the linear addresses themselves.
unsafe extern "C" fn get_state(_t: *mut Table, key: u32, out: u32, _: u32) -> i32 {
    log!("GetState {} ({key})", state_name(key).unwrap_or("?"));
    if out != 0 {
        unsafe { (out as *mut u32).write_unaligned(0) };
    }
    1
}

/// Calls by rendering type (0 Gouraud, 1 lit texture, 2 unlit texture, 3 other), for the log.
static COUNTS: [AtomicU32; 4] = [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];

unsafe extern "C" fn triangle_set(_t: *mut Table, list: u32, n: u32, kind: u32) -> i32 {
    let rtype = STATE.lock().unwrap_or_else(|e| e.into_inner()).values[10];
    let k = match rtype { 0 => 0, 0x0800_0000 => 1, 0x1000_0000 => 2, _ => 3 };
    let seen = COUNTS[k].fetch_add(1, Ordering::Relaxed);
    if seen % 5000 == 4999 {
        log!("TriangleSet calls by type: {:?}", COUNTS.iter().map(|c| c.load(Ordering::Relaxed)).collect::<Vec<_>>());
    }
    if seen < 3 && list != 0 {
        let s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let surf = |p: u32| if p == 0 { String::new() } else { unsafe { (0..8).map(|i| format!("{:#x}", (p as *const u32).add(i).read_unaligned())).collect::<Vec<_>>().join(" ") } };
        log!("TriangleSet {n} kind {kind} type {:#x} draw [{}] tex [{}]", s.values[10], surf(s.values[4]), surf(s.values[15]));
        for i in 0..n as usize {
            let v = unsafe { (list as *const u32).add(i).read_unaligned() } as *const u32;
            let w: Vec<String> = (0..12).map(|j| {
                let x = unsafe { v.add(j).read_unaligned() };
                format!("{:.3}/{x:#x}", f32::from_bits(x))
            }).collect();
            log!("  {v:?}: {}", w.join(" "));
        }
    }
    let s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { crate::draw::triangles(&s, list as *const u32, n, kind) };
    1
}

unsafe extern "C" fn other(_t: *mut Table, a: u32, b: u32, c: u32) -> i32 {
    log!("unexpected S3DTK table call ({a:#x}, {b:#x}, {c:#x})");
    0
}

pub fn new() -> *mut Table {
    let mut slots: [Fn4; 16] = [other; 16];
    slots[0] = set_state;
    slots[1] = get_state;
    slots[2] = triangle_set;
    Box::into_raw(Box::new(Table { slots }))
}
