//! The render thread. It owns the OpenGL context, on the game's window, and draws what the
//! game's thread sends: textures, each frame's views of the scene, and the game's 2D picture
//! to put over them. The game calls SGL and DirectDraw from its own threads, and an OpenGL
//! context belongs to one; so they only send.
//!
//! - `draw`: the 3D views into an off-screen buffer at the output size, then the 2D picture.
//! - `shaders`: the two programs.
//! - `gl`, `win32`: the entry points, declared by hand.

mod draw;
#[allow(dead_code)]
mod gl;
mod shaders;
#[allow(dead_code, clippy::upper_case_acronyms)]
mod win32;

use crate::scene::graph::Texture;
use crate::scene::walk::View;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock};

/// One `sgl_render`: the viewport (device size, rectangle, camera rectangle, the parts not
/// subtracted) and what the camera sees.
pub struct ViewDraw {
    pub device: (i32, i32),
    pub cam: [f32; 4],
    pub visible: Vec<[i32; 4]>,
    pub view: View,
}

/// The game's picture, `w × h` RGBA with alpha 0 where the marker was; `None` shows the 3D
/// frame alone. `hwnd` 0: find the game's window.
pub struct Present {
    pub hwnd: usize,
    pub w: u32,
    pub h: u32,
    pub rgba: Option<Vec<u8>>,
}

pub enum Cmd {
    Texture(u32, Texture),
    DeleteTexture(u32),
    /// A frame's views, in the order rendered.
    Frame(Vec<ViewDraw>),
    Present(Present),
}

static QUEUE: OnceLock<Mutex<Sender<Cmd>>> = OnceLock::new();

pub fn send(c: Cmd) {
    let q = QUEUE.get_or_init(|| {
        let (tx, rx) = channel();
        std::thread::spawn(move || run(rx));
        Mutex::new(tx)
    });
    let _ = q.lock().unwrap_or_else(|e| e.into_inner()).send(c);
}

fn run(rx: Receiver<Cmd>) {
    let mut r = draw::Renderer::new();
    for c in rx {
        r.exec(c);
    }
}

/// `SGL_SCALE`: the 3D frame's size as a multiple of the device's; by default the window's.
pub fn scale() -> Option<f32> {
    std::env::var("SGL_SCALE").ok().and_then(|v| v.parse().ok()).filter(|s: &f32| *s > 0.0 && *s <= 16.0)
}
