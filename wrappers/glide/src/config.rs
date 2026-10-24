//! Settings, from environment variables (Wine passes the Unix environment through):
//!
//! - `GLIDE_SCALE`: the output size as a multiple of the game's resolution (`2`, `4`, `1.5`;
//!   default 2).
//! - `GLIDE_SIZE`: the output size in pixels (`1920x1440`); wins over `GLIDE_SCALE`. The
//!   picture keeps its 4:3 shape inside it.
//! - `GLIDE_LFB_FILTER`: `linear` to smooth the game's 2D pictures when scaling them up
//!   (default `nearest`).
//! - `GLIDE_LOG`: a file to log to (a Windows path such as `C:\glide.log`).
//! - `GLIDE_SCALE_FILE`: a file holding a scale, read again at each `grSstOpen`; when it
//!   exists it wins over `GLIDE_SCALE`. The game opens the card again for each race, so the
//!   menus can run small and the race large (the menus are slow in software OpenGL).

use std::sync::OnceLock;

pub struct Config {
    pub scale: f32,
    pub size: Option<(u32, u32)>,
    pub lfb_linear: bool,
    pub scale_file: Option<String>,
}

impl Config {
    /// The size of the drawing surface for a game resolution of `w × h`.
    pub fn output(&self, w: u32, h: u32) -> (u32, u32) {
        match self.size {
            Some(s) => s,
            None => {
                let scale = self
                    .scale_file
                    .as_ref()
                    .and_then(|f| std::fs::read_to_string(f).ok())
                    .and_then(|v| v.trim().parse::<f32>().ok())
                    .filter(|s| *s > 0.0)
                    .unwrap_or(self.scale);
                ((w as f32 * scale).round() as u32, (h as f32 * scale).round() as u32)
            }
        }
    }
}

pub fn get() -> &'static Config {
    static C: OnceLock<Config> = OnceLock::new();
    C.get_or_init(|| {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let scale = var("GLIDE_SCALE").and_then(|v| v.parse::<f32>().ok()).filter(|s| *s > 0.0).unwrap_or(2.0);
        let size = var("GLIDE_SIZE").and_then(|v| {
            let (w, h) = v.split_once(['x', 'X'])?;
            Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
        });
        Config {
            scale,
            size,
            lfb_linear: var("GLIDE_LFB_FILTER").is_some_and(|v| v.eq_ignore_ascii_case("linear")),
            scale_file: var("GLIDE_SCALE_FILE"),
        }
    })
}
