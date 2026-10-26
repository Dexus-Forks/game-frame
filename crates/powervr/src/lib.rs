#![doc = include_str!("../README.md")]
#![doc = ""]
//! A software model of PowerVR's PCX2 as VideoLogic's SGL 3.1 drives it.
//!
//! - [`scene`]: SGL's scene graph (lists, transforms, meshes, materials, lights, cameras,
//!   light volumes) and the traversal `sgl_render` makes, lighting each vertex as the
//!   driver does.
//! - [`pcx2`]: the per-pixel rules: deferred depth, texture effects, fog, light volumes,
//!   translucent passes, the 565 frame buffer.
//! - [`sgl`]: a software SGL context, the calls by their names, at any output scale.
//!
//! For ports of games whose PowerVR build should be drawn the way the card drew it, at any
//! size, with no SGL library and no GPU. Every place the model is not known to be exact says
//! so in a `Deviation:` comment.

#![allow(clippy::needless_range_loop)]

pub mod pcx2;
pub mod scene;
pub mod sgl;

pub use pcx2::{Frame, Viewport};
pub use scene::Scene;
pub use sgl::{MapType, Sgl};
