//! SGL's scene graph, kept as a game builds it. A game makes display lists of transforms,
//! materials, meshes, lights and cameras once and each frame only edits transforms, lights
//! and which lists are ignored, then calls `sgl_render`. This module keeps that graph; `walk`
//! turns it into a frame of lit triangles seen from a camera, which `pcx2` draws.
//!
//! - `graph`: the objects and the editing state (`Scene`), one method per SGL call.
//! - `walk`: the traversal, from the root list to triangles in camera space.
//! - `convex`: a convex object's planes (light volumes).
//! - `math`: affine matrices.
//!
//! Moved here from GameFrame's `wrappers/sgl`, which draws the same graph with OpenGL.

pub mod convex;
pub mod graph;
pub mod math;
pub mod walk;

pub use graph::Scene;

/// An SGL colour, each part 0 to 1.
pub type Colour = [f32; 3];

/// SGL's error codes (sgl.h, `sgl_err_*`), as a game reads them: below zero is an error.
pub const ERR_BAD_NAME: i32 = -253;
pub const ERR_BAD_PARAMETER: i32 = -252;

/// `SGL_DEFAULT_LIST`: the root list.
pub const DEFAULT_LIST: i32 = -1;
