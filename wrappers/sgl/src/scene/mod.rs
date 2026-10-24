//! SGL's scene graph, kept as the game builds it. The game makes display lists of transforms,
//! materials, meshes, lights and cameras once (`FUN_0049d540` builds a race's scene) and each
//! frame only edits transforms, lights and which lists are ignored, then calls `sgl_render`.
//! This module keeps that graph; `walk` turns it into a frame of lit triangles seen from a
//! camera. Plain Rust, no Windows: its tests run on the host.
//!
//! - `graph`: the objects and the editing state (`Scene`), one method per SGL call.
//! - `walk`: the traversal, from the root list to triangles in camera space.
//! - `tri`: immediate mode (`sgltri_*`), faces already on the screen, as batches like `walk`'s.
//! - `shadow`: immediate mode's shadow and light volumes (`sgltri_shadow`), as planes.
//! - `convex`: a convex object's solid from its planes (light volumes).
//! - `texmem`: free texture memory, as the driver counts it.
//! - `math`: affine matrices.
//! - `dump`: the graph as text, for debugging.

pub mod convex;
pub mod dump;
pub mod graph;
pub mod math;
pub mod shadow;
pub mod texmem;
pub mod tri;
pub mod walk;

pub use graph::Scene;

/// An SGL colour, each part 0 to 1.
pub type Colour = [f32; 3];

/// SGL's error codes (sgl.h, `sgl_err_*`), as the game reads them: below zero is an error.
pub const ERR_BAD_NAME: i32 = -253;
pub const ERR_BAD_PARAMETER: i32 = -252;

/// `SGL_DEFAULT_LIST`: the root list.
pub const DEFAULT_LIST: i32 = -1;
