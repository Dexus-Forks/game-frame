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
//! - `convex`: a convex object's planes (light volumes).
//! - `texmem`: free texture memory, as the driver counts it.
//! - `math`: affine matrices.
//! - `dump`: the graph as text, for debugging.

pub mod dump;
pub mod shadow;
pub mod texmem;
pub mod tri;

// The graph, its traversal, the planes and the matrices are GameFrame's `gameframe-powervr`,
// which also draws them in software.
pub use powervr::scene::{Colour, DEFAULT_LIST, ERR_BAD_NAME, ERR_BAD_PARAMETER, Scene, convex, graph, math, walk};
