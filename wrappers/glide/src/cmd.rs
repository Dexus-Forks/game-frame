//! The commands the exported Glide calls record, and the queue that carries them to the render
//! thread. The game may call Glide from more than one thread, and an OpenGL context belongs to
//! one; so every call only records, and one thread (`render`) owns the context and replays the
//! commands in order. A frame's commands go over at `grBufferSwap`; a read of the frame buffer
//! sends what is queued and waits for the answer.

use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Mutex, OnceLock};

/// A Glide `GrVertex` (Glide 2.x with two TMUs): 15 floats, 60 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub ooz: f32,
    pub a: f32,
    pub oow: f32,
    pub tmu: [[f32; 3]; 2],
}

/// A `GrTexInfo` without its data pointer: what selects or describes a texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TexInfo {
    pub small_lod: u32,
    pub large_lod: u32,
    pub aspect: u32,
    pub format: u32,
}

/// Pixels the game wrote to the frame buffer, as RGBA with alpha 0 where it wrote nothing.
pub struct LfbImage {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    /// Through the pixel pipeline (`grLfbBypassMode(GR_LFBBYPASS_DISABLE)`): blended with
    /// the current blend function, using the pixels' alpha.
    pub pipeline: bool,
}

/// Which buffer a read wants, and where to send it: 16-bit values, 1024 per row.
pub struct Read {
    pub buffer: u32,
    pub reply: SyncSender<Vec<u16>>,
}

pub enum Cmd {
    /// A resolution opened; the window (an `HWND`) to draw in, sized to the output.
    Open { width: u32, height: u32, out: (u32, u32), hwnd: usize },
    ColorCombine([u32; 5]),
    AlphaCombine([u32; 5]),
    ConstantColor(u32),
    AlphaBlend([u32; 4]),
    DepthMode(u32),
    DepthFunc(u32),
    DepthMask(bool),
    ChromaMode(bool),
    ChromaValue(u32),
    FogMode(u32),
    FogColor(u32),
    FogTable(Box<[u8; 64]>),
    Gamma(f32),
    Clip([u32; 4]),
    TexCombine(u32),
    TexFilter(u32, u32),
    TexClamp(u32, u32),
    TexSource { start: u32, info: TexInfo },
    Download { start: u32, data: Vec<u8> },
    Palette(Box<[u32; 256]>),
    Triangle([Vertex; 3]),
    Line([Vertex; 2]),
    Clear { color: u32, alpha: u32, depth: u32 },
    Lfb(LfbImage),
    Swap,
    Read(Read),
}

struct Queue {
    pending: Mutex<Vec<Cmd>>,
    send: SyncSender<Vec<Cmd>>,
}

static QUEUE: OnceLock<Queue> = OnceLock::new();

/// Start the render thread (once) and return the queue.
fn queue() -> &'static Queue {
    QUEUE.get_or_init(|| {
        // One frame in flight: a game faster than the renderer waits at grBufferSwap.
        let (send, recv): (SyncSender<Vec<Cmd>>, Receiver<Vec<Cmd>>) = sync_channel(1);
        std::thread::Builder::new()
            .name("glide-render".into())
            .spawn(move || crate::render::run(recv))
            .expect("render thread");
        Queue { pending: Mutex::new(Vec::with_capacity(4096)), send }
    })
}

/// Record a command.
pub fn push(c: Cmd) {
    queue().pending.lock().unwrap_or_else(|e| e.into_inner()).push(c);
}

/// Send what is recorded to the render thread.
pub fn flush() {
    let q = queue();
    let batch = std::mem::take(&mut *q.pending.lock().unwrap_or_else(|e| e.into_inner()));
    if !batch.is_empty() {
        let _ = q.send.send(batch);
    }
}

/// Read a buffer of the current frame: send everything recorded so far, then wait.
pub fn read(buffer: u32) -> Vec<u16> {
    let (reply, answer) = sync_channel(1);
    push(Cmd::Read(Read { buffer, reply }));
    flush();
    answer.recv().unwrap_or_default()
}
