//! The 46 calls POD's `WPODPVR.EXE` imports from `sgl.dll`, under their own names (`__cdecl`, as
//! sgl.h declares them; `sgl.def`). Each reads the game's pointers and edits the scene graph
//! (`scene::Scene`); `sgl_render` hands a frame to `frame`.
//!
//! - `device`: devices, viewports, the address callback, `sgl_render`.
//! - `list`: display lists and transforms.
//! - `object`: meshes, convex objects and light volumes.
//! - `material`: the material calls, translucency and fog switches.
//! - `light`: lights and cameras, fog and background.
//! - `texture`: texture maps.
//!
//! `SGL_TRACE=N` logs the first N calls of each function with their arguments (default 0).

mod device;
mod light;
mod list;
mod material;
mod object;
mod texture;

pub use device::{Viewport, install_ini};

use crate::scene::Scene;
use crate::scene::math::V3;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

pub struct Sgl {
    pub scene: Scene,
    pub viewports: HashMap<i32, Viewport>,
    /// The device the game made: its size in pixels.
    pub device: Option<(i32, i32)>,
    next_handle: i32,
    /// Maps `sgl_preprocess_texture` made, by the pixel pointer it wrote into the game's
    /// `sgl_intermediate_map`, until `sgl_create_texture` takes them.
    pub preprocessed: HashMap<usize, Box<[u8]>>,
}

static SGL: OnceLock<Mutex<Sgl>> = OnceLock::new();

pub fn sgl() -> MutexGuard<'static, Sgl> {
    SGL.get_or_init(|| {
        Mutex::new(Sgl { scene: Scene::new(), viewports: HashMap::new(), device: None, next_handle: 1, preprocessed: HashMap::new() })
    })
    .lock()
    .unwrap_or_else(|e| e.into_inner())
}

impl Sgl {
    pub fn handle(&mut self) -> i32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }
}

/// How many calls of each function to log.
pub fn trace_limit() -> u32 {
    static N: OnceLock<u32> = OnceLock::new();
    *N.get_or_init(|| std::env::var("SGL_TRACE").ok().and_then(|v| v.parse().ok()).unwrap_or(0))
}

/// Log a call while its function has been called fewer than `SGL_TRACE` times.
macro_rules! trace {
    ($name:literal) => {
        $crate::api::trace!($name, "")
    };
    ($name:literal, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        if N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < $crate::api::trace_limit() {
            shim::log!("{} {}", $name, format_args!($fmt $(, $arg)*));
        }
    }};
}
pub(crate) use trace;

/// Three floats from the game.
pub unsafe fn v3(p: *const f32) -> V3 {
    if p.is_null() { [0.0; 3] } else { unsafe { [p.read_unaligned(), p.add(1).read_unaligned(), p.add(2).read_unaligned()] } }
}

pub unsafe fn v2(p: *const f32) -> [f32; 2] {
    if p.is_null() { [0.0; 2] } else { unsafe { [p.read_unaligned(), p.add(1).read_unaligned()] } }
}
